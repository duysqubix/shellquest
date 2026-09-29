"""End-to-end tests for the shell hook (shellqeuest-dsz.1).

The hook is installed with the real binary (`sq hook --shell X --install`) into a
throwaway HOME, then a real interactive shell is driven through a pseudo-terminal. A
fake `sq` on PATH records every tick and hands `sq hook ...` to the real binary, so the
rc file exercises the actual install path. The notice test runs the real binary against
a throwaway character. Nothing touches the developer's rc files or save. Shells that
aren't installed are skipped.

Run: python3 -m unittest discover -s tests/shell_hooks   (needs target/debug/sq)
"""

import datetime
import json
import os
import pty
import re
import select
import shutil
import stat
import subprocess
import tempfile
import time
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SQ = REPO / "target" / "debug" / "sq"
PROMPT = b"P>"
ESCAPES = re.compile(rb"\x1b\[[0-9;?]*[A-Za-z]|\x1b[()][0-9A-Za-z]|[\x07\r]")

# The hooks sq 1.28 printed for `sq hook --shell X >> rc` (see hook::legacy_hooks).
LEGACY = {
    "zsh": """
# shellquest (sq) — passive terminal RPG hook
__sq_hook() {
    local exit_code=$?
    local cmd=$(fc -ln -1)
    local first=${cmd[(w)1]}
    [[ "$first" == "sq" ]] && return
    sq tick --cmd "$cmd" --cwd "$PWD" --exit-code "$exit_code"
}
precmd_functions+=(__sq_hook)
""",
    "bash": """
# shellquest (sq) — passive terminal RPG hook
__sq_hook() {
    local exit_code=$?
    local cmd=$(HISTTIMEFORMAT= history 1 | sed 's/^ *[0-9]* *//')
    local first=$(printf '%s' "$cmd" | awk '{print $1}')
    [ "$first" = "sq" ] && return
    sq tick --cmd "$cmd" --cwd "$PWD" --exit-code "$exit_code"
}
PROMPT_COMMAND="__sq_hook;$PROMPT_COMMAND"
""",
}


def bash_version(path):
    out = subprocess.run([path, "-c", "echo ${BASH_VERSINFO[0]}.${BASH_VERSINFO[1]}"], capture_output=True, text=True)
    major, minor = out.stdout.strip().split(".")
    return int(major), int(minor)


def find_shells():
    shells = {}
    zsh = shutil.which("zsh") or ("/bin/zsh" if Path("/bin/zsh").exists() else None)
    if zsh:
        shells["zsh"] = zsh
    for candidate in ["/opt/homebrew/bin/bash", "/usr/local/bin/bash", "/usr/bin/bash", "/bin/bash", shutil.which("bash")]:
        if candidate and Path(candidate).exists():
            key = "bash-modern" if bash_version(candidate) >= (4, 4) else "bash-3"
            shells.setdefault(key, candidate)
    return shells


SHELLS = find_shells()


def parse_tick(args):
    """(cmd, exit code, hook version) from `sq tick` arguments in either spelling."""
    found = {}
    i = 0
    while i < len(args):
        name, eq, value = args[i].partition("=")
        if not eq and i + 1 < len(args):  # sq <= 1.28 hooks: `--cmd "$cmd"`
            value, i = args[i + 1], i + 1
        found[name] = value
        i += 1
    return found.get("--cmd"), found.get("--exit-code"), found.get("--hook")


