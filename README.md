# shellquest (sq)

A passive RPG that lives in your terminal. Your shell is the dungeon.

Every command you run has a chance to trigger an encounter. Your character gains XP, finds loot, fights monsters, and levels up -- all while you're just doing your normal work.

## Install

One-liner (requires Rust and git):

```bash
curl -fsSL https://raw.githubusercontent.com/USER/shellquest/main/install.sh | bash
```

This will clone the repo, build the binary, install `sq` to `~/.cargo/bin/`, and add the shell hook automatically.

### Manual install

```bash
git clone https://github.com/USER/shellquest.git
cd shellquest
cargo install --path .
sq hook --shell zsh >> ~/.zshrc   # or bash/fish
```

### Requirements

- [Rust](https://rustup.rs) (cargo)
- git

## Quick Start

```bash
# 1. Reload your shell (or restart terminal)
source ~/.zshrc

# 2. Create your character
sq init

# 3. Just use your terminal -- events happen automatically!

# Check your stats anytime
sq status
sq inventory
sq journal
```

## How It Works

A shell hook runs `sq tick` in the background after every command you type. Each tick has a chance to trigger events based on what you just did:

| Command | Event |
|---------|-------|
| `git commit` | Craft XP from committing to the archives |
| `git push` | Quest completion with XP + gold |
| `cargo build` / `npm build` | Forge -- chance to craft weapons & armor |
| `docker build` | Container forge -- high-quality loot |
| `docker compose` | Orchestration ritual -- big XP + gold |
| `rm` / `del` | Angry spirits attack |
| `cat` / `less` | Befriend a familiar (heals HP) |
| `sudo` | Power surge -- raw energy XP |
| `vim` / `nvim` / `emacs` | Editor meditation -- heals + XP |
| `kill` / `pkill` | Banish a process -- combat XP + gold |
| `grep` / `rg` | Scrying -- find hidden loot |
| `tar` / `zip` | Open a treasure chest |
| `man` / `tldr` | Study ancient tomes -- knowledge XP |
| `ssh` / `curl` | Open a portal to remote realms |
| Failed commands | Traps -- take damage |
| Everything else | ~15% random encounter chance |

## Character System

- **5 Classes**: Wizard, Warrior, Rogue, Ranger, Necromancer
- **5 Races**: Human, Elf, Dwarf, Orc, Goblin
- **Stats**: STR, DEX, INT -- affect combat and defense
- **150 Levels** with scaling XP curve
- **15 Titles** from Terminal Novice to Root Overlord

## Loot

80+ dev-themed items across 5 rarity tiers with distinct visual styling:

- **Common** -- plain white
- **Uncommon** -- gray with `~` marker
- **Rare** -- green with `~~` borders
- **Epic** -- purple with star decorators
- **Legendary** -- gold boxed frame

Items include weapons (Sword of Regex, Excalibash, Vorpal Pointer), armor (Hoodie of Comfort, Warplate of Kubernetes), rings (The One Ring of SSH Keys), and potions (Potion of Coffee, Phoenix Elixir of Hot Reload).

## Zones

Your current directory determines the biome:

| Path | Zone | Danger |
|------|------|--------|
| `~` | Home Village | Low |
| `/tmp` | The Wasteland | Medium |
| `/dev` | Device Caverns | High |
| `node_modules` | The Abyss | Extreme |
| `src` / `lib` | Source Sanctum | Medium |
| `.git` | Time Vaults | Medium |
| `target` / `build` | The Forge | Medium |

Higher danger = tougher monsters but better loot.

## Prestige

At level 150, you can prestige:

```bash
sq prestige
```

This resets you to level 1 but you gain:
- A **subclass** with unique stat bonuses (3 per class, 15 total)
- **+2 to all stats** per prestige tier
- **+10 max HP** per prestige tier
- You **keep** your gold, gear, kills, and inventory

Prestige titles stack: Prestigious, Exalted, Transcendent, Mythical, Godlike.

## Commands

```
sq init        Create a new character
sq status      View your character sheet
sq inventory   Check your gear
sq journal     Adventure log
sq prestige    Reset to level 1 with a subclass (requires level 150)
sq hook        Print shell hook code
sq reset       Delete your character
```

## Design

- Single Rust binary, ~1.5MB
- Tick completes in <1ms -- never slows your shell
- State saved to `~/.shellquest/save.json`
- Atomic writes prevent corruption from concurrent ticks
- All output goes to stderr so it never pollutes stdout pipes
- MUD-style colorized output with rich inline formatting

## License

MIT
