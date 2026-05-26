set shell := ["bash", "-cu"]

sim_dir := justfile_directory() / "dev-tools/balance-sim"
py := if `command -v uv >/dev/null 2>&1; echo $?` == "0" { "uv run --script" } else { "python3" }
runner := py + " " + sim_dir / "runner.py"
report := py + " " + sim_dir / "report.py"
dashboard := py + " " + sim_dir / "dashboard.py"
watcher := py + " " + sim_dir / "watch.py"
default_label := "v1.24-soft"

default:
    @just --list --unsorted

build:
    cargo build --bin sq

test:
    cargo test

sim-quick label=default_label:
    {{runner}} --runs 1 --classes Warrior --races Human --strategies greedy \
        --target-level 20 --parallel 1 \
        --tuning-label {{label}} --min-arena-tier 1

sim-pit label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Necromancer \
        --races Human --strategies greedy balanced conservative \
        --target-level 22 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 1

sim-gauntlet label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced conservative \
        --target-level 40 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 2

sim-colosseum label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 75 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 3

sim-abyssal label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 115 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 4

sim-endgame label=default_label runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes Warrior Wizard Necromancer \
        --races Human --strategies greedy balanced \
        --target-level 150 --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier 4

sim-full label=default_label runs="5":
    {{runner}} --runs {{runs}} \
        --classes Warrior Wizard Rogue Ranger Necromancer \
        --races Human Elf Dwarf \
        --strategies greedy balanced conservative \
        --target-level 60 --parallel 4 \
        --tuning-label {{label}}

sim-custom label classes strategies target_level min_tier="1" runs="3" parallel="4":
    {{runner}} --runs {{runs}} --classes {{classes}} \
        --strategies {{strategies}} --races Human \
        --target-level {{target_level}} --parallel {{parallel}} \
        --tuning-label {{label}} --min-arena-tier {{min_tier}}

watch:
    {{watcher}}

report label=default_label:
    {{report}} --tuning-label {{label}} \
        --output {{sim_dir}}/reports/{{label}}.md
    @echo "report → {{sim_dir}}/reports/{{label}}.md"

dashboard label=default_label:
    {{dashboard}} --primary {{label}}
    @echo "open file://{{sim_dir}}/dashboard.html"

clean-sims:
    rm -f {{sim_dir}}/runs.db {{sim_dir}}/runs.db-* {{sim_dir}}/runs-w*.db*
    rm -f {{sim_dir}}/dashboard.html
    @echo "cleared sim DB and dashboard"

ship version="patch":
    ./publish.sh {{version}}
