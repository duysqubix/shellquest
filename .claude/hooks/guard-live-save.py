#!/usr/bin/env python3
"""PreToolUse guard for Bash: keep Claude away from the developer's live save.

The developer plays shellquest: ~/.shellquest/save.json is a real character and
their shell hook runs `sq tick` before every prompt. This hook:

* denies running the game binary (`sq`, `target/*/sq`, `~/.cargo/bin/sq`,
  `cargo run`) unless HOME is redirected for that command or the subcommand
  never touches the save (help, items, bestiary, hook without --install/--file,
  --help, --version). The reason points Claude at dev-tools/sq-sandbox.
* asks the user before interpreter scripts that appear to launch sq without
  setting HOME, and before commands that name the real save directory.

It is a guard rail for honest mistakes, not a sandbox: it reads command text
only and approximates shell semantics (quoting, subshell scope, expansion
order, heredocs). A crash inside the guard never blocks the command.
Tests: python3 -m unittest discover -s .claude/hooks
"""

from __future__ import annotations

import json
import os
import pwd
import re
import shlex
import sys

PUNCTUATION = ";&|()\n"
SAVE_FREE_SUBCOMMANDS = {"help", "items", "bestiary"}
INFO_FLAGS = {"-h", "--help", "-V", "--version"}
KEYWORDS = {"then", "do", "else", "elif", "if", "while", "until", "time", "exec", "{", "}", "!"}
WRAPPERS = {"command", "builtin", "noglob", "nohup", "nice", "stdbuf", "sudo", "doas", "xargs", "timeout", "gtimeout", "env", "rtk"}
# Wrapper options that consume the following word.
OPTION_ARGS = {
    "env": {"-u", "--unset", "-C", "--chdir", "-S", "--split-string", "-P"},
    "sudo": {"-u", "-g", "-C", "-D", "-h", "-p", "-r", "-t", "-U", "-T"},
    "doas": {"-u", "-C"},
    "timeout": {"-s", "--signal", "-k", "--kill-after"},
    "gtimeout": {"-s", "--signal", "-k", "--kill-after"},
    "nice": {"-n", "--adjustment"},
    "xargs": {"-I", "-n", "-L", "-P", "-s", "-d", "-E", "-a"},
}
SHELLS = {"sh", "bash", "zsh", "dash", "ksh", "fish"}
# cargo options that take a value (global or `run`); the first other positional after
# `run` starts the binary's arguments, even without `--` (cargo's trailing var arg).
CARGO_VALUE_FLAGS = {
    "--bin", "-p", "--package", "--example", "-F", "--features", "--target", "--profile",
    "--manifest-path", "--target-dir", "--color", "--config", "-j", "--jobs", "-Z",
    "--message-format", "--lockfile-path", "-C",
}
WATCH_VALUE_FLAGS = {"-n", "--interval", "-d", "--differences"}
VARIABLE_REF = re.compile(r"^\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$")
INTERPRETERS = {"python", "python3", "node", "ruby", "perl", "deno", "bun"}
ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
REDIRECT = re.compile(r"^(?:\d*|ALL)(?:>>?|<)(.*)$")
HEREDOC = re.compile(r"(?<!<)<<(?!<)-?\s*(['\"]?)([^\s'\"<>;&|()]+)\1")
SUBSTITUTION = re.compile(r"\$\(([^()]*)\)|`([^`]*)`")
REFERS_TO_HOME = re.compile(r"\$\{?HOME\b|^~")
# In an interpreter script: a quoted "sq" argv0, a target/*/sq path, or `cargo run`.
INTERPRETER_RUNS_SQ = re.compile(r"""['"](?:[^'"\s]*/)?sq['"]|target/(?:debug|release)/sq\b|\bcargo\s+run\b""")
SETS_HOME = re.compile(r"""\bHOME\b\s*['"]?\s*[:=\]]""")

SANDBOX_HINT = (
    "Running sq with the real HOME would read/modify the developer's live character "
    "(~/.shellquest/save.json). Use the sandbox instead: `dev-tools/sq-sandbox new` once, then "
    "`dev-tools/sq-sandbox <sq args>` (see the sq-qa skill). If the user explicitly wants their real "
    "character touched, ask them to run the command themselves with the `!` prefix."
)


def real_home() -> str:
    return os.path.realpath(pwd.getpwuid(os.getuid()).pw_dir)