class Shell:
    """A throwaway HOME with a fake `sq`, plus helpers to install the hook and drive a shell."""

    def __init__(self, kind, settings="", history=None):
        self.kind, self.path = kind, SHELLS[kind]
        self.shell = "zsh" if kind == "zsh" else "bash"
        self.home = Path(tempfile.mkdtemp(prefix="sq-hook-"))
        (self.home / "bin").mkdir()
        self.log = self.home / "ticks.log"
        self.log.touch()
        fake = self.home / "bin" / "sq"
        fake.write_text(
            "#!/bin/sh\n"
            f'if [ "$1" = hook ]; then exec "{SQ}" "$@"; fi\n'
            '[ "$1" = tick ] || exit 0\n'
            "shift\n"
            # One record per tick: arguments end in \037, the record in \036, so
            # commands with newlines in them survive.
            '{ for a in "$@"; do printf "%s\\037" "$a"; done; printf "\\036"; } >> "$SQ_TICK_LOG"\n'
        )
        fake.chmod(0o755)
        self.rc = self.home / (".zshrc" if self.shell == "zsh" else ".bashrc")
        self.hist = self.home / ".hist"
        if history:
            self.hist.write_text(history)
        base = f"HISTFILE={self.hist}\nHISTSIZE=100\nSAVEHIST=100\nPS1='P> '\nPROMPT='P> '\nPS2='C> '\n"
        if self.shell == "zsh":
            base += "setopt HIST_IGNORE_DUPS\nunset zle_bracketed_paste\n"
            (self.home / ".zshenv").write_text("setopt NO_GLOBAL_RCS\n")  # skip /etc/zshrc
        else:
            base += "HISTCONTROL=ignoredups\n"
        self.rc.write_text(base + settings)
        self.env = {"HOME": str(self.home), "PATH": f"{self.home}/bin:/usr/bin:/bin", "TERM": "dumb", "SQ_TICK_LOG": str(self.log)}
        if self.shell == "zsh":
            self.env["ZDOTDIR"] = str(self.home)
        self.transcript = b""

    def install(self, file=None):
        argv = [str(SQ), "hook", "--shell", self.shell, "--install"] + (["--file", str(file)] if file else [])
        return subprocess.run(argv, env={"HOME": str(self.home), "PATH": "/usr/bin:/bin"}, capture_output=True, text=True)

    def append_rc(self, text):
        with self.rc.open("a") as f:
            f.write(text)

    def backups(self):
        return sorted(p.name for p in self.home.glob("*.sq-backup-*"))

    def run(self, keys, rcfile=None):
        """Type each key (text, or a callable to run between prompts), then exit."""
        if self.shell == "zsh":
            argv = [self.path, "-i"]
        else:
            argv = [self.path, "--noprofile", "--rcfile", str(rcfile or self.rc), "-i"]
        pid, fd = pty.fork()
        if pid == 0:
            os.chdir(self.home)
            os.execve(argv[0], argv, self.env)
        try:
            self._wait_prompt(fd)
            for key in keys:
                if callable(key):
                    key()
                    continue
                os.write(fd, key.encode())
                self._wait_prompt(fd)
            os.write(fd, b"exit\n")
        finally:
            deadline = time.time() + 5
            while time.time() < deadline:
                done, _ = os.waitpid(pid, os.WNOHANG)
                if done:
                    break
                self._read(fd, 0.05)
            else:
                os.kill(pid, 9)
                os.waitpid(pid, 0)
            os.close(fd)
        return self.ticks()

    def _read(self, fd, timeout):
        ready, _, _ = select.select([fd], [], [], timeout)
        if not ready:
            return b""
        try:
            data = os.read(fd, 4096)
        except OSError:
            return b""
        self.transcript += data
        return data

    def _wait_prompt(self, fd, timeout=10.0):
        buf, deadline = b"", time.time() + timeout
        while time.time() < deadline:
            buf += self._read(fd, 0.05)
            if ESCAPES.sub(b"", buf).rstrip(b" ").endswith(PROMPT):
                time.sleep(0.05)  # let anything still in flight settle
                return
        raise AssertionError(f"no prompt from {self.kind}; got {buf[-300:]!r}")

    def ticks(self):
        records = self.log.read_text().split("\x1e")[:-1]
        return [parse_tick(r.split("\x1f")[:-1]) for r in records]

    def close(self):
        shutil.rmtree(self.home, ignore_errors=True)


def ticked(*pairs):
    """Ticks from the current hook, which passes --hook=2."""
    return [(cmd, code, "2") for cmd, code in pairs]


class ShellTest(unittest.TestCase):
    def shell(self, kind, **kw):
        if kind not in SHELLS:
            self.skipTest(f"{kind} not installed")
        sh = Shell(kind, **kw)
        self.addCleanup(sh.close)
        return sh

    def kinds(self, *wanted):
        """The installed shells among `wanted` (all by default); skips if none is."""
        found = [k for k in (wanted or ("zsh", "bash-modern", "bash-3")) if k in SHELLS]
        if not found:
            self.skipTest(f"none of {wanted} installed")
        return found

    def installed(self, kind, **kw):
        sh = self.shell(kind, **kw)
        result = sh.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        return sh


