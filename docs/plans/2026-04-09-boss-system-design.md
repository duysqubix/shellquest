# Boss System Design

**Date:** 2026-04-09  
**Status:** Approved — ready for implementation

---

## Overview

Passive boss encounters that persist across terminal commands. A boss spawns as a rare world event, lives in `save.json`, and takes damage every tick until it dies or the player does. Fully passive — no extra commands, no menu. Just keep working.

---

## Spawn Mechanics

- **Rate:** ~0.1% per tick (`gen_ratio(1, 1000)`) — rarer than Epic loot, less rare than Legendary
- **Condition:** Only one boss active at a time; spawn roll is skipped if `active_boss.is_some()`
- **Timing:** Spawn check runs at the end of each tick, after all normal events resolve
- **Stale boss:** If `spawned_at` is older than 24 hours, the boss auto-flees on the next tick with a message

---

## Data Model

New `Boss` struct added to `src/boss.rs`:

```rust
pub struct Boss {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub xp_reward: u32,
    pub gold_reward: u32,
    pub spawned_at: DateTime<Utc>,
}
```

New field on `GameState` in `src/state.rs`:

```rust
#[serde(default)]
pub active_boss: Option<Boss>,
```

---

## Boss Roster

Five bosses, one selected at random on spawn:

| Name | HP | Attack | Flavor |
|---|---|---|---|
| The Kernel Panic | 100 | 22 | *The system itself turns against you* |
| Lord of /dev/null | 85 | 18 | *It consumes everything, returns nothing* |
| SIGKILL Supreme | 90 | 25 | *No handler. No mercy.* |
| The Infinite Loop | 110 | 15 | *It never stops. Neither can you.* |
| The Memory Corruption | 95 | 20 | *Every value it touches becomes wrong* |

At ~8 average damage per hit, fights last approximately 10–14 commands.

---

## Per-Tick Combat

Runs every tick when `active_boss.is_some()`, after normal events:

1. **Player attacks boss:** `hit_roll (d20) + player_attack_power > 10`  
   - Hit → `damage = rng.gen_range(player_power / 2 ..= player_power)`, `boss.hp -= damage`
2. **Check boss death:** if `boss.hp <= 0` → call `boss_victory()`, clear boss, return
3. **Boss attacks player:** `dodge_roll (d20) > 10 + player_defense`  
   - Hit → `damage = (boss.attack - player_defense).max(1)`, `player.take_damage(damage)`
4. **Check player death:** if player died → call `boss_flees()`, clear boss, no reward

Normal events (XP, loot, random encounters) still fire as usual. The boss is an additional combat layer per tick.

---

## Rewards on Kill

- **XP:** `rng.gen_range(500..=1000)`
- **Gold:** `rng.gen_range(200..=400)`
- **Loot:** One item from a modified table — no Commons:

| Rarity | Normal | Boss |
|---|---|---|
| Common | 70% | 0% |
| Uncommon | 25% | 40% |
| Rare | 4% | 47% |
| Epic | 0.99% | 10% |
| Legendary | 0.01% | 3% |

Implemented as `roll_boss_loot()` in `src/loot.rs`.

---

## Death & Flee

- **Player dies:** Boss flees instantly. Message: *"[Boss name] laughs as you fall... and vanishes into the void."* No reward.
- **24h timeout:** Boss auto-flees. Message: *"[Boss name] grows bored waiting and retreats. It will return."* No reward.

---

## Display

| Event | Output |
|---|---|
| Boss spawns | Multi-line dramatic announcement with boss flavor text |
| Each tick (player hits) | `💀 [BOSS] [name]! You strike for X damage! (HP: Y/Z)` |
| Each tick (boss hits) | `   It retaliates — took X damage. HP: A/B` |
| Each tick (miss) | Brief "circling" line |
| Boss dies | Victory fanfare, reward printout |
| Boss flees (death) | Shame message, no reward |
| Boss flees (timeout) | Warning that it will return |

All output via `eprintln!` (stderr), consistent with the rest of the game.

---

## Code Surface

| File | Change |
|---|---|
| `src/boss.rs` | New module: `Boss` struct, roster constant, `spawn_boss()`, `tick_boss()`, `boss_victory()`, `boss_flees()` |
| `src/loot.rs` | Add `roll_boss_loot()` with modified rarity weights |
| `src/state.rs` | Add `active_boss: Option<Boss>` with `#[serde(default)]` |
| `src/display.rs` | Add `print_boss_spawn()`, `print_boss_tick()`, `print_boss_victory()`, `print_boss_flee()` |
| `src/events.rs` | Call `boss::maybe_spawn()` and `boss::tick_boss()` from `tick()` |
| `src/main.rs` | Declare `mod boss` |
| `AGENTS.md` / `src/AGENTS.md` | Document new module and boss mechanic |

---

## What Is Not In Scope

- Player-triggered boss commands (`sq fight`, etc.) — fully passive
- Multiple simultaneous bosses
- Boss difficulty scaling by player level (may be a future iteration)
- Boss-specific unique loot (using shared loot table with modified weights)