def mask_single_quotes(text: str) -> str:
    """Blank out single-quoted spans (outside double quotes), keeping offsets."""
    out, state = [], None
    for ch in text:
        if state == "'":
            out.append("'" if ch == "'" else " ")
            if ch == "'":
                state = None
            continue
        if ch == "'" and state is None:
            state = "'"
        elif ch == '"':
            state = None if state == '"' else ('"' if state is None else state)
        out.append(ch)
    return "".join(out)


def split_heredocs(command: str) -> tuple[str, list[tuple[str, str, bool]]]:
    """Remove heredoc bodies. Returns (command without bodies, [(intro line, body, quoted)])."""
    kept: list[str] = []
    docs: list[tuple[str, str, bool]] = []
    waiting: list[tuple[str, bool, str]] = []  # (delimiter, quoted, intro line)
    body: list[str] = []
    for line in command.split("\n"):
        if waiting:
            if line.strip() == waiting[0][0]:
                delim, quoted, intro = waiting.pop(0)
                docs.append((intro, "\n".join(body), quoted))
                body = []
            else:
                body.append(line)
            continue
        kept.append(line)
        masked = mask_single_quotes(line)
        for m in HEREDOC.finditer(line):
            if masked[m.start()] == "<":  # not inside single quotes
                waiting.append((m.group(2), bool(m.group(1)), line))
    if waiting:  # unterminated heredoc: treat the rest as its body
        docs.append((waiting[0][2], "\n".join(body), waiting[0][1]))
    return "\n".join(kept), docs


def normalize_redirections(command: str) -> str:
    """Make `2>&1`, `>&2` and `&>file` single words so `&` isn't read as a separator."""
    command = re.sub(r"(\d*)([<>])&(\d+|-)", r"\1\2FD\3", command)
    return re.sub(r"&>(>?)", r"ALL>\1", command)


def tokenize(command: str) -> list[str]:
    lexer = shlex.shlex(command.replace("`", " ; "), posix=True, punctuation_chars=PUNCTUATION)
    lexer.whitespace = " \t\r"
    lexer.whitespace_split = True
    lexer.commenters = ""
    return list(lexer)


def is_separator(token: str) -> bool:
    return bool(token) and all(c in PUNCTUATION for c in token)


def overrides_home(value: str, home: str) -> bool:
    """True when HOME=value provably points somewhere other than the real home."""
    if not value or REFERS_TO_HOME.search(value):
        return False
    return os.path.realpath(os.path.expanduser(value)) != home


def is_sq_path(word: str) -> bool:
    return os.path.basename(word) == "sq"


def sq_args(words: list[str]) -> list[str] | None:
    """If this simple command runs the game binary, return its arguments."""
    if not words:
        return None
    if is_sq_path(words[0]):
        return words[1:]
    if os.path.basename(words[0]) == "cargo":
        rest = [w for w in words[1:] if not w.startswith("+")]
        i = 0
        while i < len(rest):  # find the subcommand, skipping global options
            word = rest[i]
            if word in CARGO_VALUE_FLAGS:
                i += 2
            elif word.startswith("-"):
                i += 1
            elif word in ("run", "r"):
                break
            else:
                return None  # another cargo subcommand (build, test, ...)
        else:
            return None
        i += 1
        while i < len(rest):  # `run`'s own options, then the binary's arguments
            word = rest[i]
            if word == "--":
                return rest[i + 1 :]
            if word in CARGO_VALUE_FLAGS:
                i += 2
            elif word.startswith("-"):
                i += 1
            else:
                return rest[i:]
        return []
    return None


def touches_save(args: list[str]) -> bool:
    if set(args) & INFO_FLAGS:
        return False  # clap prints help/version and exits before any save access
    positional = [a for a in args if not a.startswith("-")]
    if not positional:
        return bool(args)
    sub = positional[0]
    if sub in SAVE_FREE_SUBCOMMANDS:
        return False
    if sub == "hook":
        return any(a == "--install" or a.startswith("--file") for a in args)
    return True


def inline_script(words: list[str], letters: str = "c") -> str | None:
    """The script of `bash -c SCRIPT` / `zsh -lc SCRIPT` (letters "c") or `node -e SCRIPT` (letters "ce")."""
    for j, word in enumerate(words[1:], start=1):
        if word.startswith("-") and not word.startswith("--") and any(ch in word[1:] for ch in letters):
            return words[j + 1] if j + 1 < len(words) else None
    return None


def interpreter_launches_sq(script: str) -> bool:
    return bool(INTERPRETER_RUNS_SQ.search(script)) and not SETS_HOME.search(script)


