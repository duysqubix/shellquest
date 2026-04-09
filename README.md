<p align="center">
  <h1 align="center">shellquest (sq)</h1>
  <p align="center">
    <strong>A passive RPG that lives in your terminal. Your shell is the dungeon.</strong>
  </p>
  <p align="center">
    <a href="https://crates.io/crates/shellquest"><img src="https://img.shields.io/crates/v/shellquest?style=flat-square&color=orange" alt="Crates.io"></a>
    <a href="https://crates.io/crates/shellquest"><img src="https://img.shields.io/crates/d/shellquest?style=flat-square&color=blue" alt="Downloads"></a>
    <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" alt="License: MIT"></a>
    <a href="https://github.com/duysqubix/shellquest"><img src="https://img.shields.io/github/stars/duysqubix/shellquest?style=flat-square&color=yellow" alt="Stars"></a>
  </p>
</p>

---

Every command you run has a chance to trigger an encounter. Your character gains XP, finds loot, fights monsters, and levels up -- all while you're just doing your normal work. Zero effort. Zero interruption. Pure vibes.

```
~/projects $ git push
  🏆 Quest complete! You pushed your code to the realm! +21 XP +8 gold

~/projects $ cargo build
  💎 ★·.· The forge burns hot! You crafted: ★Vorpal Pointer★ (+18 Weapon) [Epic] ·.·★

~/projects $ docker compose up
  🏆 The orchestration ritual completes! A symphony of microservices plays! +23 XP +9 gold

~/projects $ cat README.md
  🐾 You befriend a friendly daemon! It heals you for 5 HP. HP: 47/47

~/projects $ rm -rf node_modules
  ⚔️  A Dependency Hell Hound appears! You strike true! Victory! +22 XP

~/projects $ bad_command
  🪤 You stumble on a trap! Took 3 damage. HP: 44/47
```

---

## Install

### From crates.io (recommended)

```bash
cargo install shellquest
```

### One-liner (auto-setup)

```bash
curl -fsSL https://raw.githubusercontent.com/duysqubix/shellquest/main/install.sh | bash
```

Clones, builds, installs `sq`, detects your shell, and adds the hook automatically.

### From source

```bash
git clone https://github.com/duysqubix/shellquest.git
cd shellquest
cargo install --path .
sq hook --shell zsh >> ~/.zshrc   # or bash/fish
```

