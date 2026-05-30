from __future__ import annotations

import json
import os
import pty
import re
import select
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Callable

SQ_BIN = os.environ.get("SQ_BIN", "/opt/sq")
ANSI_RE = re.compile(r"\x1b\[[0-9;]*[mGKHJ]")
SQ_TIMEOUT_S = 20
TIMEOUT_EXIT_CODE = 124

DANGER_CWDS = {
    2: "/zones/src",
    3: "/tmp",
    4: "/dev",
    5: "/zones/node_modules",
}

_InvocationRecorder = Callable[[list[str], str, int, str, str, int], None]
_recorder: _InvocationRecorder | None = None

CRAFT_CMDS = ["git commit", "git push", "cargo build", "npm install", "make"]
BENIGN_CMDS = ["ls", "cd", "pwd", "echo hi"]
FAIL_CMDS = ["bad_command_xyz", "ls /nonexistent", "git pull"]


def set_invocation_recorder(fn: _InvocationRecorder | None) -> None:
    global _recorder
    _recorder = fn


def _record_invocation(argv: list[str], cwd: str, exit_code: int,
                       stdout: str, stderr: str, duration_ms: int) -> None:
    if _recorder is None:
        return
    try:
        _recorder(argv, cwd, exit_code, stdout, stderr, duration_ms)
    except Exception as e:
        print(f"[balance-sim] sq invocation recorder failed: {type(e).__name__}: {e}",
              file=sys.stderr)


