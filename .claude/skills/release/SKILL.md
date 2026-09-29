---
name: release
description: Prepare and ship a shellquest release. Covers the cadence check, release notes in the project voice, quality gates, and publish.sh (git push, GitHub release, crates.io). Use only when the user explicitly asks to cut, prepare, or publish a release.
disable-model-invocation: true
---

# Cut a shellquest release

Publishing is irreversible: crates.io versions can be yanked but never deleted, and the GitHub release notifies watchers. Run `./publish.sh` only after the user has confirmed the version and the notes in this session. The `ask` permission rules on `publish.sh`, `cargo publish`, and `gh release` enforce the same thing.

## 1. Is it time?

Follow AGENTS.md → Release Cadence.

1. Last release: `git fetch --tags origin && git describe --tags --abbrev=0`. publish.sh creates tags only on GitHub, so skipping the fetch returns an older tag. Then `git log --oneline <tag>..HEAD`.
2. Sort the commits into player-visible and internal.
3. Ship only if the notes would carry at least 5 paragraphs of player-visible material on one coherent theme. A critical bug fix is the exception: it ships right away as a patch.
4. If the release is below the bar, tell the user what is pending and stop.
5. Choose the bump:
   - `minor` for a feature or balance arc.
   - `patch` for fixes.
   - `major` only when old saves or the shell hook break.

## 2. Gates

Every gate must pass.

- **Code:** `just check` (rustfmt, clippy, tests, guard-hook tests).
- **Balance:** if gameplay numbers changed since the tag, confirm each change was sim-validated (the `balance-check` skill). If one wasn't, stop and say so.
- **Save compatibility:** check `git diff <tag>..HEAD -- src/state.rs src/character.rs src/boss.rs src/journal.rs`. Every new persisted field needs `#[serde(default)]`.
  - When those structs changed, load a save created by the previous release.
  - Build the tag in a scratch worktree: `git worktree add ../sq-prev <tag>`, then `cargo build` inside it.
  - Create the save with the old build in a fresh sandbox: `dev-tools/sq-sandbox reset && SB=$(dev-tools/sq-sandbox path) && printf 'Old\n2\n1\nn\n' | HOME="$SB" ../sq-prev/target/debug/sq init`. The reset matters: with an existing save, init's overwrite prompt eats the answers. The `&&` chain matters too: an empty HOME makes sq fall back to the real home.
  - Load it with the new build: `dev-tools/sq-sandbox status`.
  - Clean up: `git worktree remove ../sq-prev`.
- **Behavior:** run the relevant `sq-qa` recipes for every player-facing change.
- **Docs:** README.md tables, `src/help.rs` topics, and the `sq init` prompt text must match the new numbers and commands.

## 3. Release notes

1. Write `release-notes/vX.Y.Z.md`. Follow `release-notes/README.md` for voice and structure; `v1.16.0.md` and `v1.17.0.md` are the reference.
   - A headline that states the benefit.
   - A second-person hook.
   - Prose for each feature, plus real terminal output captured from the sandbox.
   - A Balance Note if any numbers moved.
   - An honest compatibility footer and the Full Changelog link.
2. Commit the notes before publishing: `docs(release-notes): add vX.Y.Z notes`.

## 4. Publish

Only after the user says go.

1. Show the user the bump, the notes file, and the commit range, and get an explicit yes.
2. Run `./publish.sh <patch|minor|major|X.Y.Z>`. It does the following, in order:
   - checks for a clean tree;
   - bumps Cargo.toml and runs `cargo build --release`;
   - commits `vX.Y.Z` and pushes master;
   - runs `gh release create` with the notes file;
   - runs `cargo publish`.

   It does **not** run the tests (the gates above do). It also does **not** build or push a Docker image, even though its header and final summary mention Docker.
3. On a failure partway through, don't rerun it; the version bump would double up. Find which steps landed (`git log -1`, `gh release view vX.Y.Z`, `cargo search shellquest --limit 1`) and finish the rest by hand.

## 5. After

- `bd close` the shipped issues and push.
- Tell the user what shipped, with the release URL.
