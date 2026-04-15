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

*Your shell is the dungeon. Every command counts.*

[![Crates.io](https://img.shields.io/crates/v/shellquest?style=flat-square&color=orange)](https://crates.io/crates/shellquest)
[![Downloads](https://img.shields.io/crates/d/shellquest?style=flat-square&color=blue)](https://crates.io/crates/shellquest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Stars](https://img.shields.io/github/stars/duysqubix/shellquest?style=flat-square&color=yellow)](https://github.com/duysqubix/shellquest)

[Install](#install) · [Quick Start](#quick-start) · [Classes](#classes) · [Commands](#commands)

</div>

---

You type hundreds of commands a day. `ls`, `git push`, `cargo build`. Every one disappears into the void -- no reward, no acknowledgment, just a prompt waiting for the next one.

`shellquest` hooks into your shell and turns every command into a game event. XP, loot, combat, bosses. You don't do anything differently. You just stop losing progress.

```bash
~/projects $ git push
  🏆 Quest complete! You pushed your code to the realm! +21 XP +8 gold

~/projects $ cargo build
  💎 ★·.· The forge burns hot! You crafted: ★Vorpal Pointer★ (+18 Weapon) [Epic] ·.·★

~/projects $ bad_command
  🪤 You stumble on a trap! Took 3 damage. HP: 44/47

~/projects $ grep -r "TODO" .
  ⚔️  A Lint Wraith appears! You dodge! Riposte for 14 damage. Victory! +19 XP

~/projects $ vim README.md
  🧘 You enter a meditative state. HP restored: 47/47. +6 XP

~/projects $ docker stop db
  ☠️  BOSS BATTLE: The Memory Corruption looms from the void! 95 HP. The fight begins.
```

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
> Requires [Rust](https://rustup.rs) (cargo).

---

## Quick start

```bash
sq init       # pick your class and race
sq status     # view your character sheet
sq journal    # read the adventure log
```

That's it. Just use your terminal.

---

## What you get

- **5 classes** -- Wizard, Warrior, Rogue, Ranger, Necromancer
- **5 races** -- Human, Elf, Dwarf, Orc, Goblin
- **150 levels** with a scaling XP curve, titles from *Terminal Novice* to *Root Overlord*
- **15 subclasses** -- unlocked at prestige
- **130+ items** across 5 rarity tiers: Common (70%) down to Legendary (0.01%)
- **11 zones** determined by your current directory, each with unique flavor and danger scaling
- **5 named bosses**, spawning at 1 in 1,000 commands
- **Prestige system** -- reset at 150, keep your gear, go again stronger
- **Permadeath mode** -- optional; standard death costs 15% of your gold and resets current-level XP

---

## Classes

Each class earns +50% XP on affinity commands.

| Class | Affinity | Flavor |
|-------|----------|--------|
| Wizard | `python`, `node`, `ruby`, `vim`, `emacs`, `man` | Scholarly. Grimoire-keeper. Knows 47 ways to open a file. |
| Warrior | `cargo`, `make`, `cmake`, `gcc`, `ninja` | Compiler-whisperer. Builds things that actually run. |
| Rogue | `grep`, `rg`, `ssh`, `find`, `ls` | Lurks in pipes. Finds things that don't want to be found. |
| Ranger | `curl`, `wget`, `docker`, `kubectl`, `terraform` | Tames the cloud. Mostly. |
| Necromancer | `kill`, `pkill`, `rm`, `git`, `shred` | Raises and destroys. Often the same operation. |

---

## Bosses

Five named bosses roam the dungeon. They don't announce their arrival.

**The Kernel Panic** · **Lord of /dev/null** · **SIGKILL Supreme** · **The Infinite Loop** · **The Memory Corruption**

1 in 1,000 commands. d20 combat. No guarantees.

---

## Prestige

Level 150. Choose a subclass. Reset. Go again stronger.

What resets: level and XP.
What you keep: gold, gear, kills, inventory.
What you gain: +2 to all stats per prestige tier, +10 max HP per prestige tier, one of 15 subclasses.

Level 150. You can stop. You won't.

---

## Commands

| Command | Description |
|---------|-------------|
| `sq init` | Create your character |
| `sq status` / `sq stat` | View your character sheet (`--full` to include inventory) |
| `sq inventory` | Check gear and potions |
| `sq journal` | Last 20 adventure log entries |
| `sq shop` | Browse the shop (home directory only) |
| `sq buy <n>` | Buy item by number from `sq shop` list |
| `sq sell <n>` | Sell inventory item at the shop (home directory only) |
| `sq equip <name>` / `sq wear <name>` | Equip armor or ring from inventory |
| `sq wield <name>` | Wield a weapon from inventory |
| `sq remove <name>` / `sq unequip <name>` | Unequip weapon, armor, or ring → back to inventory |
| `sq drink <name>` | Drink a potion |
| `sq drop <name>` | Drop an item from inventory |
| `sq prestige` | Ascend at level 150 |
| `sq hook --shell zsh` | Print shell hook code |
| `sq update` | Update to latest via cargo |
| `sq reset` | Delete your character (permanent) |

---

## Contributing

PRs welcome. More monsters, more loot, more zones -- the dungeon is always hiring.

---

MIT -- do whatever you want with it.

Made with Rust and vibes.