def _output_text(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value


def _status_exit_code(status: int | None) -> int:
    if status is None:
        return TIMEOUT_EXIT_CODE
    if os.WIFEXITED(status):
        return os.WEXITSTATUS(status)
    if os.WIFSIGNALED(status):
        return -os.WTERMSIG(status)
    return TIMEOUT_EXIT_CODE


def _wait_for_child(pid: int) -> int | None:
    try:
        _, status = os.waitpid(pid, 0)
    except OSError:
        return None
    return status


def _run_sq(home: Path, args: list[str], cwd: str | None = None) -> tuple[int, str, str]:
    env = os.environ.copy()
    env["HOME"] = str(home)
    env["SQ_NO_PACING"] = "1"
    env.setdefault("SQ_DEBUG", "1")
    env.setdefault("RUST_BACKTRACE", "full")
    argv = [SQ_BIN, *args]
    effective_cwd = cwd or str(home)
    start = time.perf_counter()
    try:
        proc = subprocess.run(
            argv,
            env=env, cwd=effective_cwd,
            capture_output=True, text=True, check=False, timeout=SQ_TIMEOUT_S,
        )
    except subprocess.TimeoutExpired as e:
        duration_ms = SQ_TIMEOUT_S * 1000
        _record_invocation(argv, effective_cwd, TIMEOUT_EXIT_CODE,
                           _output_text(e.stdout), _output_text(e.stderr),
                           duration_ms)
        raise
    duration_ms = int((time.perf_counter() - start) * 1000)
    _record_invocation(argv, effective_cwd, proc.returncode,
                       proc.stdout, proc.stderr, duration_ms)
    return proc.returncode, proc.stdout, proc.stderr


def init_save(home: Path, save_json: dict[str, Any]) -> None:
    save_dir = home / ".shellquest"
    save_dir.mkdir(parents=True, exist_ok=True)
    (save_dir / "save.json").write_text(json.dumps(save_json, indent=2))


def read_save(home: Path) -> dict[str, Any]:
    path = home / ".shellquest" / "save.json"
    return json.loads(path.read_text())


def write_save(home: Path, save: dict[str, Any]) -> None:
    path = home / ".shellquest" / "save.json"
    path.write_text(json.dumps(save, indent=2))


def derive_state(save: dict[str, Any]) -> dict[str, Any]:
    c = save["character"]
    w = c.get("weapon")
    a = c.get("armor")
    r = c.get("ring")
    return {
        "level": c["level"], "xp": c["xp"],
        "hp": c["hp"], "max_hp": c["max_hp"],
        "gold": c["gold"], "kills": c["kills"], "deaths": c["deaths"],
        "strength": c["strength"], "dexterity": c["dexterity"],
        "intelligence": c["intelligence"],
        "attack_power": (
            c["strength"] + c["dexterity"] // 2
            + ((w["power"] + w["enchant_level"]) if w else 0)
        ),
        "defense": (
            c["dexterity"] // 3
            + ((a["power"] + a["enchant_level"]) if a else 0)
            + ((r["power"] + r["enchant_level"]) if r else 0)
        ),
        "inventory_count": len(c.get("inventory", [])),
        "weapon_power": w["power"] + w["enchant_level"] if w else None,
        "armor_power": a["power"] + a["enchant_level"] if a else None,
        "ring_power": r["power"] + r["enchant_level"] if r else None,
        "weapon_rarity": w["rarity"] if w else None,
        "armor_rarity": a["rarity"] if a else None,
        "ring_rarity": r["rarity"] if r else None,
        "prestige": c.get("prestige", 0),
    }


def cmd_tick(home: Path, cmd: str, cwd: str, exit_code: int) -> tuple[int, str]:
    rc, _stdout, stderr = _run_sq(home, [
        "tick", "--cmd", cmd, "--cwd", cwd, "--exit-code", str(exit_code),
    ], cwd=str(home))
    return rc, stderr


def _encounter_int(fields: dict[str, str], key: str) -> int:
    try:
        return int(fields.get(key, "0"))
    except (TypeError, ValueError):
        return 0


def _encounter_enemy_name(value: str) -> str:
    try:
        return bytes.fromhex(value).decode("utf-8", errors="replace")
    except (TypeError, ValueError):
        return f"?{value}"


def parse_encounter_lines(stderr: str) -> list[dict[str, object]]:
    encounters: list[dict[str, object]] = []
    for raw_line in stderr.splitlines():
        line = raw_line.strip()
        if not line.startswith("SQ_ENCOUNTER "):
            continue
        try:
            fields: dict[str, str] = {}
            for token in line[len("SQ_ENCOUNTER "):].split():
                key, sep, value = token.partition("=")
                if sep and key:
                    fields[key] = value
            encounters.append({
                "kind": fields.get("kind", ""),
                "enemy_name": _encounter_enemy_name(fields.get("enemy", "")),
                "elite": _encounter_int(fields, "elite"),
                "dmg_dealt": _encounter_int(fields, "dmg_dealt"),
                "dmg_taken": _encounter_int(fields, "dmg_taken"),
                "outcome": fields.get("outcome", ""),
                "xp_earned": _encounter_int(fields, "xp"),
                "gold_earned": _encounter_int(fields, "gold"),
            })
        except Exception:
            continue
    return encounters


def cmd_shop_list(home: Path) -> tuple[int, str]:
    rc, _stdout, stderr = _run_sq(home, ["shop"])
    return rc, stderr


def cmd_shop_buy(home: Path, index: int) -> tuple[int, str]:
    rc, _stdout, stderr = _run_sq(home, ["buy", str(index)])
    return rc, stderr


def cmd_enchant(home: Path, item_ref: str) -> tuple[int, str]:
    rc, _stdout, stderr = _run_sq(home, ["enchant", item_ref])
    return rc, stderr


def cmd_sell(home: Path, item_ref: str) -> tuple[int, str]:
    rc, _stdout, stderr = _run_sq(home, ["sell", item_ref])
    return rc, stderr


def cmd_equip(home: Path, item_ref: str) -> tuple[int, str]:
    rc, _stdout, stderr = _run_sq(home, ["equip", item_ref])
    return rc, stderr


def cmd_arena(home: Path, tier_choice: int, max_rounds: int,
              timeout_s: int = 180) -> str:
    env = os.environ.copy()
    env["HOME"] = str(home)
    env["SQ_NO_PACING"] = "1"
    env.setdefault("SQ_DEBUG", "1")
    env.setdefault("RUST_BACKTRACE", "full")
    env["TERM"] = "xterm-256color"

    start = time.perf_counter()
    pid, fd = pty.fork()
    if pid == 0:
        os.execvpe(SQ_BIN, [SQ_BIN, "arena"], env)

    chunks = []
    inputs = [f"{tier_choice}\r".encode(), b"y\r"] + [b"1\r"] * (max_rounds + 5)
    sent_idx = 0
    last_send = 0.0
    deadline = time.time() + timeout_s
    timed_out = False
    while time.time() < deadline:
        try:
            rlist, _, _ = select.select([fd], [], [], 0.5)
        except OSError:
            break
        if rlist:
            try:
                data = os.read(fd, 8192)
            except OSError:
                break
            if not data:
                break
            chunks.append(data.decode("utf-8", errors="replace"))
        full = "".join(chunks)
        if sent_idx < len(inputs) and time.time() - last_send > 1.5:
            prompts = (full.count("Choose [1-2]") + full.count("Select tier")
                       + full.count("[y/N]"))
            if prompts >= sent_idx:
                os.write(fd, inputs[sent_idx])
                sent_idx += 1
                last_send = time.time()
        if "Knocked out" in full or "Victor" in full:
            time.sleep(1.0)
            try:
                data = os.read(fd, 16384)
                chunks.append(data.decode("utf-8", errors="replace"))
            except OSError:
                pass
            break
    else:
        timed_out = True
    if timed_out:
        try:
            os.kill(pid, signal.SIGKILL)
        except OSError:
            pass
    try:
        os.close(fd)
    except OSError:
        pass
    status = _wait_for_child(pid)
    transcript = ANSI_RE.sub("", "".join(chunks))
    duration_ms = int((time.perf_counter() - start) * 1000)
    stderr = f"arena automation timeout after {timeout_s}s" if timed_out else ""
    exit_code = TIMEOUT_EXIT_CODE if timed_out else _status_exit_code(status)
    _record_invocation([SQ_BIN, "arena"], str(home), exit_code, transcript, stderr, duration_ms)
    return transcript


PLAYER_DMG_RE = re.compile(r"for (\d+) damage")
ENEMY_DMG_RE = re.compile(r"(\d+) damage")
ROUND_INTRO_RE = re.compile(r"⚔️\s*Round (\d+)")
ROUND_CLEARED_RE = re.compile(r"Round (\d+) cleared")
ENTRY_FEE_RE = re.compile(r"Entry fee: (\d+) gold")


def parse_arena_output(text: str, character_level: int, tier: str,
                       tier_index: int) -> dict[str, Any]:
    rounds_won = 0
    rounds_attempted = 0
    p_swings = 0
    p_crits = 0
    e_swings = 0
    e_hits = 0
    e_crits = 0
    dmg_dealt = 0
    dmg_taken = 0
    cur_round = 0
    entry_fee = 0
    outcome = "unknown"
    gold_earned = 0
    xp_earned = 0

    for line in text.splitlines():
        s = line.strip()
        m_fee = ENTRY_FEE_RE.search(s)
        if m_fee:
            entry_fee = int(m_fee.group(1))
        m = ROUND_INTRO_RE.search(s)
        if m:
            cur_round = int(m.group(1))
            rounds_attempted = max(rounds_attempted, cur_round)
            continue
        if cur_round == 0:
            continue
        if "Round" in s and "cleared" in s.lower():
            rounds_won = max(rounds_won, cur_round)
            continue
        if "CRITICAL" in s and "Your" in s:
            p_crits += 1
            p_swings += 1
            d = PLAYER_DMG_RE.search(s)
            if d:
                dmg_dealt += int(d.group(1))
        elif "CRITICAL" in s and "The" in s:
            e_crits += 1
            e_hits += 1
            e_swings += 1
            d = ENEMY_DMG_RE.search(s)
            if d:
                dmg_taken += int(d.group(1))
        elif s.startswith("Your") and "damage" in s:
            p_swings += 1
            d = PLAYER_DMG_RE.search(s)
            if d:
                dmg_dealt += int(d.group(1))
        elif s.startswith("Your"):
            p_swings += 1
        elif s.startswith("The") and "damage" in s.lower():
            e_swings += 1
            e_hits += 1
            d = ENEMY_DMG_RE.search(s)
            if d:
                dmg_taken += int(d.group(1))
        elif s.startswith("The"):
            e_swings += 1
        if "Knocked out" in s:
            outcome = "defeat"
        if "Victorious" in s or "Champion" in s:
            outcome = "victory"
        if "Cash" in s and "Out" in s and "now" not in s.lower():
            outcome = "cashout"

    if outcome == "unknown" and rounds_won == rounds_attempted and rounds_attempted > 0:
        outcome = "cashout"

    return {
        "character_level": character_level,
        "tier": tier,
        "tier_index": tier_index,
        "entry_fee": entry_fee,
        "rounds_attempted": rounds_attempted,
        "rounds_won": rounds_won,
        "outcome": outcome,
        "gold_earned": gold_earned,
        "xp_earned": xp_earned,
        "dmg_dealt": dmg_dealt,
        "dmg_taken": dmg_taken,
        "enemy_crits": e_crits,
        "player_crits": p_crits,
        "player_swings": p_swings,
        "enemy_swings": e_swings,
    }


def has_segment_zone(danger: int) -> str:
    return cwd_for_danger(danger, os.environ.get("HOME", "/sim-home"))


def cwd_for_danger(danger: int, home: Path | str) -> str:
    if danger == 1:
        return str(home)
    return DANGER_CWDS.get(danger, "/tmp")