@unittest.skipUnless(SQ.exists(), "build target/debug/sq first (cargo build)")
class TicksOncePerCommand(ShellTest):
    def check_basics(self, kind):
        sh = self.installed(kind)
        again = sh.install()
        self.assertIn("already installed", again.stdout, "installing twice is a no-op")
        self.assertEqual(sh.rc.read_text().count(f"# >>> shellquest hook ({sh.shell}) >>>"), 1)

        ticks = sh.run(["true\n", "\n", "false\n", "\n", "abc\x03", "echo a\n", "echo a\n", "sq status\n", "-x 2>/dev/null\n"])
        # zsh sees each command; bash can't tell a repeat dropped by ignoredups from an empty Enter.
        repeats = 2 if kind == "zsh" else 1
        want = [("true", "0"), ("false", "1")] + [("echo a", "0")] * repeats + [("-x 2>/dev/null", "127")]
        self.assertEqual(ticks, ticked(*want))

    def test_zsh(self):
        self.check_basics("zsh")

    def test_bash_modern(self):
        self.check_basics("bash-modern")

    def test_bash_3(self):
        self.check_basics("bash-3")

    def test_a_new_shell_does_not_replay_history(self):
        for kind in self.kinds():
            with self.subTest(kind=kind):
                sh = self.installed(kind, history="git push origin main\n")
                self.assertEqual(sh.run(["\n", "\n"]), [])

    def test_hidden_and_comment_lines_do_not_count(self):
        settings = {"zsh": ["setopt HIST_IGNORE_SPACE\n", ""], "bash": ["HISTCONTROL=ignoreboth\n", "HISTCONTROL=\n"]}
        for kind in self.kinds():
            for extra in settings["zsh" if kind == "zsh" else "bash"]:
                with self.subTest(kind=kind, settings=extra.strip() or "defaults"):
                    sh = self.installed(kind, settings=extra)
                    ticks = sh.run(["true\n", " false\n", "\n", "# a note\n", "\n", "echo b\n"])
                    self.assertEqual(ticks, ticked(("true", "0"), ("echo b", "0")))

    def test_a_multi_line_command_ticks_once_with_its_text(self):
        for kind in self.kinds():
            with self.subTest(kind=kind):
                sh = self.installed(kind)
                self.assertEqual(sh.run(["echo 'a\nb'\n", "\n"]), ticked(("echo 'a\nb'", "0")))

    def test_bash_counts_a_command_that_erasedups_moved(self):
        for kind in self.kinds("bash-modern", "bash-3"):
            with self.subTest(kind=kind):
                sh = self.installed(kind, settings="HISTCONTROL=erasedups\n")
                ticks = sh.run(["echo a\n", "echo b\n", "echo a\n", "\n"])
                self.assertEqual(ticks, ticked(("echo a", "0"), ("echo b", "0"), ("echo a", "0")))

    def test_bash_ignores_history_read_from_other_shells(self):
        # Needs PS0 (bash >= 4.4); older bash may count another shell's command here.
        sh = self.installed(self.kinds("bash-modern")[0], settings="PROMPT_COMMAND='history -a; history -n'\n")

        def other_shell():
            with sh.hist.open("a") as f:
                f.write("git push origin main\n")

        ticks = sh.run(["true\n", other_shell, "\n", "\n", "echo c\n"])
        self.assertEqual(ticks, ticked(("true", "0"), ("echo c", "0")))

    def test_bash_does_not_count_imported_history_after_a_hidden_command(self):
        for kind in self.kinds("bash-modern", "bash-3"):
            with self.subTest(kind=kind):
                sh = self.installed(kind, settings="HISTCONTROL=ignorespace\nPROMPT_COMMAND='history -a; history -n'\n")

                def other_shell():
                    with sh.hist.open("a") as f:
                        f.write("git push origin main\n")

                ticks = sh.run(["true\n", other_shell, "\n", " false\n", "echo c\n"])
                self.assertEqual(ticks, ticked(("true", "0"), ("echo c", "0")))


