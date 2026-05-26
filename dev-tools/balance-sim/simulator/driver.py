from __future__ import annotations

import json
import os
import pty
import re
import select
import subprocess
import time
from pathlib import Path
from typing import Any

SQ_BIN = "/home/duys/.repos/shellquest/target/debug/sq"
ANSI_RE = re.compile(r"\x1b\[[0-9;]*[mGKHJ]")

DANGER_CWDS = {
    1: "/home/duys",
    2: "/home/duys/.repos/shellquest/src",
    3: "/tmp",
    4: "/dev",
    5: "/home/duys/.repos/shellquest/node_modules-fake",
}

CRAFT_CMDS = ["git commit", "git push", "cargo build", "npm install", "make"]
BENIGN_CMDS = ["ls", "cd", "pwd", "echo hi"]
FAIL_CMDS = ["bad_command_xyz", "ls /nonexistent", "git pull"]


def _run_sq(home: Path, args: list[str], cwd: str | None = None) -> tuple[int, str, str]:
    env = os.environ.copy()
    env["HOME"] = str(home)
    env["SQ_NO_PACING"] = "1"
    proc = subprocess.run(
        [SQ_BIN, *args],
        env=env, cwd=cwd or str(home),
        capture_output=True, text=True, check=False, timeout=20,
    )
    return proc.returncode, proc.stdout, proc.stderr


def init_save(home: Path, save_json: dict) -> None:
    save_dir = home / ".shellquest"
    save_dir.mkdir(parents=True, exist_ok=True)
    (save_dir / "save.json").write_text(json.dumps(save_json, indent=2))


def read_save(home: Path) -> dict:
    path = home / ".shellquest" / "save.json"
    return json.loads(path.read_text())


def write_save(home: Path, save: dict) -> None:
    path = home / ".shellquest" / "save.json"
    path.write_text(json.dumps(save, indent=2))


def derive_state(save: dict) -> dict:
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
    env["TERM"] = "xterm-256color"

    pid, fd = pty.fork()
    if pid == 0:
        os.execvpe(SQ_BIN, [SQ_BIN, "arena"], env)

    chunks = []
    inputs = [f"{tier_choice}\r".encode(), b"y\r"] + [b"1\r"] * (max_rounds + 5)
    sent_idx = 0
    last_send = 0.0
    deadline = time.time() + timeout_s
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
    try:
        os.close(fd)
    except OSError:
        pass
    try:
        os.waitpid(pid, os.WNOHANG)
    except OSError:
        pass
    return ANSI_RE.sub("", "".join(chunks))


PLAYER_DMG_RE = re.compile(r"for (\d+) damage")
ENEMY_DMG_RE = re.compile(r"(\d+) damage")
ROUND_INTRO_RE = re.compile(r"⚔️\s*Round (\d+)")
ROUND_CLEARED_RE = re.compile(r"Round (\d+) cleared")
ENTRY_FEE_RE = re.compile(r"Entry fee: (\d+) gold")


def parse_arena_output(text: str, character_level: int, tier: str,
                       tier_index: int) -> dict:
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
    return DANGER_CWDS.get(danger, "/tmp")
