"""Tests for guard-live-save.py. Run: python3 -m unittest discover -s .claude/hooks"""

import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path

HOOK = Path(__file__).with_name("guard-live-save.py")
spec = importlib.util.spec_from_file_location("guard_live_save", HOOK)
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)

HOME = "/Users/player"


def verdict(command: str) -> str | None:
    decision = guard.decide(command, HOME)
    return decision[0] if decision else None


class DeniesUnsandboxedSq(unittest.TestCase):
    def test_game_binary_forms(self):
        for cmd in [
            "sq status",
            "sq tick --cmd 'git commit' --cwd . --exit-code 0",
            "./target/debug/sq journal",
            "target/release/sq init",
            "~/.cargo/bin/sq reset",
            "/Users/player/.cargo/bin/sq shop",
            "cargo run -- status",
            "cargo run --bin sq -- tick --cmd ls",
            "cargo +nightly r -q -- arena",
            "sq hook --shell zsh --install",
            "sq hook --shell bash --file /tmp/rc",
        ]:
            with self.subTest(cmd=cmd):
                self.assertEqual(verdict(cmd), "deny")

    def test_command_positions(self):
        for cmd in [
            "cargo build && ./target/debug/sq status",
            "cd ~ ; sq shop",
            "cargo build\nsq status",
            "true || sq status",
            "echo $(sq status)",
            'echo "result: $(sq status)"',
            "echo `sq journal`",
            "for i in 1 2 3; do sq tick --cmd ls; done",
            "if ! sq status; then echo no; fi",
            "time sq status",
            "timeout 5 sq status",
            "env -i PATH=/usr/bin sq status",
            "xargs sq status",
            "bash -c 'sq status'",
            "zsh -lc \"sq journal\"",
            "eval sq status",
            "hyperfine 'target/release/sq tick --cmd ls'",
            "script -q /dev/null ./target/debug/sq arena",
            "script -qc './target/debug/sq arena' /dev/null",
            "{ sq status; }",
            "(sq status)",
            "sq status 2>&1 | head",
        ]:
            with self.subTest(cmd=cmd):
                self.assertEqual(verdict(cmd), "deny")

    def test_home_pointing_at_real_home_is_not_an_override(self):
        for cmd in ["HOME=$HOME sq status", "HOME=~ sq status", "HOME=/Users/player sq status", "HOME=${HOME} sq status"]:
            with self.subTest(cmd=cmd):
                self.assertEqual(verdict(cmd), "deny")

    def test_substitution_before_override_still_denied(self):
        self.assertEqual(verdict('echo "$(sq status)"; export HOME=/tmp/x'), "deny")

    def test_astra_bypasses_2026_09_29(self):
        # Shapes an adversarial review (Codex gpt-6-astra) got past the first version.
        for cmd in [
            "(export HOME=/tmp/sb; true); sq tick --cmd ls",       # subshell export must not leak
            'HOME="${HOME:-/tmp/sb}" sq tick --cmd ls',            # unresolved expansion of HOME
            'HOME=/tmp/sb echo "$(sq tick --cmd ls)"',             # substitution runs before the prefix
            "/usr/bin/env sq tick --cmd ls",                       # absolute wrapper path
            "env -u HOME sq tick --cmd ls",                        # HOME unset → real home via getpwuid
            "env -i sq status",                                    # environment cleared
            "cargo --config net.offline=true run -- tick --cmd ls",  # cargo global option with a value
            "2>/dev/null target/debug/sq tick --cmd ls",           # leading redirection
            "2> /dev/null sq tick --cmd ls",
            ">/dev/null 2>&1 sq tick --cmd ls",
            "sudo -u root sq status",
            "bash -e -c 'sq status'",
            "bash <<'EOF'\nsq tick --cmd ls\nEOF",                  # executable shell heredoc
            "cat <<EOF\n$(sq status)\nEOF",                        # unquoted heredoc runs substitutions
            # Readiness review (2026-09-29): cargo forwards trailing args without `--`.
            "cargo run tick",
            "cargo run -q status",
            "cargo run --bin sq reset",
            "cargo --color never run --release journal",
            "SQ=./target/debug/sq; $SQ tick --cmd ls",               # binary through a variable
            'BIN=target/release/sq; "$BIN" status',
            "export SQ=target/debug/sq; ${SQ} journal",
            "bash <<< 'sq status'",                                  # here-string into a shell
            "timeout -k 1 5 sq status",
            "gtimeout 5 sq status",
            "watch -n1 sq status",
            "rtk proxy ./target/debug/sq status",
            'RUN="cargo run"; $RUN tick',                           # command stored in a variable
        ]:
            with self.subTest(cmd=cmd):
                self.assertEqual(verdict(cmd), "deny")

    def test_interpreters_that_launch_sq_ask(self):
        for cmd in [
            "python3 -c \"import subprocess; subprocess.run(['sq','tick','--cmd','ls'])\"",
            "python3 - <<'EOF'\nimport subprocess\nsubprocess.run(['./target/debug/sq', 'status'])\nEOF",
            "node -e \"require('child_process').execSync('target/release/sq status')\"",
            "python3 - <<EOF\nimport subprocess\nsubprocess.run(['sq', 'status'])\nEOF",
        ]:
            with self.subTest(cmd=cmd):
                self.assertEqual(verdict(cmd), "ask")