> **Requires:** [Rust](https://rustup.rs) (cargo)

---

## Quick Start

```bash
source ~/.zshrc          # reload shell (or restart terminal)
sq init                  # create your character
                         # ...now just use your terminal!
sq status                # check your stats anytime
```

---

## How The Tick Works

The magic behind shellquest is a **shell hook** -- a tiny function that runs invisibly after every command you type.

### The Flow

```
You type a command
        |
        v
  Command executes normally
        |
        v
  Shell hook fires (precmd / PROMPT_COMMAND)
        |
        v
  sq tick --cmd "your cmd" --cwd "/your/path" --exit-code 0
        |       (runs in background with & disown)
        v
  ┌─────────────────────────────────────────┐
  │  1. Load save from ~/.shellquest/       │
  │  2. Increment commands_run              │
  │  3. Check exit code (!=0 = trap!)       │
  │  4. Match command to event table        │
  │  5. Roll dice for event trigger         │
  │  6. Run event (combat/loot/xp/travel)   │
  │  7. Passive heal check (25% chance)     │
  │  8. Atomic save back to disk            │
  └─────────────────────────────────────────┘
        |
        v
  Output appears on stderr (never pollutes pipes)
  Total time: <1ms
```

### What the hook looks like (zsh)

```bash
__sq_hook() {
    local exit_code=$?                    # capture exit code
    local cmd=$(fc -ln -1)                # grab last command from history
    sq tick --cmd "$cmd" \                # pass command text
            --cwd "$PWD" \                # pass current directory
            --exit-code "$exit_code" \    # pass success/failure
            2>/dev/null &                 # background, silence errors
    disown 2>/dev/null                    # detach from shell
}
precmd_functions+=(__sq_hook)
```

### Why it's invisible

- Runs in the **background** (`&`) -- never blocks your prompt
- **Detached** (`disown`) -- won't die with your shell
- Output goes to **stderr** -- `ls | grep foo` still works perfectly
- Completes in **<1ms** -- faster than your terminal can redraw
- **No network calls** -- everything is local file I/O

### Event probability

Not every command triggers something. Each command type has its own trigger chance:

| Trigger | Chance |
|---------|--------|
| `git commit` (craft) | Always |
| `git push` (quest) | Always |
| `cargo build` (forge) | Always |
| `docker compose` (orchestra) | 1 in 3 |
| `rm` (angry spirit) | 1 in 3 |
| `kill` (banish) | 1 in 3 |
| `sudo` (power surge) | 1 in 4 |
| `ssh`/`curl` (portal) | 1 in 4 |
| `grep` (scrying) | 1 in 4 |
| `cat`/`less` (familiar) | 1 in 6 |
| `vim`/`nvim` (meditation) | 1 in 5 |
| Everything else | ~15% (3 in 20) |
| Failed command (trap) | Always |

---

## Command Events

| Command | What Happens |
|---------|-------------|
| `git commit` | Craft XP from committing to the archives |
| `git push` | Quest completion -- XP + gold |
| `cargo build` / `npm build` | Forge -- chance to craft weapons & armor |
| `docker build` | Container forge -- high-quality loot |
| `docker compose` | Orchestration ritual -- big XP + gold |
| `docker pull` | Summon image from the Cloud Registry |
| `docker stop` / `docker rm` | Banish container to the void |
| `rm` / `del` | Angry spirits attack |
| `cat` / `less` / `bat` | Befriend a familiar (heals HP) |
| `sudo` | Power surge -- raw energy XP |
| `vim` / `nvim` / `emacs` | Editor meditation -- heals + XP |
| `kill` / `pkill` | Banish a rogue process -- combat XP + gold |
| `grep` / `rg` | Scrying -- find hidden loot or patterns |
| `tar` / `zip` | Open a treasure chest -- guaranteed loot |
| `man` / `tldr` | Study ancient tomes -- knowledge XP |
| `ssh` / `curl` / `wget` | Open a portal to remote realms |
| `python` / `node` / `ruby` | Cast an interpreted incantation |
| `pip` / `gem` | Alchemy -- transmute packages into power |
| `chmod` / `chown` | Enchant files with new permissions |
| `cp` / `mv` / `rsync` | Telekinesis -- move files with your mind |
| `top` / `htop` | Omniscience -- peer into the process table |
| `echo` / `printf` | Echo spell -- resonance heals HP |
| Failed commands | Traps -- take damage |
| Everything else | ~15% random encounter (combat, loot, gold, heal, or XP) |

---

## Character System

```
┌──────────────────────────────────────────┐
│  Ferris the Goblin Assassin Rogue  (Lvl 42 [P1])
│
│  HP: ████████████████░░░░ 89/110
│  XP: ████████░░░░░░░░░░░░ 2340/5660
│
│  STR: 52  DEX: 68  INT: 47
│  Gold: 1,847
│
│  Weapon: ✦ Mass Migration Blade of the Kernel ✦ (+31) [LEGENDARY]
│  Armor:  ★ Warplate of Kubernetes ★ (+16) [Epic]
│  Ring:   Band of the Borrow Checker (+9) [Rare]
│
│  Kills: 312  Deaths: 7  Cmds: 14,203
│  Title: Prestigious Daemon Slayer
└──────────────────────────────────────────┘
```

- **5 Classes**: Wizard, Warrior, Rogue, Ranger, Necromancer
- **5 Races**: Human, Elf, Dwarf, Orc, Goblin
- **Stats**: STR (attack), DEX (dodge + crit), INT (class power)
- **150 Levels** with scaling XP curve
- **15 Titles** from *Terminal Novice* to *Root Overlord*

---

## Loot

80+ dev-themed items across 5 rarity tiers, each with its own visual flair:

```
  📦 You found: Rusty Pipe (+2 Weapon) [Common]
  📦 ~ You crafted: Mace of Makefile (+6 Weapon) [Uncommon]
  📦 ~~ Your search reveals: Scythe of Segfault (+13 Weapon) [Rare] ~~
  💎 ★·.· The forge burns hot! You crafted: ★Mjolnir of Monorepo★ (+19 Weapon) [Epic] ·.·★
  ╔═══════════════════════════════════════════╗
  ║ ✦✦✦ Mass Migration Blade of the Kernel (+31 Weapon) [LEGENDARY] ✦✦✦ ║
  ╚═══════════════════════════════════════════╝
```

**Sample items:**

| Slot | Common | Rare | Legendary |
|------|--------|------|-----------|
| Weapon | Rusty Pipe, Floppy Disk Shuriken | Blade of Sudo, Trident of TypeScript | Mass Migration Fork Bomb |
| Armor | Hoodie of Comfort, Pajama Pants of WFH | Shield of CORS, Greaves of GraphQL | Divine Armor of /dev/null |
| Ring | Ring of Tab Completion | Band of the Borrow Checker | Eternal Band of Uptime |
| Potion | Potion of Coffee | Draught of Deep Work | Elixir of Infinite Context |

---

## Zones

Your current directory determines the biome. Higher danger = tougher monsters but better loot.

| Path | Zone | Danger | Flavor |
|------|------|:------:|--------|
| `~` | Home Village | 1 | *The safety of your home directory...* |
| `src` / `lib` | The Source Sanctum | 2 | *Lines of power flow through structured halls...* |
| `test` | The Proving Grounds | 2 | *Assertions echo through the arena...* |
| `/etc` | The Config Archives | 2 | *Ancient scrolls of configuration line the walls...* |
| `/tmp` | The Wasteland | 3 | *A desolate land where files come to die...* |
| `/var` | The Variable Marshes | 3 | *Shifting logs and pools of data...* |
| `.git` | The Time Vaults | 3 | *Echoes of past commits whisper around you...* |
| `/dev` | The Device Caverns | 4 | *Strange devices hum with raw power...* |
| `node_modules` | The Abyss | 5 | *An infinite void of dependencies...* |

---

## Prestige System

At level 150, you've mastered the terminal. But the journey doesn't end -- it ascends.

```bash
sq prestige
```

```
  ✨ PRESTIGE ✨
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  You will reset to level 1 but gain:
  • +2 to all stats per prestige tier
  • A subclass with unique stat bonuses
  • +10 max HP per prestige tier
  • You keep your gold, gear, kills, and inventory

  Choose your subclass:
    1. Assassin — STR:+1 DEX:+3 INT:+0
    2. Hacker   — STR:+0 DEX:+2 INT:+2
    3. Shadow   — STR:+1 DEX:+3 INT:+0
```

**15 subclasses** (3 per base class):

| Class | Subclasses |
|-------|-----------|
| Wizard | Archmage, Chronomancer, Datamancer |
| Warrior | Berserker, Paladin, Warlord |
| Rogue | Assassin, Hacker, Shadow |
| Ranger | Beastmaster, Sniper, Scout |
| Necromancer | Lich, Plaguebearer, Soul Reaper |

Prestige title tiers: **Prestigious** > **Exalted** > **Transcendent** > **Mythical** > **Godlike**

---

## Commands

| Command | Description |
|---------|-------------|
| `sq init` | Create a new character |
| `sq status` | View your character sheet |
| `sq inventory` | Check your gear and potions |
| `sq journal` | Adventure log (last 20 events) |
| `sq prestige` | Ascend with a subclass (requires level 150) |
| `sq hook --shell zsh` | Print shell hook code |
| `sq reset` | Permanently delete your character |

---

## Design

| | |
|---|---|
| **Binary** | Single Rust executable, ~1.6MB |
| **Tick latency** | <1ms -- never slows your shell |
| **State** | `~/.shellquest/save.json` (atomic writes) |
| **Output** | stderr only -- never pollutes pipes |
| **Permissions** | `~/.shellquest/` is `0700` (owner-only) |
| **Concurrency** | Atomic write-then-rename prevents corruption |
| **Colors** | MUD-style rich inline formatting via `colored` crate |
| **Dependencies** | clap, colored, dirs, rand, serde, chrono |

---

## Contributing

PRs welcome! Some ideas:

- More monsters, loot, and events
- Achievement system
- Multiplayer leaderboards (shared server)
- `sq shop` -- spend gold on items
- `sq use <potion>` -- consume potions
- ASCII art boss encounters
- Sound effects via terminal bell

---

## License

MIT -- do whatever you want with it.

Made with Rust and vibes.
