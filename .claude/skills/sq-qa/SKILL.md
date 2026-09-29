---
name: sq-qa
description: Run, smoke-test, or manually QA the `sq` game binary in an isolated sandbox HOME. Use when asked to run, try, or demo sq; reproduce a gameplay bug; verify a CLI, message, or balance change end-to-end; or exercise the arena, Void quest, shop, enchanting, bosses, permadeath, or the update sage. Never run sq against the developer's real save.
---

# QA `sq` in the sandbox

The developer plays this game. `~/.shellquest/save.json` is their live character and their shell hook ticks it before every prompt, so a stray `sq tick`, `sq reset`, or permadeath test run with the real HOME destroys real progress. Every run goes through `dev-tools/sq-sandbox`, which rebuilds `target/debug/sq` (a no-op when fresh) and runs it with `HOME=<repo>/.sq-sandbox/home` and `SQ_NO_PACING=1`. The PreToolUse guard in `.claude/hooks/` denies unsandboxed `sq` and `cargo run`.

## Loop

1. **Fresh character:** `dev-tools/sq-sandbox new --class warrior --race human` (add `--permadeath` or `--name`). It runs the real `sq init` and pre-seeds the sage's version check so ticks stay offline and quiet.
2. **Seed the scenario** with dotted-path assignments (JSON values, bare strings, or `@now` / `@now-25h` / `@now-2d` timestamps):
   `dev-tools/sq-sandbox set character.level=60 character.gold=50000 quest_refreshed=@now-1d`
3. **Act.**
   - Pass any sq command through: `status`, `journal`, `shop`, `buy 1`, `sell junk`, `enchant sword`, `id sword`, `quest`. Commands that prompt need their answers piped in: `printf '1\ny\n' | dev-tools/sq-sandbox prestige`, `echo y | dev-tools/sq-sandbox run reset`.
   - Simulate shell activity with `dev-tools/sq-sandbox tick "cargo build" -n 30 --cwd /tmp/work --exit-code 101`. It prints each tick's output, then a before→after summary: level, xp, hp, gold, kills, deaths, inventory, boss.
4. **Inspect:** `dev-tools/sq-sandbox show character.inventory`, `dev-tools/sq-sandbox journal`.
5. **Report** the commands you ran and what you observed. Rerunning `new` resets everything; `dev-tools/sq-sandbox reset` deletes the sandbox.

## Know before you test

- **Working directory.** Commands run from the sandbox HOME, so the home-only commands work (shop, buy, sell, quest, and enchant for non-Wizards). Put `--in DIR` before the command to run elsewhere, e.g. to test the "return home first" refusal.
- **Zones come from strings.** `tick --cwd` is only the string sq maps to a zone; the directory need not exist. Default is the sandbox HOME (Home Village, danger 1). `/tmp/x` is the Wasteland (3); `/x/node_modules` is the Abyss (5).
- **Events are random.** Traps fire on ~25% of failing commands; bosses spawn at 1/500. Use `-n` for volume and judge the summary, not one roll.
- **Telemetry.** `SQ_DEBUG=1 dev-tools/sq-sandbox tick ...` prints `SQ_ENCOUNTER` lines and makes tick fail loudly on load/save errors.
- **Interactive commands.** `sq init` and the arena need `new` and `tty`. `dev-tools/sq-sandbox run <args>` forces raw passthrough, e.g. `echo y | dev-tools/sq-sandbox run reset` for sq's own reset flow. sq's prompts loop forever on closed stdin, so the wrapper kills a passthrough run after 4 MB of output or 300 s (exit 124). Pipe the answers in.
- **Edits are validated.** After `set` or `boss`, the wrapper makes sq load the save. If sq can't load it (e.g. `character.gold=1e6` is a float where sq wants an integer), the previous save is restored and the command fails. A silently unparseable save would make every later tick report "no change".
- **Parallel agents** must each use their own sandbox: `SQ_SANDBOX_HOME=$PWD/.sq-sandbox/<name> dev-tools/sq-sandbox …`.
- **Paths.** Keep the sandbox path free of zone keywords (`tmp`, `var`, `private`, `target`, `src`, `dev`, …). zones.rs checks path segments before it checks for `$HOME`, so a HOME under `/var/folders` (macOS `mktemp`) never registers as Home Village.

