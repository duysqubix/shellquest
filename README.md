<div align="center">

```
                                                                      
                                                                      
        #             ###    ###                                      
        #               #      #                                  #   
        #               #      #                                  #   
 :###:  #:##:   ###     #      #     ## #  #   #   ###   :###:  ##### 
 #: .#  #  :#     :#    #      #    #   #  #   #     :#  #: .#    #   
 #:.    #   #  #   #    #      #    #   #  #   #  #   #  #:.      #   
 .###:  #   #  #####    #      #    #   #  #   #  #####  .###:    #   
    :#  #   #  #        #      #    #   #  #   #  #         :#    #   
 #. :#  #   #      #    #.     #.   #   #  #:  #      #  #. :#    #.  
 :###:  #   #   ###:    :##    :##   ## #  :##:#   ###:  :###:    :## 
                                        #                             
                                        #                             
                                        #                             
```

***Your shell is the dungeon. Every command is a turn.***

[![Crates.io](https://img.shields.io/crates/v/shellquest?style=flat-square&color=orange)](https://crates.io/crates/shellquest)
[![Downloads](https://img.shields.io/crates/d/shellquest?style=flat-square&color=blue)](https://crates.io/crates/shellquest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Stars](https://img.shields.io/github/stars/duysqubix/shellquest?style=flat-square&color=yellow)](https://github.com/duysqubix/shellquest)

[Install](#install) · [Quick Start](#quick-start) · [Classes](#classes--signatures) · [Arena](#arena) · [Commands](#commands)

</div>

---

You spend your day in the terminal. Hundreds of commands. `ls`, `git push`, `cargo build`. They vanish into the void — no XP, no loot, no glory.

`shellquest` makes them count.

Hook your shell **once**. From then on, every command is a swing of the blade. `git commit` forges relics. `grep` flushes monsters from the brush. `docker stop` summons world bosses. `vim` heals you. Bad commands? Traps. Wrong directory? Welcome to the **Abyss of `node_modules`** — danger 5, here be dragons.

You don't change how you work. You just stop losing progress.

```bash
~/projects $ git push
  🏆 The siege succeeds! Code pushed to the realm! +21 XP, +8 gold

~/projects $ cargo build
  💎 ★·.· Your hammer strikes true! Forged: ★Vorpal Pointer★ (+18) ·.·★

~/code $ grep -r "TODO" .
  ⚔️  CRITICAL! Your dagger finds a vital point on the Null Pointer Imp for 24 damage! ✦ shadow strike ✦

~/big-monorepo $ docker stop db
  ⚠️  WORLD BOSS: THE MEMORY CORRUPTION HAS APPEARED! 95 HP. The fight begins.

~ $ vim README.md
  🧘 You vanish into the text. Wounds close. +6 HP, +3 XP. HP: 47/47

~/projects $ bad_command
  🪤 A crude mechanism bites! -4 HP. Unfair.

~/projects $ git commit -m "fix typo"
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  LEVEL UP! Your might grows! Level 27. Title: Pipe Weaver
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

That's a real terminal session. No screenshots, no animations, no nonsense. Just your shell with a 150-level RPG bolted on.

---

## Install

```bash
cargo install shellquest
```

Then add the shell hook:

```bash
sq hook --shell zsh >> ~/.zshrc   # or bash, fish
source ~/.zshrc
```

<details>
<summary>One-liner (auto-installs hook)</summary>

```bash
curl -fsSL https://raw.githubusercontent.com/duysqubix/shellquest/main/install.sh | bash
```

Detects your shell and wires everything up automatically.

</details>

<details>
<summary>From source</summary>

```bash
git clone https://github.com/duysqubix/shellquest.git
cd shellquest
cargo install --path .
sq hook --shell zsh >> ~/.zshrc
```

</details>

> [!NOTE]
> Requires [Rust](https://rustup.rs) (cargo). Hook is idempotent — installing twice does nothing weird.

---

## Quick Start

```bash
sq init           # roll your hero — pick class, race, permadeath
sq status         # view your character sheet and inventory
sq journal        # read the last 20 adventure log entries
sq help arena     # learn how to play the arena
```

Then just… use your terminal. The game plays itself around you.

---

## What You Get

A full, code-backed RPG hiding inside your prompt:

- **5 classes × 5 races × 15 prestige subclasses** — 375 unique character builds
- **150 levels**, scaling XP curve, **15 progression titles** from *Terminal Novice* to *Root Overlord*
- **132 items** across 5 rarity tiers — Common (70%) · Uncommon (25%) · Rare (4%) · Epic (0.99%) · Legendary (0.01%)
- **11 zones** mapped to your `$PWD` — `node_modules` is the Abyss (danger 5), `/tmp` is the Wasteland (danger 3), `$HOME` is safe. Danger scales loot and XP.
- **30+ event handlers** — `git commit` crafts, `kill` banishes, `vim` meditates, `sudo` surges, `grep` scries, `man` reads ancient tomes, and every command has its own flavor
- **Attrition combat** — d20-style rolls; tough fights chip HP for hours
- **Elite encounters** — rare *Enraged* variants hit harder and drop richer loot
- **INT-scaled critical hits** — smarter heroes crit more often (threshold = `max(15, 20 − INT/4)`)
- **5 class signature passives** — each class has its own combat trick (see [Classes](#classes--signatures))
- **5 named world bosses** spawning at **1 in 1,000** commands. No warning. No mercy.
- **5-tier arena gauntlet** — transactional risk/reward combat with cash-out mechanics
- **Daily-refreshing shop** — 6 items, rotates at UTC midnight, home directory only
- **Passive home healing** — recover 1 HP per 30s while at `$HOME` (capped at 30 min stored)
- **Prestige system** — reset at 150, keep your gear, choose a subclass, go again stronger
- **Permadeath mode** — opt in at character creation. You get an **eulogy**. The save file is deleted. *Roll a new hero, adventurer.*
- **Sage update notifier** — an in-world herald checks crates.io and announces new versions in-character

---

## Classes & Signatures

Each class earns **+50% XP** on its affinity commands and unlocks a unique **signature passive** in combat.

| Class | Affinity Commands | Signature | Flavor |
|-------|-------------------|-----------|--------|
| 🧙 **Wizard** | `python`, `node`, `ruby`, `vim`, `emacs`, `man` | **Arcane Burn** — bonus damage scaling with INT | Grimoire-keeper. Knows 47 ways to open a file. |
| ⚔️ **Warrior** | `cargo`, `make`, `cmake`, `gcc`, `ninja` | **Battle Frenzy** — bonus damage when below ⅓ HP | Compiler-whisperer. Builds things that actually run. |
| 🗡️ **Rogue** | `grep`, `rg`, `ssh`, `find`, `ls` | **Shadow Strike** — survives the fumble; nat 1s still land | Lurks in pipes. Finds things that don't want to be found. |
| 🏹 **Ranger** | `curl`, `wget`, `docker`, `kubectl`, `terraform` | **Mark Prey** — bonus damage on the opening strike | Tames the cloud. Mostly. |
| 💀 **Necromancer** | `kill`, `pkill`, `rm`, `git`, `shred` | **Soul Drain** — heal HP every time you land a kill | Raises and destroys. Often the same operation. |

All combat messages — combat, craft, loot, meditation, death — are **rewritten per class**. A Wizard's victories sound like spellwork. A Necromancer's failed commands have *ironic backlash*. Your hero has a voice.

---

## Races

| Race | STR | DEX | INT |
|------|:---:|:---:|:---:|
| Human | +1 | +1 | +1 |
| Elf | +0 | +2 | +2 |
| Dwarf | +3 | +0 | +1 |
| Orc | +4 | +1 | −1 |
| Goblin | −1 | +3 | +1 |

Stack a race onto your class. Necromancer Goblin? +3 DEX +1 INT on top of `6/6/18`. The math is real.

---

## Bosses

Five world bosses prowl the dungeon. They spawn at **1 in 1,000** commands. They do not announce their arrival.

| Boss | HP | Atk | XP | Gold |
|------|:--:|:---:|:--:|:----:|
| ☠️ The Kernel Panic | 100 | 22 | 900 | 350 |
| 🌀 The Infinite Loop | 110 | 15 | 950 | 300 |
| ⚡ SIGKILL Supreme | 90 | 25 | 800 | 320 |
| 💀 The Memory Corruption | 95 | 20 | 850 | 310 |
| 🕳️ Lord of /dev/null | 85 | 18 | 700 | 280 |

Boss combat is d20-style with INT-scaled crits. Loot is Rare/Epic/Legendary only — no Common scraps from a god. Stale bosses flee after 24 hours; if you ignore them, they leave.

---

## Arena

The **Arena** is an interactive combat gauntlet. Five tiers, each with its own gate, fee, and reward curve. **Risk/reward**: after every round you choose to **Continue** for deeper rewards or **Cash Out** to bank what you've earned. Get KO'd and you lose everything from this run — including the entry fee.

| Tier | Rounds | Unlock | Entry Fee |
|------|:------:|--------|-----------|
| **The Pit** | 5 | — | `max(40, lvl×12)` |
| **The Gauntlet** | 10 | lvl 25 *or* prestige 1 | `max(100, lvl×18 + p×50)` |
| **The Colosseum** | 15 | lvl 60 *or* prestige 1 | `max(300, lvl×28 + p×150)` |
| **The Abyssal Arena** | 25 | lvl 100 *or* prestige 2 | `max(800, lvl×40 + p×250)` |
| 👑 **Godslayer's Court** | 50 | lvl 150 **and** prestige 3 | `max(2500, lvl×60 + p×400)` |

- **25 named enemies** — Segmentation Fault Sprite, Buffer Overflow Beast, Null Pointer Imp, Deadlock Demon, Race Condition Raider, and more
- **Chest milestones** drop guaranteed loot at preset rounds
- **Inventory full?** Chest loot converts to half its sell value in gold
- **KO penalty** — fee is consumed, HP drops to 25% of max, journal logs your shameful exit
- **Atomic runs** — `Ctrl+C` mid-arena? Full rollback. Nothing was saved.
- **Crown** — clear Godslayer's Court for the only crown in the game

Requires an interactive TTY. No piping `yes` into the arena, hero.

```bash
sq arena       # enter the gauntlet
sq help arena  # full rules
```

---

## Prestige & Titles

Hit **level 150** and you can prestige. You don't lose much — you become more.

| What | Outcome |
|------|---------|
| **Resets** | Level → 1, XP → 0 |
| **Keeps** | Gold, gear, inventory, kills, journal, shop state |
| **Gains** | +2 STR/DEX/INT and +10 max HP **per prestige tier** |
| **Unlocks** | One of **15 class subclasses** (3 per class) |

Titles ladder all the way up:

> *Terminal Novice → Shell Apprentice → Command Adept → Pipe Weaver → Script Sorcerer → Kernel Knight → Daemon Slayer → Binary Sage → System Architect → Process Overlord → Thread Titan → Memory Monarch → Stack Sovereign → Root Demigod → **Root Overlord***

Prestige titles stack on top: *Prestigious* · *Exalted* · *Transcendent* · *Mythical* · *Godlike*.

Level 150. You can stop. You won't.

---

## Commands

| Command | What |
|---------|------|
| `sq help [topic]` | Built-in manual — `sq help`, `sq help arena`, `sq help prestige`, etc. |
| `sq init` | Roll a character — class, race, permadeath choice |
| `sq status` / `sq stat` | Character sheet **and** inventory |
| `sq inventory` / `sq inv` | Inventory only |
| `sq journal` | Last 20 adventure log entries |
| `sq equip <name>` / `sq wear <name>` | Equip armor or ring |
| `sq wield <name>` | Wield a weapon |
| `sq remove <name>` / `sq unequip <name>` | Send equipped gear back to inventory |
| `sq drink <name>` | Drink a potion |
| `sq drop <name>` | Permanently drop an item |
| 🏠 `sq shop` | Browse the shop *(home directory only)* |
| 🏠 `sq buy <n>` | Buy item by number from the shop |
| 🏠 `sq sell <n>` | Sell inventory item for half its buy price |
| `sq arena` | Enter the 5-tier combat gauntlet |
| `sq prestige` | Ascend at level 150 |
| `sq hook --shell zsh` | Print shell hook code (zsh / bash / fish) |
| `sq update` | Update via cargo |
| `sq reset` | Delete your character (permanent) |
| `sq tournament` | Deprecated alias for `sq arena` |
| `sq tick --cmd ...` | Internal — called by the shell hook on every command |

> 🏠 = must be in your `$HOME` directory. Shops don't follow you into the dungeon.

---

## Contributing

PRs welcome. More monsters, more zones, more loot, more flavor — the dungeon is always hiring.

The codebase is small and approachable Rust: `src/events.rs` is the game loop, `src/messages.rs` is the per-class flavor text, `src/loot.rs` is the item tables. Read [AGENTS.md](./AGENTS.md) for an architecture overview.

---

MIT — do whatever you want with it.

Made with Rust and vibes.
