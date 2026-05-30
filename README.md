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

[Install](#install) · [Quick Start](#quick-start) · [Classes](#classes--signatures) · [Combat](#combat) · [Arena](#arena) · [Commands](#commands)

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
  💎 ★·.· Your hammer strikes true! Forged: ★Logic-Lash★ (+14) [Rare] ·.·★

~/code $ grep -r "TODO" .
  ⚔️  CRITICAL! Your dagger finds a vital point on the Pointer Panther for 24 damage! ✦ shadow strike ✦

~/big-monorepo $ docker stop db
  ⚠️  WORLD BOSS: THE MEMORY CORRUPTION HAS APPEARED! 95 HP. The fight begins.

~ $ vim README.md
  🧘 You vanish into the text. Wounds close. +6 HP, +3 XP. HP: 47/47

~/projects $ bad_command
  🪤 A crude mechanism bites! -9 HP. Unfair.

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
sq status         # view your character sheet (ATK/DEF + inventory)
sq journal        # read the last 20 adventure log entries
sq help arena     # learn how to play the arena
```

Then just… use your terminal. The game plays itself around you.

---

## What You Get

A full, code-backed RPG hiding inside your prompt:

- **5 classes × 5 races × 15 prestige subclasses** — 375 unique character builds
- **150 levels**, scaling XP curve, **15 progression titles** from *Terminal Novice* to *Root Overlord*
- **160 items** across 5 rarity tiers — Common (70%) · Uncommon (25%) · Rare (4%) · Epic (0.99%) · Legendary (0.01%). 8 items per slot, per rarity.
- **40-monster bestiary** across 5 difficulty tiers — *Vermin → Bruiser → Hunter → Horror → Boss-adjacent* — that spawn by zone danger, so `$HOME` sees Symlink Slugs and `node_modules` sees Deadlock Dragons
- **33 zones** mapped to your `$PWD` — from safe `$HOME`, Desktop, Documents, and media galleries to `/proc`, `/sys`, `.ssh`, secrets, caches, build artifacts, backups, trash, and the Abyss of `node_modules`. Danger scales which monsters spawn, plus loot and XP.
- **30+ event handlers** — `git commit` crafts, `kill` banishes, `vim` meditates, `sudo` surges, `grep` scries, `man` reads ancient tomes, and every command has its own flavor
- **HP-pool multi-turn combat** — every monster has an HP bar; fights resolve over multiple swings inside a single command, with crits and class signatures firing on every turn
- **Elite encounters** — rare *Enraged* variants get +60% attack, +50% HP, and 2× XP
- **INT-scaled critical hits** — smarter heroes crit more often (threshold = `max(13, 20 − INT/8)`)
- **5 class signature passives** — each class has its own combat trick (see [Classes](#classes--signatures))
- **5 named world bosses** spawning at **1 in 500** commands. No warning. No mercy.
- **5-tier arena gauntlet** — interactive, paced, wave-escalating risk/reward combat with cash-out mechanics (see [Arena](#arena))
- **`sq enchant`** — a gold sink that pours your hoard into permanent +power on the gear you actually wear (up to +5)
- **`sq identify`** — full item cards: base vs effective power, enchant, prices, what each stat affects
- **Daily-refreshing shop** — 6 items, rotates at UTC midnight, home directory only
- **Passive home healing** — recover 1 HP per 30s while at `$HOME` (capped at 30 min stored)
- **Prestige system** — reset at 150, keep your gear, choose a subclass, go again stronger
- **Permadeath mode** — opt in at character creation. You get an **eulogy**. The save file is deleted. *Roll a new hero, adventurer.*
- **Sage update notifier** — an in-world herald checks crates.io and announces new versions in-character

---

## Classes & Signatures

Each class earns **+50% XP** on its affinity commands and unlocks a unique **signature passive** that fires on every combat turn.

| Class | Affinity Commands | Signature | Flavor |
|-------|-------------------|-----------|--------|
| 🧙 **Wizard** | `python`, `node`, `ruby`, `vim`, `emacs`, `man` | **Arcane Burn** — always-on bonus damage of `+INT/5` | Grimoire-keeper. Knows 47 ways to open a file. |
| ⚔️ **Warrior** | `cargo`, `make`, `cmake`, `gcc`, `ninja` | **Battle Frenzy** — `+STR/4` damage while below 60% HP | Compiler-whisperer. Builds things that actually run. |
| 🗡️ **Rogue** | `grep`, `rg`, `ssh`, `find`, `ls` | **Shadow Strike** — survives the fumble; nat 1s still land | Lurks in pipes. Finds things that don't want to be found. |
| 🏹 **Ranger** | `curl`, `wget`, `docker`, `kubectl`, `terraform` | **Mark Prey** — `+INT/3` damage on the opening strike | Tames the cloud. Mostly. |
| 💀 **Necromancer** | `kill`, `pkill`, `rm`, `git`, `shred` | **Soul Drain** — heal `INT/3` HP every time you land a kill | Raises and destroys. Often the same operation. |

All combat messages — combat, craft, loot, meditation, death — are **rewritten per class**. A Wizard's victories sound like spellwork. A Necromancer's failed commands have *ironic backlash*. Your hero has a voice.

Base stats (STR / DEX / INT): Wizard `6/8/16` · Warrior `16/8/6` · Rogue `8/16/6` · Ranger `10/14/6` · Necromancer `6/6/18`.

---

## Races

| Race | STR | DEX | INT |
|------|:---:|:---:|:---:|
| Human | +1 | +1 | +1 |
| Elf | +0 | +2 | +2 |
| Dwarf | +3 | +0 | +1 |
| Orc | +3 | +1 | −1 |
| Goblin | +0 | +3 | +1 |

Stack a race onto your class. Necromancer Goblin? +3 DEX, +1 INT on top of `6/6/18`. The math is real. No single race bonus exceeds +3 — the spread is wide enough to matter, tight enough to stay fair.

---

## Combat

Every monster has an **HP pool**, and combat is a **multi-turn loop** that resolves inside a single command: you swing, the monster swings back if it's still standing, repeat until someone hits zero (or 30 turns pass and it's a draw). You see a single summary line — but the outcome is a real exchange, not a coin flip.

- **d20 to hit, d20 to dodge** — your attack power and defense move the thresholds
- **Crits** land per-turn at `max(13, 20 − INT/8)` for 2× damage — INT builds get more reliable burst
- **Class signatures fire every turn** — a high-INT Wizard out-damages a low-INT one against anything with an HP bar
- **Monsters spawn to fit the zone** — danger 1 (`$HOME`) is Vermin; danger 5 (`node_modules`) is Horror and Boss-adjacent. A fresh level-1 character won't meet a Deadlock Dragon in their home directory.
- **Traps scale with you** — a bad command costs **3–6% of your max HP**, so typos stay scary from level 1 to 150
- **Death is a setback, not a wipe** — normal-mode death costs gold and **half** your in-progress XP (permadeath is still hardcore: eulogy, save deleted)

### The Bestiary

40 monsters, 8 per tier, spawning by zone danger:

| Tier | Vibe | Example | Zone danger |
|------|------|---------|:-----------:|
| **Vermin** | Small filesystem creatures, one-shot kills | Symlink Slug, Cache Cricket | 1–2 |
| **Bruiser** | Process spirits that fight back | Daemon Duelist, Cron-Job Crusader | 2–3 |
| **Hunter** | Memory dwellers with predator aesthetic | Pointer Panther, Segfault Shark | 3–4 |
| **Horror** | Kernel terrors, serious threats | Deadlock Dragon, Race-Condition Reaper | 4–5 |
| **Boss-adjacent** | Architectural entities you flee from | The Legacy-Code Lich, Cloud-Native Chimera | 5+ |

---

## Bosses

Five world bosses prowl the dungeon. They spawn at **1 in 500** commands. They do not announce their arrival. (Separate roster from the bestiary above.)

| Boss | HP | Atk | XP | Gold |
|------|:--:|:---:|:--:|:----:|
| ☠️ The Kernel Panic | 100 | 22 | 900 | 350 |
| 🌀 The Infinite Loop | 110 | 15 | 950 | 300 |
| ⚡ SIGKILL Supreme | 90 | 25 | 800 | 320 |
| 💀 The Memory Corruption | 95 | 20 | 850 | 310 |
| 🕳️ Lord of /dev/null | 85 | 18 | 700 | 280 |

Boss combat is the same HP-pool, INT-scaled-crit system as everything else. Loot is Rare/Epic/Legendary only — no Common scraps from a god. Stale bosses flee after 24 hours; if you ignore them, they leave.

---

## Arena

The **Arena** is an interactive combat gauntlet. Five tiers, each with its own gate, fee, and reward curve. **Risk/reward**: after every won round you see exactly what's in the pot and choose to **Continue** for deeper rewards or **Cash Out** to bank it. Get KO'd and you lose everything from this run — including the entry fee.

| Tier | Rounds | Unlock | Entry Fee |
|------|:------:|--------|-----------|
| **The Pit** | 5 | — | `max(40, lvl×12, gold/10)` |
| **The Gauntlet** | 10 | lvl 25 *or* prestige 1 | `max(100, lvl×18 + p×50, gold/8)` |
| **The Colosseum** | 15 | lvl 60 *or* prestige 1 | `max(300, lvl×28 + p×150, gold/6)` |
| **The Abyssal Arena** | 25 | lvl 100 *or* prestige 2 | `max(800, lvl×40 + p×250, gold/5)` |
| 👑 **Godslayer's Court** | 50 | lvl 150 **and** prestige 3 | `max(2500, lvl×60 + p×400, gold/4)` |

### Paced, wave-escalating combat

Combat plays out **line by line, 1.5 seconds apart** — you watch the HP tick, feel the back-and-forth, and decide with your pulse up. Each round maps to a **difficulty wave**: enemy HP, accuracy, and crit chance all step up as you push deeper. Round 1 is a warmup. Round 10 is something you flex about clearing.

| Rounds | Wave | Enemy HP | Enemy lands | Enemy crit |
|:------:|------|:--------:|:-----------:|:----------:|
| 1–3 | Warmup | ×1.0 | ~10% | 0% |
| 4–6 | Building | ×1.4 | ~15% | 3% |
| 7–9 | Tense | ×2.0 | ~25% | 6% |
| 10–15 | Boss-tier | ×3.0 | ~30% | 8% |
| 16–25 | Brutal | ×4.5 | ~40% | 12% |
| 26–40 | Nightmare | ×6.5 | ~50% | 18% |
| 41+ | Endgame | ×9.0 | ~65% | 22% |

- **Misses are real** — your DEX drives accuracy; low-DEX heroes whiff, high-DEX heroes dodge
- **Enemies crit you** in later waves — a `CRITICAL!` for double damage can swing a marginal fight into a KO
- **Banked-gold preview** — after each round you see `💰 Cash Out now: +N gold, +N XP` before deciding
- **Chest milestones** drop guaranteed loot at preset rounds; inventory full? Loot converts to half its sell value in gold
- **KO penalty** — fee is consumed, HP drops to 25% of max, journal logs your shameful exit
- **Atomic runs** — `Ctrl+C` mid-arena? Full rollback. Nothing was saved.
- **Crown** — clear Godslayer's Court for the only crown in the game

In a hurry? `SQ_NO_PACING=1 sq arena` drops the 1.5s beats and resolves instantly. Requires an interactive TTY either way — no piping `yes` into the arena, hero.

```bash
sq arena       # enter the gauntlet
sq help arena  # full rules
```

---

## Enchanting

Your gold has somewhere to go now. Stand in your home directory (**Wizards can enchant anywhere** — the arcane workbench lives between their ears), point at an equipped item, and burn gold for permanent **+1 power, up to +5**.

```bash
$ cd ~ && sq enchant scythe
✨ Scythe of Segfault is now [Enchanted +1] (cost: 260 gold).
   Gold: 5000 → 4740
```

Costs scale steeply — each level is the item's buy price × the next level, so a full +5 costs **15× the item's buy price**. The `[Enchanted +N]` tag escalates in color as you invest, ending in a rainbow `[Enchanted +5]` trophy. Potions can't be enchanted; the magic wouldn't take. Enchant-then-sell is always a net loss — this is a commitment, not an exploit loop.

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
| `sq status` / `sq stat` | Character sheet with ATK/DEF gear breakdown **and** inventory |
| `sq inventory` / `sq inv` | Inventory only, grouped by slot and sorted best-first |
| `sq journal` | Last 20 adventure log entries |
| `sq identify <name>` / `sq id <name>` | Full item card — base vs effective power, enchant, prices, source (read-only, anywhere) |
| `sq equip <name>` / `sq wear <name>` | Equip armor or ring |
| `sq wield <name>` | Wield a weapon |
| `sq remove <name>` / `sq unequip <name>` | Send equipped gear back to inventory |
| `sq drink <name>` | Drink a potion |
| `sq drop <name>` | Permanently drop an item |
| 🏠 `sq shop` | Browse the shop |
| 🏠 `sq buy <n>` | Buy item by number from the shop |
| 🏠 `sq sell <n\|name>` | Sell one item; **`sq sell junk`** sweeps all Common + Uncommon at once |
| 🏠 `sq enchant <name>` | Burn gold for +power on equipped gear *(Wizards: anywhere)* |
| `sq arena` | Enter the 5-tier combat gauntlet |
| `sq prestige` | Ascend at level 150 |
| `sq hook --shell zsh` | Print shell hook code (zsh / bash / fish); `--install` writes it for you |
| `sq update` | Update via cargo |
| `sq reset` | Delete your character (permanent) |
| `sq tournament` | Deprecated — use `sq arena` |
| `sq tick --cmd ...` | Internal — called by the shell hook on every command |

> 🏠 = must be in your `$HOME` directory. Shops don't follow you into the dungeon. (Wizards enchant anywhere.)

---

## Contributing

PRs welcome. More monsters, more zones, more loot, more flavor — the dungeon is always hiring.

The codebase is approachable Rust: `src/events.rs` is the overworld game loop and combat, `src/arena.rs` is the gauntlet, `src/messages.rs` is the per-class flavor text, `src/loot.rs` is the item tables, `src/character.rs` is the data model. Balance changes are validated empirically by the simulator in `dev-tools/balance-sim/`. Read [AGENTS.md](./AGENTS.md) for an architecture overview, and `release-notes/` for the human-readable history of every version.

---

MIT — do whatever you want with it.

Made with Rust and vibes.