## Recipes

| Scenario | Commands |
|---|---|
| Class-flavored craft | `tick "git commit"` → `journal` (text matches the class voice) |
| Trap damage | `tick bad --exit-code 1 -n 12` (some ticks cost 3–6% max HP) |
| Zone XP scaling | `tick "git commit" -n 10` vs `tick "git commit" -n 10 --cwd /tmp/x`: compare the per-tick "+N XP" lines (`ls` never grants XP) |
| Shop / buy / sell | `shop` → `buy 1` → `sell junk` (sweeps Common + Uncommon only) |
| Enchant | `set character.gold=100000` → `buy 1` → `wield <name>` or `equip <name>` → `enchant <name>` (+1 power, gold deducted, `[Enchanted +N]` tag, capped at +5); non-Wizard away from home: `--in / enchant <name>` is refused |
| Identify | `id <name>` prints the item card; `show commands_run` is unchanged (read-only, no tick consumed) |
| World boss | `boss` or `boss "kernel panic"` → `tick ls -n 10`; stale flee: `boss --age 25h` → `tick ls`; clear: `set active_boss=null` |
| Permadeath | `new --permadeath` → `set character.hp=1` → `tick bad --exit-code 1 -n 30` → eulogy, summary says the save is gone |
| Sage notice | `set latest_version=99.0.0 last_announced_version=null` → `tick ls` (appears once), or `run tick --cmd ls --test-sage` |
| Void quest | `quest` → `find "$(dev-tools/sq-sandbox path)/.shellquest/the_void" -name 'lost_scroll_*'` → read it → `quest answer <phrase>`; wrong phrase gives a hint; `set quest_refreshed=@now-1d` → `quest` reshuffles |
| Arena cash-out | `new` → `set character.level=20 character.max_hp=400 character.hp=400 character.gold=2000` → `tty -i 1 -i y -i 2 -- arena` (the Pit, pay fee, cash out after round 1; `set` doesn't raise stats, so give HP explicitly) |
| Arena KO | `new` (level 1) → `set character.gold=500` → `tty -i 1 -i y -i 1 -i 1 -i 1 -- arena` → fee gone, HP = 25% of max, "Knocked out" summary, `journal` shows "Arena KO in …" |
| Arena rollback | `new` → `set character.gold=500` → `tty -i 1 -i y -- arena` (inputs run out mid-run, so the tool sends Ctrl-C) → `show character.gold` is still 500 |
| Arena chest overflow | fill the pack: `set "character.inventory=$(python3 -c 'import json;print(json.dumps([{"name":f"Junk {i}","slot":"Ring","power":1,"rarity":"Common","enchant_level":0} for i in range(20)]))')"`, then win a chest round |
| Arena needs a TTY | `SB=$(dev-tools/sq-sandbox path) && HOME="$SB" target/debug/sq arena </dev/null` prints "Arena requires an interactive terminal." (keep the `&&`: an empty HOME makes sq fall back to the real home) |
| Prestige | `set character.level=150` → `printf '1\ny\n' \| dev-tools/sq-sandbox prestige` (subclass 1, confirm) |

**Arena menus.** Tier select: `1`–`5`, or empty to cancel. Fee confirmation: `y`. Between rounds: `1` Continue, `2` Cash Out, `3` Healer, `4` Quaff potion (shown only with potions in the pack). `tty` types each `-i` line when output goes idle (0.6 s). When the lines run out it sends Ctrl-C, or Ctrl-D with `--eof`. Use `--paced` to watch real 1.5 s pacing.

## Beyond the sandbox

- **Release QA.** Before a release, run every recipe row above that the release touches. This table is the project's manual-QA checklist; AGENTS.md points here.
- **Balance questions.** "Is X too strong?" needs the simulator, not a handful of sandbox ticks; see the `balance-check` skill.
- **Real-shell behavior** (what the hook ticks, rc upgrades) is covered by `tests/shell_hooks/test_shell_hooks.py`. It installs the hook into a temp HOME with the real binary and drives zsh/bash through a pty with a fake `sq` logger; extend it rather than poking at a live shell. Never touch the developer's rc files.