@unittest.skipUnless(SQ.exists(), "build target/debug/sq first (cargo build)")
class PlaysWellWithTheShell(ShellTest):
    def test_later_prompt_hooks_still_see_the_exit_status(self):
        user_hook = '__user() { echo "status=$?" >> "$HOME/status.log"; }\n'
        cases = [("zsh", "", user_hook + "precmd_functions+=(__user)\n")]
        cases += [(k, user_hook + "PROMPT_COMMAND=__user\n", "") for k in ("bash-modern", "bash-3")]
        if "bash-modern" in SHELLS and bash_version(SHELLS["bash-modern"]) >= (5, 1):
            cases.append(("bash-modern", user_hook + "PROMPT_COMMAND=(__user)\n", ""))
        for kind, before, after in cases:
            if kind not in SHELLS:
                continue
            with self.subTest(kind=kind, rc=(before + after).splitlines()[-1]):
                sh = self.installed(kind, settings=before)
                sh.append_rc(after)
                self.assertEqual(sh.run(["false\n", "true\n", "(exit 3)\n"]), ticked(("false", "1"), ("true", "0"), ("(exit 3)", "3")))
                statuses = (sh.home / "status.log").read_text().split()
                self.assertEqual(statuses, ["status=0", "status=1", "status=0", "status=3"])

    def test_zsh_options_do_not_break_the_hook(self):
        options = (
            "setopt KSH_ARRAYS NO_UNSET WARN_CREATE_GLOBAL WARN_NESTED_VAR EXTENDED_GLOB SH_WORD_SPLIT IGNORE_BRACES\n"
            '__other() { echo x >> "$HOME/other.log"; }\nprecmd_functions=(__other)\n'
        )
        sh = self.installed(self.kinds("zsh")[0], settings=options)
        self.assertEqual(sh.run(["true\n", "echo a  b\n", "\n"]), ticked(("true", "0"), ("echo a  b", "0")))
        self.assertEqual(len((sh.home / "other.log").read_text().split()), 4, "the player's own precmd still runs")
        for complaint in [b"created globally", b"set in enclosing scope", b"parameter not set", b"not found", b"bad "]:
            self.assertNotIn(complaint, sh.transcript)

    def test_bash_options_do_not_break_the_hook(self):
        for kind in self.kinds("bash-modern", "bash-3"):
            with self.subTest(kind=kind):
                sh = self.installed(kind, settings="set -u\nshopt -s nocasematch extglob\nHISTTIMEFORMAT='%F %T '\n")
                self.assertEqual(sh.run(["true\n", "SQ x\n", "\n"]), ticked(("true", "0")))
                self.assertNotIn(b"unbound variable", sh.transcript)

    def test_bash_leaves_the_players_regex_captures_alone(self):
        for kind in self.kinds("bash-modern", "bash-3"):
            with self.subTest(kind=kind):
                sh = self.installed(kind)
                sh.run(["[[ abc =~ (b) ]]\n", 'echo "m=${BASH_REMATCH[1]}" > "$HOME/m.log"\n'])
                self.assertEqual((sh.home / "m.log").read_text().strip(), "m=b")

    def test_bash_without_promptvars_gets_no_ps0_marker(self):
        sh = self.installed(self.kinds("bash-modern")[0], settings="shopt -u promptvars\n")
        self.assertEqual(sh.run(["true\n", "\n", "false\n"]), ticked(("true", "0"), ("false", "1")))
        self.assertNotIn(b"__sq_ran", sh.transcript, "PS0 would print the marker literally")

    def test_zsh_retires_an_old_hook_loaded_in_the_same_shell(self):
        # Say a copy of the old hook lives in a file --install doesn't scan. Either
        # way round, it must not replay history at the first prompt or tick again.
        kind = self.kinds("zsh")[0]
        for order in ("old hook first", "old hook last"):
            with self.subTest(order):
                if order == "old hook first":
                    sh = self.shell(kind, settings=LEGACY["zsh"], history="git push origin main\n")
                    sh.append_rc('eval "$(sq hook --shell zsh)"\n')
                else:
                    sh = self.installed(kind, history="git push origin main\n")
                    sh.append_rc(LEGACY["zsh"])
                self.assertEqual(sh.run(["true\n", "\n"]), ticked(("true", "0")))


