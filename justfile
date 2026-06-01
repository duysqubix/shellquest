set shell := ["bash", "-cu"]

sim_dir := justfile_directory() / "dev-tools/balance-sim"
py := if `command -v uv >/dev/null 2>&1; echo $?` == "0" { "uv run --script" } else { "python3" }
runner := py + " " + sim_dir / "runner.py"
report := py + " " + sim_dir / "report.py"
dashboard := py + " " + sim_dir / "dashboard.py"
watcher := py + " " + sim_dir / "watch.py"
# Empty by default so each sim-* run auto-labels as '<recipe>-<timestamp>' (kept separate
# and selectable in the dashboard). Pass label=NAME to override / group runs intentionally.
default_label := ""

# List every recipe with its description (this is the default action when you run `just`).
default:
    @just --list --unsorted

# Compile the `sq` binary in dev mode (target/debug/sq). Run before any sim recipe to pick up source changes.
build:
    cargo build --bin sq

# Run the full Rust unit test suite (361 tests including all arena/wave/crit logic).
test:
    cargo test

# Fastest smoke test: 1 Warrior to L20, single worker. Runs in Docker (one container per character); requires the sim image + a linux sq.
sim-quick label=default_label: sim-image sim-sq-linux
    LABEL="{{label}}"; LABEL="${LABEL:-sim-quick-$(date +%Y%m%d-%H%M%S)}"; \
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs 1 --classes Warrior --races Human --strategies greedy \
        --target-level 20 --parallel 1 \
        --tuning-label "$LABEL" --min-arena-tier 1

# Pit-focused sweep: 4 classes × 3 strategies × N runs, target L22. Runs in Docker (one container per character); requires the sim image + a linux sq. Generates Pit (tier 1) arena data only.
sim-pit label=default_label runs="3" parallel="4": sim-image sim-sq-linux
    LABEL="{{label}}"; LABEL="${LABEL:-sim-pit-$(date +%Y%m%d-%H%M%S)}"; \
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Necromancer \
        --races Human --strategies greedy balanced conservative \
        --target-level 22 --parallel {{parallel}} \
        --tuning-label "$LABEL" --min-arena-tier 1

# Gauntlet-focused sweep: all 5 classes × 3 strategies × N runs, target L40, AI skips Pit and only enters Gauntlet. Runs in Docker (one container per character); requires the sim image + a linux sq.
sim-gauntlet label=default_label runs="3" parallel="4": sim-image sim-sq-linux
    LABEL="{{label}}"; LABEL="${LABEL:-sim-gauntlet-$(date +%Y%m%d-%H%M%S)}"; \
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced conservative \
        --target-level 40 --parallel {{parallel}} \
        --tuning-label "$LABEL" --min-arena-tier 2

# Colosseum-focused sweep: 5 classes × 2 strategies × N runs, target L75 (unlocks tier 3), AI skips Pit/Gauntlet. Runs in Docker (one container per character); requires the sim image + a linux sq.
sim-colosseum label=default_label runs="3" parallel="4": sim-image sim-sq-linux
    LABEL="{{label}}"; LABEL="${LABEL:-sim-colosseum-$(date +%Y%m%d-%H%M%S)}"; \
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 75 --parallel {{parallel}} \
        --tuning-label "$LABEL" --min-arena-tier 3

# Abyssal-focused sweep: 5 classes × 2 strategies × N runs, target L115. Runs in Docker (one container per character); requires the sim image + a linux sq. Slow (~20 min per char) — use parallel=4+.
sim-abyssal label=default_label runs="3" parallel="4": sim-image sim-sq-linux
    LABEL="{{label}}"; LABEL="${LABEL:-sim-abyssal-$(date +%Y%m%d-%H%M%S)}"; \
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 115 --parallel {{parallel}} \
        --tuning-label "$LABEL" --min-arena-tier 4

# Endgame sweep: 3 strongest classes × 2 strategies × N runs, target L150. Runs in Docker (one container per character); requires the sim image + a linux sq. Very slow (~1 hr per char). Plan for it.
sim-endgame label=default_label runs="3" parallel="4": sim-image sim-sq-linux
    LABEL="{{label}}"; LABEL="${LABEL:-sim-endgame-$(date +%Y%m%d-%H%M%S)}"; \
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} --classes Warrior Wizard Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 150 --parallel {{parallel}} \
        --tuning-label "$LABEL" --min-arena-tier 4

