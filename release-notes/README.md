# release-notes/

User-facing release notes for each published version, in the voice of the project — written for actual humans, not as auto-generated commit changelogs.

## Convention

- One file per release: `vX.Y.Z.md` (e.g. `v1.17.0.md`).
- The file is committed **before** running `./publish.sh`. `publish.sh` detects it via the convention path `release-notes/v${NEW_VERSION}.md` and passes it to `gh release create --notes-file`.
- If no file exists for the target version, `publish.sh` falls back to `--generate-notes` (auto changelog from commits) and prints a warning.

## How it flows

1. Decide the target version (e.g. `1.18.0`).
2. Write `release-notes/v1.18.0.md` using the structure below.
3. Commit it — typically alongside the feature commit, or as a separate `docs(release-notes):` commit.
4. Run `./publish.sh minor` (or `major`/`patch`/exact). It picks the file up automatically.

## Source of truth

This directory is the canonical source. GitHub release notes can be re-pulled from these files at any time:

```bash
gh release edit v1.17.0 --notes-file release-notes/v1.17.0.md
```

If you edit notes directly on GitHub, mirror the change back here in a follow-up commit so the two stay in sync.

## Structure (canonical voice)

The reference is [`v1.16.0.md`](v1.16.0.md) and [`v1.17.0.md`](v1.17.0.md). The shape:

```markdown
# vX.Y.Z — <short headline stating user-visible benefit>

<2–3 sentence hook paragraph. Address the player's pain. Promise the fix.
Write for someone scanning the release page, not someone reading the diff.>

---

## <emoji> <Feature 1 name>

<Prose explanation of what changed and why it matters.
Include a fenced code block showing the new behavior in action —
real terminal output, real commands.>

```

(Repeat per feature.)

```markdown
---

## ⚠️ Balance Note  *(only if gameplay numbers shifted)*

<Call out any change that will feel different mid-run.>

---

**Saves are unchanged. CLI surface is unchanged. No migration needed.**  *(adjust as honest)*

**Full Changelog**: https://github.com/duysqubix/shellquest/compare/v<PREV>...v<NEW>
```

## Voice rules

- Title is a benefit statement, not a feature list. `# v1.16.0 — The Arena Slows Down So You Can Watch`, not `# v1.16.0 — Release Notes`.
- Hook paragraph addresses the user directly, in second person. No corporate voice.
- Each feature gets prose + a real output example. No bullet-point dumps.
- Always show terminal output the user will actually see — colors, emojis, exact spacing where it matters.
- End with the migration / compatibility footer. Be honest: if a save schema changed, say so loudly.

## Scope: what to include

| Include | Skip |
|---|---|
| Player-facing behavior changes | Internal refactors |
| New CLI commands or flags | Test additions |
| Balance / gameplay shifts | Lint / formatting passes |
| New content (items, bosses, zones) | CI / build tweaks |
| Save / shell-hook compatibility notes | Doc-only changes (mention briefly at most) |

For a release that is mostly chores, a short note is fine — keep the convention so the file exists. Don't pad with marketing.