@unittest.skipUnless(SQ.exists(), "build target/debug/sq first (cargo build)")
class UpgradingOldHooks(ShellTest):
    def test_an_old_hook_is_replaced_in_place_with_a_backup(self):
        for kind in self.kinds("zsh", "bash-modern"):
            with self.subTest(kind=kind):
                shell = "zsh" if kind == "zsh" else "bash"
                sh = self.shell(kind, settings="alias ll='ls -l'\n" + LEGACY[shell] + "export AFTER=1\n")
                result = sh.install()
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("Upgraded", result.stdout)
                rc = sh.rc.read_text()
                self.assertNotIn("__sq_hook() {", rc)
                self.assertIn(f'eval "$(sq hook --shell {shell})"', rc)
                self.assertLess(rc.index("alias ll"), rc.index(f"# >>> shellquest hook ({shell}) >>>"))
                self.assertLess(rc.index(f"# <<< shellquest hook ({shell}) <<<"), rc.index("export AFTER=1"))
                self.assertEqual(len(sh.backups()), 1)
                self.assertIn("__sq_hook() {", (sh.home / sh.backups()[0]).read_text())
                self.assertEqual(sh.run(["true\n", "\n", "\n"]), ticked(("true", "0")))

    def test_every_rc_file_with_an_old_hook_is_upgraded(self):
        for kind in self.kinds("zsh", "bash-modern"):
            with self.subTest(kind=kind):
                if kind == "zsh":
                    sh = self.shell(kind, settings=LEGACY["zsh"] + 'source "$HOME/.zshrc_local"\n')
                    other = sh.home / ".zshrc_local"
                    other.write_text("# local settings\n" + LEGACY["zsh"])
                    rcfile = None
                else:  # a login shell reads .bash_profile, which also loads .bashrc
                    sh = self.shell(kind, settings=LEGACY["bash"])
                    other = rcfile = sh.home / ".bash_profile"
                    other.write_text(LEGACY["bash"] + '. "$HOME/.bashrc"\n')
                result = sh.install()
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout.count("Upgraded"), 2)
                for path in (sh.rc, other):
                    text = path.read_text()
                    self.assertEqual(text.count(f"# >>> shellquest hook ({sh.shell}) >>>"), 1, path.name)
                    self.assertNotIn("__sq_hook() {", text, path.name)
                self.assertEqual(len(sh.backups()), 2)
                self.assertEqual(sh.run(["true\n", "\n"], rcfile=rcfile), ticked(("true", "0")), "the hook loads twice but ticks once")

    def test_re_sourcing_an_upgraded_rc_file_leaves_one_hook(self):
        for kind in self.kinds():
            with self.subTest(kind=kind):
                shell = "zsh" if kind == "zsh" else "bash"
                sh = self.shell(kind, settings=LEGACY[shell], history="ls\n")
                ticks = sh.run(["true\n", f"sq hook --shell {shell} --install\n", f"source {sh.rc}\n", "echo c\n", "\n"])
                first_new = next(i for i, t in enumerate(ticks) if t[2] is not None)
                self.assertIn(("true", "0", None), ticks[:first_new], "the old hook was live before the upgrade")
                self.assertEqual(ticks[first_new:], ticked(("echo c", "0")))

    def test_an_explicit_file_gets_the_hook_even_when_another_file_has_one(self):
        sh = self.installed(self.kinds("zsh")[0])
        other = sh.home / ".config" / "zsh" / ".zshrc"
        result = sh.install(file=other)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(other.read_text().count("# >>> shellquest hook (zsh) >>>"), 1)
        self.assertEqual(sh.install(file=other).stdout.count("Hook installed"), 0, "and only once")

    def test_a_symlinked_rc_file_is_upgraded_through_the_link(self):
        sh = self.shell(self.kinds("zsh")[0])
        real = sh.home / "dotfiles" / "zshrc"
        real.parent.mkdir()
        real.write_text(sh.rc.read_text() + LEGACY["zsh"])
        real.chmod(0o600)
        sh.rc.unlink()
        sh.rc.symlink_to(real)
        result = sh.install(file=sh.rc)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.count("Upgraded"), 1, "the file is handled once under both names")
        self.assertTrue(sh.rc.is_symlink())
        self.assertIn('eval "$(sq hook --shell zsh)"', real.read_text())
        self.assertEqual(stat.S_IMODE(real.stat().st_mode), 0o600)
        self.assertEqual(len(sh.backups()), 1)
        self.assertEqual([p.name for p in real.parent.iterdir()], ["zshrc"], "no temp file is left behind")
        self.assertEqual(sh.run(["true\n"]), ticked(("true", "0")))

    def test_a_file_that_is_not_utf8_is_appended_to_but_never_rewritten(self):
        sh = self.shell(self.kinds("zsh")[0])
        latin1 = b"export NAME='caf\xe9'\n"
        sh.rc.write_bytes(latin1)
        result = sh.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(sh.rc.read_bytes().startswith(latin1))
        self.assertIn(b'eval "$(sq hook --shell zsh)"', sh.rc.read_bytes())

    def test_files_it_cannot_safely_edit_are_left_alone(self):
        edited = LEGACY["zsh"].replace("fc -ln -1", "fc -ln -1 | head -1")
        cases = {
            "hand-edited hook": (edited.encode(), "doesn't recognize"),
            "not UTF-8": (b"export NAME='caf\xe9'\n" + LEGACY["zsh"].encode(), "UTF-8"),
        }
        for name, (content, reason) in cases.items():
            with self.subTest(name):
                sh = self.shell(self.kinds("zsh")[0])
                sh.rc.write_bytes(content)
                result = sh.install()
                self.assertEqual(result.returncode, 1)
                self.assertIn(reason, result.stderr)
                self.assertIn('eval "$(sq hook --shell zsh)"', result.stderr, "says what to add by hand")
                self.assertEqual(sh.rc.read_bytes(), content)
                self.assertEqual(sh.backups(), [])