# Comprehensive lifetime sweep: 5 classes × 3 races × 3 strategies × N runs through L60. Runs in Docker (one container per character); requires the sim image + a linux sq. ~45 sims at N=1, scales with N.
sim-full label=default_label runs="5": sim-image sim-sq-linux
    LABEL="{{label}}"; LABEL="${LABEL:-sim-full-$(date +%Y%m%d-%H%M%S)}"; \
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} \
        --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human Elf Dwarf \
        --strategies greedy balanced conservative \
        --target-level 60 --parallel 4 \
        --tuning-label "$LABEL"

# Free-form sim. Runs in Docker (one container per character); requires the sim image + a linux sq. Example: just sim-custom v1.25-test "Warrior Wizard" "greedy balanced" 80 3 5 4
sim-custom label classes strategies target_level min_tier="1" runs="3" parallel="4": sim-image sim-sq-linux
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} --classes {{classes}} \
        --strategies {{strategies}} --races Human \
        --target-level {{target_level}} --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier {{min_tier}}

sim-custom-seeded label classes strategies start_level target_level min_tier="4" runs="3" parallel="4" start_prestige="0": sim-image sim-sq-linux
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} --classes {{classes}} \
        --strategies {{strategies}} --races Human \
        --start-level {{start_level}} --start-prestige {{start_prestige}} --target-level {{target_level}} --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier {{min_tier}}

sim-custom-seeded-prestige label classes strategies start_level start_prestige target_level min_tier="5" runs="3" parallel="4": sim-image sim-sq-linux
    SQ_BIN_HOST={{sim_dir}}/.sq-linux/sq SIM_IMAGE=shellquest-sim {{runner}} --runs {{runs}} --classes {{classes}} \
        --strategies {{strategies}} --races Human \
        --start-level {{start_level}} --start-prestige {{start_prestige}} --target-level {{target_level}} --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier {{min_tier}}

# Build the balance-sim Docker image (dev-only). sq binary + sim code are mounted at run time, not baked in.
sim-image:
    docker build -t shellquest-sim {{sim_dir}}

# Build a Linux `sq` for the sim containers (host `cargo build` may be macOS/Mach-O, which can't run in the Linux sim image).
sim-sq-linux:
    docker build -q -t shellquest-game {{justfile_directory()}} >/dev/null
    mkdir -p {{sim_dir}}/.sq-linux
    rm -f {{sim_dir}}/.sq-linux/sq
    set -e; cid=$(docker create shellquest-game); trap 'docker rm "$cid" >/dev/null 2>&1 || true' EXIT; docker cp "$cid":/usr/local/bin/sq {{sim_dir}}/.sq-linux/sq; test -x {{sim_dir}}/.sq-linux/sq
    @echo "linux sq -> {{sim_dir}}/.sq-linux/sq"

# Live in-place tail of the sims DB. Run in a second terminal while a long sim is going. Ctrl-C to exit.
watch:
    {{watcher}}

# Generate markdown report for a tuning_label. Pass the label positionally:
#   just report sim-quick-20260530-142233   (no arg = most recent label).
# Output: dev-tools/balance-sim/reports/<label>.md
report label=default_label:
    LABEL="{{label}}"; \
    if [ -z "$LABEL" ]; then \
      LABEL=$({{py}} -c "import sys,sqlite3; sys.path.insert(0,'{{sim_dir}}'); from simulator import db; rows=db.open_db(db.DEFAULT_DB_PATH).execute('SELECT tuning_label FROM run ORDER BY id DESC LIMIT 1').fetchall(); print(rows[0][0] if rows else '')"); \
    fi; \
    if [ -z "$LABEL" ]; then echo "no runs in db — run a sim-* recipe first" >&2; exit 1; fi; \
    mkdir -p {{sim_dir}}/reports; \
    {{report}} --tuning-label "$LABEL" --output {{sim_dir}}/reports/"$LABEL".md; \
    echo "report → {{sim_dir}}/reports/$LABEL.md"

# Generate the interactive HTML dashboard (Chart.js, A/B compare). Open the printed file:// URL in any browser.
dashboard label=default_label:
    LABEL="{{label}}"; \
    if [ -n "$LABEL" ]; then {{dashboard}} --primary "$LABEL"; else {{dashboard}}; fi; \
    echo "open file://{{sim_dir}}/dashboard.html"

# Wipe sims DB + dashboard artifacts. Use before a fresh sweep when you don't want stale runs mixed in.
clean-sims:
    rm -f {{sim_dir}}/runs.db {{sim_dir}}/runs.db-* {{sim_dir}}/runs-w*.db*
    rm -f {{sim_dir}}/dashboard.html
    rm -rf {{sim_dir}}/.sq-linux
    @echo "cleared sim DB and dashboard"

# Run the publish.sh release pipeline (patch / minor / major / X.Y.Z). Bumps version, builds, pushes, publishes.
ship version="patch":
    ./publish.sh {{version}}
