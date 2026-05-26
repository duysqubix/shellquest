set shell := ["bash", "-cu"]

sim_dir := justfile_directory() / "dev-tools/balance-sim"
py := if `command -v uv >/dev/null 2>&1; echo $?` == "0" { "uv run --script" } else { "python3" }
runner := py + " " + sim_dir / "runner.py"
report := py + " " + sim_dir / "report.py"
dashboard := py + " " + sim_dir / "dashboard.py"
watcher := py + " " + sim_dir / "watch.py"
default_label := "v1.24-soft"

# List every recipe with its description (this is the default action when you run `just`).
default:
    @just --list --unsorted

# Compile the `sq` binary in dev mode (target/debug/sq). Run before any sim recipe to pick up source changes.
build:
    cargo build --bin sq

# Run the full Rust unit test suite (361 tests including all arena/wave/crit logic).
test:
    cargo test

# Fastest smoke test: 1 Warrior to L20, single worker, ~15 seconds. Verifies the harness end-to-end.
sim-quick label=default_label:
    {{runner}} --runs 1 --classes Warrior --races Human --strategies greedy \
        --target-level 20 --parallel 1 \
        --tuning-label {{label}} --min-arena-tier 1

# Pit-focused sweep: 4 classes × 3 strategies × N runs, target L22. Generates Pit (tier 1) arena data only.
sim-pit label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Necromancer \
        --races Human --strategies greedy balanced conservative \
        --target-level 22 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 1

# Gauntlet-focused sweep: all 5 classes × 3 strategies × N runs, target L40, AI skips Pit and only enters Gauntlet.
sim-gauntlet label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced conservative \
        --target-level 40 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 2

# Colosseum-focused sweep: 5 classes × 2 strategies × N runs, target L75 (unlocks tier 3), AI skips Pit/Gauntlet.
sim-colosseum label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 75 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 3

# Abyssal-focused sweep: 5 classes × 2 strategies × N runs, target L115. Slow (~20 min per char) — use parallel=4+.
sim-abyssal label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 115 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 4

# Endgame sweep: 3 strongest classes × 2 strategies × N runs, target L150. Very slow (~1 hr per char). Plan for it.
sim-endgame label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 150 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 4

# Comprehensive lifetime sweep: 5 classes × 3 races × 3 strategies × N runs through L60. ~45 sims at N=1, scales with N.
sim-full label=default_label runs="5":
    {{runner}} --runs {{runs}} \
        --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human Elf Dwarf \
        --strategies greedy balanced conservative \
        --target-level 60 --parallel 4 \
        --tuning-label {{label}}

# Free-form sim. Example: just sim-custom v1.25-test "Warrior Wizard" "greedy balanced" 80 3 5 4
sim-custom label classes strategies target_level min_tier="1" runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes {{classes}} \
        --strategies {{strategies}} --races Human \
        --target-level {{target_level}} --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier {{min_tier}}

# Live in-place tail of the sims DB. Run in a second terminal while a long sim is going. Ctrl-C to exit.
watch:
    {{watcher}}

# Generate markdown report for a tuning_label. Output: dev-tools/balance-sim/reports/<label>.md
report label=default_label:
    {{report}} --tuning-label {{label}} \
        --output {{sim_dir}}/reports/{{label}}.md
    @echo "report → {{sim_dir}}/reports/{{label}}.md"

# Generate the interactive HTML dashboard (Chart.js, A/B compare). Open the printed file:// URL in any browser.
dashboard label=default_label:
    {{dashboard}} --primary {{label}}
    @echo "open file://{{sim_dir}}/dashboard.html"

# Wipe sims DB + dashboard artifacts. Use before a fresh sweep when you don't want stale runs mixed in.
clean-sims:
    rm -f {{sim_dir}}/runs.db {{sim_dir}}/runs.db-* {{sim_dir}}/runs-w*.db*
    rm -f {{sim_dir}}/dashboard.html
    @echo "cleared sim DB and dashboard"

# Run the publish.sh release pipeline (patch / minor / major / X.Y.Z). Bumps version, builds, pushes, publishes.
ship version="patch":
    ./publish.sh {{version}}