def strip_prefix(words: list[str], home: str) -> tuple[list[str], bool, bool]:
    """Drop assignments, keywords, wrappers and leading redirections.

    Returns (remaining words, HOME overridden for this command, HOME unset for this command).
    """
    override = unset = False
    while words:
        head = words[0]
        if REDIRECT.match(head):
            words.pop(0)
            if not REDIRECT.match(head).group(1) and words:
                words.pop(0)  # `2> file`: the target is the next word
            continue
        if ASSIGNMENT.match(head):
            words.pop(0)
            name, _, value = head.partition("=")
            if name == "HOME":
                override = overrides_home(value, home)
                unset = unset and not override
            continue
        if head in KEYWORDS:
            words.pop(0)
            continue
        base = os.path.basename(head)
        if base not in WRAPPERS:
            break
        words.pop(0)
        if base == "rtk":
            if words[:1] == ["proxy"]:
                words.pop(0)
            continue
        while words and words[0].startswith("-") and base in OPTION_ARGS | {"stdbuf": set()}:
            opt = words.pop(0)
            if base == "env" and opt in ("-i", "--ignore-environment", "-"):
                override, unset = False, True
            if opt in OPTION_ARGS.get(base, ()) and words:
                arg = words.pop(0)
                if base == "env" and opt in ("-u", "--unset") and arg == "HOME":
                    override, unset = False, True
        if base in ("timeout", "gtimeout") and words and not words[0].startswith("-"):
            words.pop(0)  # the duration
    return words, override, unset


def analyze(command: str, home: str, depth: int = 0, inherited: bool = False) -> tuple[list[str], list[str]]:
    """Return (unsafe sq invocations, interpreter scripts that seem to launch sq)."""
    hits: list[str] = []
    asks: list[str] = []
    if depth > 3:
        return hits, asks
    command, heredocs = split_heredocs(normalize_redirections(command))

    for intro, body, quoted in heredocs:
        words = []
        for token in tokenize(intro):
            if "<<" in token:
                break
            words = [] if is_separator(token) else words + [token]
        words, override, _ = strip_prefix(words, home)
        runner = os.path.basename(words[0]) if words else ""
        if runner in SHELLS:
            if not (override or inherited):
                h, a = analyze(body, home, depth + 1)
                hits, asks = hits + h, asks + a
        elif runner in INTERPRETERS:
            if not (override or inherited) and interpreter_launches_sq(body):
                asks.append(f"{runner} heredoc")
        elif not quoted:  # unquoted heredocs still run command substitutions
            for m in SUBSTITUTION.finditer(body):
                h, a = analyze(m.group(1) or m.group(2) or "", home, depth + 1, inherited)
                hits, asks = hits + h, asks + a

    # Substitutions inside double quotes survive tokenizing as one word. They run in
    # the current shell before any prefix assignment of their own command applies.
    masked = mask_single_quotes(command)
    for m in SUBSTITUTION.finditer(masked):
        if inherited or persistent_override_before(command, m.start(), home):
            continue
        group = 1 if m.group(1) is not None else 2
        h, a = analyze(command[m.start(group) : m.end(group)], home, depth + 1)
        hits, asks = hits + h, asks + a

    tokens = tokenize(command)
    state = inherited
    stack: list[bool] = []
    variables: dict[str, str] = {}  # NAME=value seen so far, for `$NAME args` in command position
    i = 0
    while i < len(tokens):
        start = i
        while i < len(tokens) and not is_separator(tokens[i]):
            i += 1
        words = tokens[start:i]
        separator = tokens[i] if i < len(tokens) else ""
        i += 1

        standalone_assignment = bool(words) and all(ASSIGNMENT.match(w) for w in words)
        for w in words[1:] if words[:1] == ["export"] else words:
            name, sep, value = w.partition("=")
            if sep and ASSIGNMENT.match(w):
                variables[name] = value  # e.g. SQ=./target/debug/sq; $SQ tick  or  RUN="cargo run"; $RUN tick
        words, override, unset = strip_prefix(list(words), home)
        var = VARIABLE_REF.match(words[0]) if words else None
        if var and var.group(1) in variables:  # expand a known variable in command position
            try:
                words = shlex.split(variables[var.group(1)]) + words[1:]
            except ValueError:
                pass
        if standalone_assignment and override:
            state = True  # `HOME=/x;` retargets HOME for the rest of this shell
        safe_env = (state or override) and not unset

        if words and words[0] == "export":
            for w in words[1:]:
                name, sep, value = w.partition("=")
                if sep and name == "HOME":
                    state = overrides_home(value, home)
        elif words and os.path.basename(words[0]) in SHELLS:
            script = inline_script(words)
            for j, w in enumerate(words):  # here-string: bash <<< 'cmd'
                if w.startswith("<<<"):
                    script = w[3:] or (words[j + 1] if j + 1 < len(words) else "")
            if script is not None and not safe_env:
                h, a = analyze(script, home, depth + 1)
                hits, asks = hits + h, asks + a
        elif words and os.path.basename(words[0]) in INTERPRETERS:
            script = inline_script(words, "ce")
            if script is not None and not safe_env and interpreter_launches_sq(script):
                asks.append(" ".join(words[:2]))
        elif words and words[0] == "eval":
            if not safe_env:
                h, a = analyze(" ".join(words[1:]), home, depth + 1)
                hits, asks = hits + h, asks + a
        elif words and os.path.basename(words[0]) == "hyperfine":  # each argument is a command
            if not safe_env:
                for arg in words[1:]:
                    if not arg.startswith("-"):
                        h, a = analyze(arg, home, depth + 1)
                        hits, asks = hits + h, asks + a
        elif words and os.path.basename(words[0]) == "watch":  # runs `sh -c "<args joined>"`
            if not safe_env:
                rest, j = [], 1
                while j < len(words) and words[j].startswith("-"):
                    j += 2 if words[j] in WATCH_VALUE_FLAGS else 1
                h, a = analyze(" ".join(words[j:]), home, depth + 1)
                hits, asks = hits + h, asks + a
        elif words and os.path.basename(words[0]) == "script":  # runs a command under a pty
            if not safe_env:
                inline = inline_script(words)  # util-linux: script -qc "cmd" file
                if inline is None:  # BSD: script [-opts] file command...
                    positional = [w for w in words[1:] if not w.startswith("-")]
                    inline = shlex.join(positional[1:])
                h, a = analyze(inline, home, depth + 1)
                hits, asks = hits + h, asks + a
        else:
            args = sq_args(words)
            if args is not None and touches_save(args) and not safe_env:
                hits.append(" ".join(words))

        # Parentheses open/close subshells (and command substitutions): scope HOME changes.
        for ch in separator:
            if ch == "(":
                stack.append(state)
            elif ch == ")" and stack:
                state = stack.pop()
    return hits, asks