class AllowsSafeRuns(unittest.TestCase):
    def test_sandbox_and_home_overrides(self):
        for cmd in [
            "dev-tools/sq-sandbox status",
            "./dev-tools/sq-sandbox tick 'git commit' -n 5",
            'T=$(mktemp -d); HOME="$T" ./target/debug/sq status; rm -rf "$T"',
            "HOME=/tmp/sandbox sq status",
            "env HOME=/tmp/sandbox target/release/sq journal",
            "export HOME=/tmp/sandbox; sq status && sq journal",
            "HOME=/tmp/sb; sq status",
            "HOME=/tmp/sb bash -c 'sq status'",
            "HOME=/tmp/sb hyperfine 'target/release/sq tick --cmd ls'",
            'HOME="$T" SQ_NO_PACING=1 script -q /dev/null ./target/debug/sq arena',
            "script -q /tmp/typescript ls -la",
            'export HOME=/tmp/x; echo "$(sq status)"',
            "env -i HOME=/tmp/sb sq status",
            "export HOME=/tmp/sb; (sq status)",                     # subshell inherits the export
            "python3 - <<'EOF'\nimport subprocess\nsubprocess.run(['sq','status'], env={'HOME': '/tmp/sb'})\nEOF",
            "python3 -c \"import os,subprocess; os.environ['HOME']='/tmp/sb'; subprocess.run(['sq','status'])\"",
        ]:
            with self.subTest(cmd=cmd):
                self.assertIsNone(verdict(cmd))

    def test_save_free_subcommands(self):
        for cmd in [
            "sq --help",
            "sq -V",
            "sq",
            "sq help arena",
            "sq items --json | jq length",
            "cargo run -q -- bestiary --json",
            "sq hook --shell zsh",
            "cargo run",
            "cargo run --release",
            "cargo run -- --version",
            "cargo run -q items --json",
            "cargo build --bin sq",
            "cargo test -- run",
        ]:
            with self.subTest(cmd=cmd):
                self.assertIsNone(verdict(cmd))

    def test_mentions_and_other_tools(self):
        for cmd in [
            "cargo test",
            "cargo build --bin sq",
            "cargo clippy --all-targets",
            'grep -rn "sq tick" README.md',
            "rg 'sq status' src/",
            "git commit -m 'fix sq status output'",
            "ls target/debug/sq",
            "file target/release/sq",
            "which sq",
            "echo 'run sq status'",
            'grep -rn "\\.shellquest" src/',
            "cat > notes.md <<'EOF'\nsq status\nrm ~/.shellquest/save.json\nEOF",
            "cat <<< 'sq status'",
            "rg '$(sq status)' README.md",                          # single-quoted: no substitution
            "git commit -m 'docs: describe `sq status`'",
            "sq status --help",
            "cargo run -- tick --help",
            "cat > notes.md <<'END-MD'\nsq status\nEND-MD",           # hyphenated heredoc delimiter
            "command -v sq",
            "python3 -m unittest discover -s .claude/hooks",
            "python3 - <<'EOF'\nprint('sq status is a command')\nEOF",
        ]:
            with self.subTest(cmd=cmd):
                self.assertIsNone(verdict(cmd))

    def test_unparseable_commands_fall_through(self):
        self.assertIsNone(verdict("echo 'unterminated"))


class AsksBeforeTouchingRealSave(unittest.TestCase):
    def test_references(self):
        for cmd in [
            "cat ~/.shellquest/save.json",
            "rm -rf ~/.shellquest/the_void",
            "python3 -c \"import json; json.load(open('/Users/player/.shellquest/save.json'))\"",
            'jq . "$HOME/.shellquest/save.json"',
            "cp ${HOME}/.shellquest/save.json /tmp/backup.json",
            "HOME=/tmp/x ls /Users/player/.shellquest",
        ]:
            with self.subTest(cmd=cmd):
                self.assertEqual(verdict(cmd), "ask")

    def test_overridden_home_paths_are_the_sandbox(self):
        self.assertIsNone(verdict("export HOME=/tmp/sb; cat ~/.shellquest/save.json"))

    def test_override_after_the_mention_does_not_count(self):
        self.assertEqual(verdict('cat "$HOME/.shellquest/save.json"; export HOME=/tmp/sb'), "ask")
        self.assertEqual(verdict("HOME=/tmp/sb cat ~/.shellquest/save.json"), "ask")  # ~ expands before the prefix


class HookProtocol(unittest.TestCase):
    def run_hook(self, payload: dict) -> str:
        return subprocess.run(
            [sys.executable, str(HOOK)], input=json.dumps(payload), capture_output=True, text=True, check=True
        ).stdout

    def test_deny_json_shape(self):
        out = json.loads(self.run_hook({"tool_name": "Bash", "tool_input": {"command": "sq status"}}))
        self.assertEqual(out["hookSpecificOutput"]["hookEventName"], "PreToolUse")
        self.assertEqual(out["hookSpecificOutput"]["permissionDecision"], "deny")
        self.assertIn("dev-tools/sq-sandbox", out["hookSpecificOutput"]["permissionDecisionReason"])

    def test_silent_when_fine_or_not_bash_or_garbage(self):
        self.assertEqual(self.run_hook({"tool_name": "Bash", "tool_input": {"command": "cargo test"}}), "")
        self.assertEqual(self.run_hook({"tool_name": "Read", "tool_input": {"file_path": "x"}}), "")
        out = subprocess.run([sys.executable, str(HOOK)], input="not json", capture_output=True, text=True)
        self.assertEqual((out.returncode, out.stdout), (0, ""))


if __name__ == "__main__":
    unittest.main()