@unittest.skipUnless(SQ.exists(), "build target/debug/sq first (cargo build)")
class OldHookNotice(unittest.TestCase):
    """`sq tick` called by an old hook (no --hook) tells the player once to upgrade."""

    def setUp(self):
        self.home = Path(tempfile.mkdtemp(prefix="sq-notice-"))
        self.addCleanup(shutil.rmtree, self.home, True)
        self.env = {"HOME": str(self.home), "PATH": "/usr/bin:/bin", "TERM": "dumb", "SHELL": "/bin/zsh"}
        made = subprocess.run([str(SQ), "init"], input="Tester\n1\n1\nn\n", env=self.env, capture_output=True, text=True)
        self.save = self.home / ".shellquest" / "save.json"
        self.assertTrue(self.save.exists(), made.stdout + made.stderr)
        # Pre-seed the daily crates.io check so ticks stay offline.
        version = subprocess.run([str(SQ), "--version"], capture_output=True, text=True).stdout.split()[-1]
        data = json.loads(self.save.read_text())
        data["last_version_check"] = datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z")
        data["latest_version"] = version
        self.save.write_text(json.dumps(data))

    def tick_in_terminal(self, *extra):
        argv = [str(SQ), "tick", "--cmd=ls", f"--cwd={self.home}", "--exit-code=0", *extra]
        pid, fd = pty.fork()
        if pid == 0:
            os.execve(argv[0], argv, self.env)
        out, deadline = b"", time.time() + 10
        while time.time() < deadline:
            ready, _, _ = select.select([fd], [], [], 0.1)
            try:
                chunk = os.read(fd, 4096) if ready else b""
            except OSError:
                break
            if ready and not chunk:
                break
            out += chunk
        os.waitpid(pid, 0)
        os.close(fd)
        return out.decode(errors="replace")

    def notice_version(self):
        return json.loads(self.save.read_text()).get("hook_notice_version", 0)

    def test_told_once_and_only_where_it_can_be_seen(self):
        self.assertNotIn("out of date", self.tick_in_terminal("--hook=2"), "the current hook")
        piped = subprocess.run([str(SQ), "tick", "--cmd=ls", f"--cwd={self.home}", "--exit-code=0"], env=self.env, capture_output=True, text=True)
        self.assertNotIn("out of date", piped.stderr, "nobody would see it")
        self.assertEqual(self.notice_version(), 0)

        first = self.tick_in_terminal()
        self.assertIn("out of date", first)
        self.assertIn("sq hook --shell zsh --install", first)
        self.assertEqual(self.notice_version(), 2)
        self.assertNotIn("out of date", self.tick_in_terminal())


if __name__ == "__main__":
    unittest.main()