def persistent_override_before(command: str, pos: int, home: str) -> bool:
    """Was HOME retargeted for the rest of the shell (export or standalone assignment) before pos?"""
    masked = mask_single_quotes(command)
    found = False
    for m in re.finditer(r"(?:^|[;&|\n(]\s*)(export\s+)?HOME=([^\s;&|()]*)(\s*)(.?)", command[:pos]):
        if masked[m.start(2) - 5 : m.start(2)] != "HOME=":
            continue  # inside single quotes: just text
        standalone = bool(m.group(1)) or m.group(4) in (";", "&", "|", "\n", "")
        if standalone:
            found = overrides_home(m.group(2).strip("'\""), home)
    return found


def mentions_real_save(command: str, home: str) -> bool:
    command, _ = split_heredocs(command)
    patterns = [re.escape(f"{home}/.shellquest")]
    relative = [r"(?:^|[\s\"'=(:,\[])~/\.shellquest\b", r"\$\{?HOME\}?/\.shellquest\b"]
    for pat in patterns + relative:
        for m in re.finditer(pat, command):
            if pat in relative and persistent_override_before(command, m.start(), home):
                continue  # ~ and $HOME point at an overridden (sandbox) home here
            return True
    return False


def decide(command: str, home: str) -> tuple[str, str] | None:
    try:
        hits, asks = analyze(command, home)
    except ValueError:  # unbalanced quotes etc.: leave it to the normal permission flow
        hits, asks = [], []
    if hits:
        return "deny", f"Blocked `{hits[0]}`. {SANDBOX_HINT}"
    if asks:
        return "ask", (
            f"This {asks[0]} script appears to launch sq without setting HOME; it may touch the developer's "
            "live save. Prefer dev-tools/sq-sandbox, or pass an env with HOME set to a sandbox."
        )
    if mentions_real_save(command, home):
        return "ask", "This command references the developer's live save directory (~/.shellquest). Confirm it is intended."
    return None


def main() -> None:
    try:
        payload = json.load(sys.stdin)
        if payload.get("tool_name") != "Bash":
            return
        command = payload.get("tool_input", {}).get("command") or ""
        decision = decide(command, real_home())
    except Exception:  # never block work because the guard itself broke
        return
    if decision:
        verdict, reason = decision
        json.dump(
            {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": verdict,
                    "permissionDecisionReason": reason,
                }
            },
            sys.stdout,
        )


if __name__ == "__main__":
    main()
