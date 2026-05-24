# shellquest balance analysis

> **Status**: v2 — incorporates designer constraints + corrections from Oracle review
> **Codebase**: v1.17.0 (post sq-sell-junk shipment)
> **Date**: 2026-05-23
> **Method**: 10 parallel research agents + Oracle synthesis pass + direct source verification
> **Scope**: diagnosis only — recommended changes listed with risk ratings but no code edits performed

## TL;DR

The game has a **difficulty bathtub**: brutal at L1-10, trivially safe by L30. There's no middle window where the player feels skilled-and-challenged. The HP-via-level accumulator (`+5/level`) silently saturates incoming damage by level 30-40 against most monsters.

The **loot ladder is malformed** in two ways: the Common→Legendary effective power gap is ~160× (industry: ~5-10×), and the regular-tick Legendary drop rate is 0.01% — though boss drops (3% Legendary, gated by 1/1000 spawn) are the real intended ladder, this is undocumented and undertuned. Casual play surfaces a Legendary roughly **every 77 days** of active terminal use, which is slow even for a years-long journey.

**Gold is now over-abundant** post-v1.17.0 because `sq sell junk` removed the friction that was rate-limiting Common+Uncommon → gold conversion. Arena entry fees are the only self-regulating sink, and they only meaningfully tax wealthy late-game players. L20-80 mid-game players accumulate gold with nothing to spend it on.

The **bestiary is AI-generated placeholder content** (designer confirmed) with no zone gating, no HP pools, and uniform random selection. A renaming pass is an opportunity to fix the difficulty bathtub at the same time — tiered monsters with HP pools and zone-appropriate spawn pools.

### Top 3 actions (all 5 open questions now locked — see §8)

1. **Add gear enchantment as a gold sink**: `sq enchant <item>` adds +1 power up to +5 per item, cost = `item_price × (current+1)`. Wizard class can enchant anywhere; others in `$HOME` only. Visual `[Enchanted +N]` tag escalates green → cyan → blue → magenta → rainbow at max. Solves the L20-80 gold-meaningless problem, adds player agency, ties into the rarity-ladder rebalance.
2. **Tiered monster redesign (5 tiers, HP pools, zone-gated, constant-tension model)**: solves the L1-instakill risk AND the L30+ trivialization in one pass. 40 monsters across Vermin/Bruiser/Hunter/Horror/Boss-adjacent tiers; HP pools added for Bruiser+ tiers so the existing crit/signature math applies to 99.9% of combat instead of 0.1%. Naming pass delegated to writing-category task.
3. **Re-tune the rarity multiplier ladder**: `5 / 10 / 20 / 40 / 100` → `5 / 8 / 13 / 22 / 35`. Drops the Common→Legendary effective power gap from 160× to ~56×. Makes Rare and Epic feel like real upgrades, not stepping stones.

Plus the **boss spawn retune**: `1/1000` → **`1/500`** (locked). Bosses become ~weekly events instead of ~monthly. Combined with the 3% boss-Legendary rate, surfaces a Legendary every ~30 days of casual play.

---

## 1 · Current state, with corrected math

### 1.1 Player formulas ([src/character.rs](file:///home/duys/.repos/shellquest/src/character.rs))

| Quantity | Formula | Code |
|---|---|---|
| Initial `max_hp` | `20 + STR × 2` | character.rs:251 |
| Per-level `max_hp` | `+5` (NOT recomputed from STR) | character.rs:343 |
| `max_hp` at level L | `(20 + initial_STR × 2) + (L − 1) × 5` | combined |
| `attack_power` | `STR + DEX/2 + weapon.power` | character.rs:282-286 |
| `defense` | `DEX/3 + armor.power + ring.power` | character.rs:288-293 |
| `crit_threshold` (boss combat) | `max(15, 20 − INT/4)` | boss.rs:92 |
| Crit multiplier | `×2` | boss.rs:120 |
| Per-level stat gain | `+1 STR, +1 DEX, +1 INT` | character.rs:340-342 |
| Post-prestige HP bonus | `+10 per prestige tier` | character.rs:374 |

**XP curve, computed**:

| Level | XP to next | Cumulative XP |
|---|---|---|
| 1 | 25 | 25 |
| 10 | 160 | ~825 |
| 30 | 780 | ~9,800 |
| 60 | 2,780 | ~64,400 |
| 100 | 8,200 | ~298,000 |
| 130 | 16,000 | ~734,000 |
| 150 | 26,300 | ~1,222,000 |

Step-jumps at L11/31/61/101/131 (per-level cost spikes 40-90% at boundaries). Per the years-long-passive design intent, this is acceptable — the steps act as informal "act breaks" in the progression narrative.

### 1.2 Max HP curve (corrected from v1)

| Build | L1 | L25 | L50 | L100 | L150 |
|---|---|---|---|---|---|
| Orc-Warrior | 60 | 180 | 305 | 555 | 805 |
| Human-Warrior | 54 | 174 | 299 | 549 | 799 |
| Wizard-Goblin | 30 | 150 | 275 | 525 | 775 |
| Necromancer-Goblin | 30 | 150 | 275 | 525 | 775 |

**Critical observation**: the L1 HP gap of 30 vs 60 is **2× spread (100% gap)**. By L100 the gap is **555 vs 525 = 5.7%**. The `+5/level` accumulator dwarfs the per-STR formula, so the squishy-class disadvantage **evaporates** by L30-40.

This was Oracle's catch on the v1 draft — my "STR compounding dominance" finding was wrong about endgame. The real finding (see §3) is the difficulty curve mismatch: L1-10 is brutal, L30+ is trivial, no middle window.

### 1.3 Classes ([src/character.rs:14-24](file:///home/duys/.repos/shellquest/src/character.rs#L14-L24))

| Class | STR | DEX | INT | Signature trigger | Effect |
|---|---|---|---|---|---|
| Wizard | 6 | 8 | 16 | Always on hit | `+INT/4` damage ("arcane burn") |
| Warrior | 16 | 8 | 6 | HP < 33% of max | `+STR/4` damage ("battle frenzy") |
| Rogue | 8 | 16 | 6 | Nat-1 still hits | (no bonus damage, "shadow strike") |
| Ranger | 10 | 14 | 6 | Boss at full HP (first strike) | `+INT/3` damage ("mark prey") |
| Necromancer | 6 | 6 | 18 | On kill | Heal `INT/3` HP ("soul drain") |

### 1.4 Races ([src/character.rs:48-57](file:///home/duys/.repos/shellquest/src/character.rs#L48-L57))

| Race | STR | DEX | INT |
|---|---|---|---|
| Human | +1 | +1 | +1 |
| Elf | 0 | +2 | +2 |
| Dwarf | +3 | 0 | +1 |
| Orc | +4 | +1 | −1 |
| Goblin | −1 | +3 | +1 |

### 1.5 Combat resolution

**Two distinct combat systems exist:**

**Regular monsters** ([events.rs:817-902](file:///home/duys/.repos/shellquest/src/events.rs#L817-L902)) — single-roll, no HP pool:

```
hit_roll  = d20
dodge_roll = d20
player_hits  = hit_roll + attack_power > 10
monster_hits = dodge_roll > 8 + defense/2

  player_hits ∧ ¬monster_hits  → WIN    (kill, +XP, no damage)
  player_hits ∧  monster_hits  → TOUGH  (kill, +XP, take max(1, monster_atk − defense/3))
 ¬player_hits ∧  monster_hits  → LOSE   (take max(1, monster_atk − defense/4))
 ¬player_hits ∧ ¬monster_hits  → DRAW   (nothing)
```

**Hit chance saturates at attack_power ≥ 10.** Every L10+ character is at ~100% hit-rate against every monster — combat tension comes entirely from the dodge roll. Note also the **asymmetric defense divisor** (3 on TOUGH, 4 on LOSE) — missing takes slightly more damage than connecting. Minor design quirk; flag for review only.

**Boss combat** ([boss.rs:91-126](file:///home/duys/.repos/shellquest/src/boss.rs#L91-L126)) — HP pool, crits, signatures, multi-turn:

```
hit_roll = d20
crit_threshold = max(15, 20 − INT/4)
player_hits =
  Rogue:  hit_roll + atk > 10  OR  hit_roll == 1   (Shadow Strike saves the fumble)
  other:  hit_roll + atk > 10
raw_dmg  = rng(atk/2..=atk)  + signature_bonus
applied  = raw_dmg × 2  if  hit_roll ≥ crit_threshold,  else  raw_dmg
```

### 1.6 Monsters ([events.rs:792-815](file:///home/duys/.repos/shellquest/src/events.rs#L792-L815))

10-monster pool, uniform random selection, **no HP pool, no level gating, no zone-appropriate filtering**:

| Name | base ATK | base XP |
|---|---|---|
| Syntax Error Snake | 4 | 6 |
| Race Condition Rat | 5 | 8 |
| Null Pointer Wraith | 6 | 10 |
| Infinite Loop Imp | 6 | 10 |
| Memory Leak Slime | 7 | 12 |
| Segfault Specter | 8 | 15 |
| Off-by-One Ogre | 10 | 20 |
| Dependency Hell Hound | 11 | 22 |
| Deadlock Demon | 12 | 25 |
| Buffer Overflow Beast | 14 | 30 |

**Zone scaling** ([events.rs:762-770](file:///home/duys/.repos/shellquest/src/events.rs#L762-L770)): `0.9× / 1.1× / 1.4× / 1.8× / 2.2×` ATK by danger level.

**Elite variants** ([events.rs:781-790](file:///home/duys/.repos/shellquest/src/events.rs#L781-L790)) — prefixed "Enraged X": `1.6 × (1 + (danger-1) × 0.15)` ATK multiplier, 2× XP. At danger 5 elite Buffer Overflow Beast: **35.8 ATK**.

**Critical issue**: any monster can spawn in any zone via uniform random; combined with zone-scale + elite multiplier, a fresh L1 character in `/tmp` can roll a 36-ATK monster on tick 2 (covered in §3 finding A1).

### 1.7 Bosses ([src/boss.rs:17-23](file:///home/duys/.repos/shellquest/src/boss.rs#L17-L23))

| Boss | HP | ATK | XP | Gold |
|---|---|---|---|---|
| Lord of /dev/null | 85 | 18 | 700 | 280 |
| SIGKILL Supreme | 90 | 25 | 800 | 320 |
| The Memory Corruption | 95 | 20 | 850 | 310 |
| The Kernel Panic | 100 | 22 | 900 | 350 |
| The Infinite Loop | 110 | 15 | 950 | 300 |

**Spawn rate**: `gen_ratio(1, 1000)` per tick = **0.1%**. Designer confirmed this was undiscovered tuning. At 100 ticks/day active use: median time to first boss ≈ 10 days. **Recommend retune to 1/300 or 1/500** to make bosses a monthly-ish event rather than seasonal.

**Boss loot table** ([loot.rs:248-263](file:///home/duys/.repos/shellquest/src/loot.rs#L248-L263)) — _this is the system v1 missed_:
- Legendary: **3%**
- Epic: ~10%
- Rare: ~47%
- Uncommon: ~40%
- Common: **0%** (bosses never drop Commons)

**Effective Legendary rate per active day** (at 100 ticks/day, current 1/1000 boss spawn): 1 boss / 10 days × 3% = ~1 Legendary per 333 days from bosses alone. Plus 1 per 100 days from regular tick drops = total **~1 Legendary per 77 days of casual play**. Retuning boss spawn to 1/300 drops this to ~1 per 30 days. Still rare, but tractable.

### 1.8 Loot system, corrected ([src/loot.rs:224-300](file:///home/duys/.repos/shellquest/src/loot.rs#L224-L300))

**FOUR roll functions exist**, not one:

| Function | Used by | Rarity distribution | Notes |
|---|---|---|---|
| `roll_loot(danger)` | Regular tick drops | 70/25/4/0.99/0.01 | **`danger_level` parameter is dead code** — prefixed `_` and unused. Flat distribution. |
| `roll_boss_loot()` | Boss defeat | 0/40/47/10/3 | No Commons. Path to Legendaries. |
| `roll_shop_loot()` | Shop daily refresh | 70/25/5/0/0 | Common+Uncommon+Rare only. No Epic/Legendary. |
| `roll_loot_scaled(danger)` | Arena chests | Danger-scaled | Actually uses danger argument. |

**Item power formula**: `power × mult + mult`, multipliers Common=5, Uncommon=10, Rare=20, Epic=40, Legendary=100.

**Effective power per rarity** (multiplier × midpoint of power range):

| Tier | Power range | Effective power | × Common median |
|---|---|---|---|
| Common | 1-4 | 12.5 | 1.0× |
| Uncommon | 3-6 | 45 | 3.6× |
| Rare | 5-10 | 150 | 12× |
| Epic | 8-15 | 460 | 36.8× |
| Legendary | 15-25 | 2000 | **160×** |

**Industry-typical Common→Legendary gap is 5-10×.** shellquest's 160× means a single Legendary obsoletes every other item for that slot permanently — there's no within-tier variance to make a great Epic compete with a mediocre Legendary.

### 1.9 Arena economy ([src/arena.rs](file:///home/duys/.repos/shellquest/src/arena.rs))

| Tier | Floor fee | Level scaling | Gold dampener | Max rounds | Unlock |
|---|---|---|---|---|---|
| The Pit | 40 | lvl × 12 | gold/10 | 5 | always |
| The Gauntlet | 100 | lvl × 18 + prestige × 50 | gold/8 | 10 | lvl ≥ 25 OR prestige ≥ 1 |
| The Colosseum | 300 | lvl × 28 + prestige × 150 | gold/6 | 15 | lvl ≥ 60 OR prestige ≥ 1 |
| The Abyssal Arena | 800 | lvl × 40 + prestige × 250 | gold/5 | 25 | lvl ≥ 100 OR prestige ≥ 2 |
| Godslayer's Court | 2500 | lvl × 60 + prestige × 400 | gold/4 | 50 | lvl 150 AND prestige ≥ 3 |

Fee = `max(floor, level_formula, gold_dampener)`. The `gold/N` divisor is the wealth-regulator — the more gold you have, the more it costs to enter, which keeps arena entry meaningful for rich players.

**Rewards** are % of fee, capped at `~60% XP` and `~110% gold` at full clear (Pit). KO penalty: HP set to 25% of max-at-entry, fee lost.

### 1.10 Trap ([events.rs:213-240](file:///home/duys/.repos/shellquest/src/events.rs#L213-L240))

25% chance on `exit_code != 0`, deals 1-3 fixed damage.

**Trap damage as % of max_hp**:
| Class+Race | L1 | L25 | L100 | L150 |
|---|---|---|---|---|
| Wizard-Goblin | 10.0% | 2.0% | 0.57% | 0.39% |
| Orc-Warrior | 5.0% | 1.7% | 0.54% | 0.37% |

Trap is genuinely threatening at L1, totally irrelevant at L30+. Either the design intent is "flavor only after early game" (then OK as-is) or it needs to scale.

### 1.11 Death penalties

| Mode | Effect |
|---|---|
| Normal | `deaths += 1`; **`xp = 0`** (current-level XP wiped, cumulative preserved); `gold *= 0.85`; `hp = max_hp / 2` |
| Arena KO | `hp = max_hp_at_entry / 4`; fee lost; no other state change |
| Permadeath (opt-in) | full save deletion, eulogy printed |

Designer confirmed permadeath is hardcore opt-in — normal mode is canonical. So the normal-mode death penalty is what matters: -15% gold and current-level XP wipe.

### 1.12 Per-command event probabilities

Selected gates ([events.rs:60-160](file:///home/duys/.repos/shellquest/src/events.rs#L60-L160)):

| Command | Event | Probability |
|---|---|---|
| `cd` | travel | 1/3 |
| `git commit` | craft (always) | 1/1 |
| `git push` | quest (always) | 1/1 |
| `cargo build` / `npm build` | forge (always) | 1/1 |
| `git` (other) | discovery | 1/5 |
| `rm` / `del` | angry_spirit | 1/3 |
| `ls` / `find` / `fd` | search_loot | 1/5 |
| `ssh` / `curl` / `wget` | portal | 1/4 |
| `sudo` | power_surge | 1/4 |
| `cat` / `less` | familiar | 1/10 |

Engagement-per-tick is **well-tuned for git+cargo workflows**, possibly thin for sysadmin workflows (heavy `cd`/`ls`/`ssh`). Per the years-long passive design, this is acceptable.

### 1.13 XP modifiers

- Zone danger XP multiplier: `1.0× / 1.25× / 1.5× / 1.75× / 2.0×` ([events.rs:11-14](file:///home/duys/.repos/shellquest/src/events.rs#L11-L14))
- Class affinity XP multiplier: `1.5×` for matching base command ([events.rs:17-31](file:///home/duys/.repos/shellquest/src/events.rs#L17-L31))
- Max stack: `2.0 × 1.5 = 3.0×` base XP

The class affinity command list is curated and thematic. Don't touch.

### 1.14 Healer

`1 HP / 30 seconds` while `cwd == $HOME`, capped at 30 minutes of accumulated regen. Excellent attrition recovery mechanic. Don't touch.

---

## 2 · Industry conventions (the bar)

### 2.1 Hit-rate baseline (D&D 5e, MUD research)
- ~65% hit rate on level-appropriate enemies feels good
- shellquest saturates at 100% hit-rate by L10. Tension lives entirely in the dodge roll.

### 2.2 Rarity ladder gaps (Diablo / PoE / Borderlands)
- Industry Common→Legendary effective power ratio: **5-10×**
- shellquest: **160×**
- Industry within-tier variance keeps tiers overlapping (a top-roll Epic can rival a low-roll Legendary)
- shellquest has tight power ranges per tier so the multiplier IS the difference — no overlap

### 2.3 Drop rate philosophy
- Diablo III Loot 2.0: Legendaries ~3% drop rate after the studio post-mortem on "feels good" tuning
- Genshin Impact: 0.6% 5-star rate **with soft pity at 75 pulls and hard pity at 90**
- shellquest regular tick: 0.01% Legendary, **no pity**, but boss path gives 3% (gated by 0.1% boss spawn rate)

### 2.4 Idle / passive RPG retention
- Prestige loops drive 15-25% D30 retention
- Engagement-per-tick target: 50-70% of ticks should produce *some* feedback
- shellquest has prestige (good); engagement depends on workflow

### 2.5 Roguelike permadeath
- Modern roguelikes (Hades, Slay the Spire, Dead Cells) preserve meta-progression across deaths
- DCSS prevents first-floor mortality via explicit OOD prevention
- shellquest permadeath preserves nothing — but designer confirmed this is hardcore opt-in, so the OOD-style hand-tuning is less critical (hardcore players signed up)

---

## 3 · Imbalance findings

Re-ranked from v1 with designer's 5 constraints applied. Permadeath-only concerns are demoted; canonical-experience concerns are promoted.

### 🔴 A2 (reframed) — **The difficulty bathtub: brutal early, trivial late, no middle**

**The new finding** (corrected math; supersedes v1's "STR compounding dominance"):

| Build | L1 HP | L30 HP | L100 HP |
|---|---|---|---|
| Wizard-Goblin | 30 | 175 | 525 |
| Orc-Warrior | 60 | 205 | 555 |

**Damage taken vs Buffer Overflow Beast (ATK 14) in danger-3 zone**:

| Level | Wizard-Goblin defense | TOUGH damage taken | Fights to die from full HP |
|---|---|---|---|
| L1 | 3 | 13 | ~2 |
| L30 | 18 | 8 | ~22 |
| L100 | 66 | 1 (floored) | ~525 |

**Effectively immortal by L100.** The `+5/level` max_hp accumulator + per-level `+1 DEX` (boosts defense) make incoming damage trend toward the floor of 1. Combined with monsters having no HP pool (1-hit kills), combat is no longer interactive past L30.

**Why this is a P0 problem** for a years-long passive game: the player is expected to spend 95%+ of their journey at L30+ (since L1-30 cumulative XP is ~10K and L30-150 is ~1.2M). If combat is trivial for 95% of the experience, the moment-to-moment gameplay is *flat*. The tension you have in the L1-10 window evaporates and never returns.

**Recommendation** (high-impact): tie this to **A5 + A12** — tiered monster redesign with HP pools and zone-appropriate spawn pools. See §6 for the proposal.

---

### 🔴 A3 — **Rarity multiplier ladder is 4× steeper than industry**

Unchanged from v1. Effective Common→Legendary power gap is **160×**. Industry: 5-10×. A single Legendary is permanently slot-defining.

**Recommendation**: `5 / 10 / 20 / 40 / 100` → `5 / 8 / 13 / 22 / 35`. Drops gap to ~56×. Makes Rares and Epics feel like meaningful upgrades, not stepping stones.

| Tier | Current mult | Proposed mult | Effective power (with current power ranges) |
|---|---|---|---|
| Common | 5 | 5 | 12.5 |
| Uncommon | 10 | 8 | 36 |
| Rare | 20 | 13 | 97.5 |
| Epic | 40 | 22 | 253 |
| Legendary | 100 | 35 | 700 |

| Ratio | Current | Proposed |
|---|---|---|
| Common → Uncommon | 3.6× | 2.9× |
| Uncommon → Rare | 3.3× | 2.7× |
| Rare → Epic | 3.1× | 2.6× |
| Epic → Legendary | 4.3× | 2.8× |
| Common → Legendary | 160× | 56× |

**Risk**: existing saves have items priced by the current multipliers. A `state::load()` shim should recompute or accept old values; sell-prices will silently rebalance. **Touchpoints**: `loot.rs:367-376` (`item_price`).

---

### 🔴 A11 (new) — **`sq sell junk` accelerated gold inflation, no proportional sink**

**The chain**: pre-v1.17.0, players sold inventory items one-at-a-time. Inventory cap of 20 created friction that rate-limited Common+Uncommon → gold conversion. The shipped `sq sell junk` removed that friction entirely. **We caused the gold abundance the designer is now reporting.**

**Where gold piles up**: L20-80 mid-game. At L20, arena Pit fee = `max(40, 240, gold/10)` — if a player has 5,000 gold sitting around, fee is 500 gold per attempt. With nothing else to spend on (shop only carries Common-Rare items), gold accumulates unbounded.

**Recommendation** (highest player-impact gold sink, low implementation risk):

`sq enchant <item-name>` adds +1 power to an equipped weapon/armor/ring, capped at +5 per item, costing **`item_price(item) × (current_enchant_level + 1)`** gold per +1. So enchanting a Common Iron Sword (item_price 15) from +0→+1 costs 15, +1→+2 costs 30, +2→+3 costs 45, … +4→+5 costs 75. Total to max-enchant a Common: 225 gold.

For a Rare (item_price ~150): +0→+5 costs 2,250 gold.
For an Epic (item_price ~460): +0→+5 costs 6,900 gold.
For a Legendary (item_price ~2000): +0→+5 costs 30,000 gold.

**This gives gold direct gameplay value at every wealth tier.** A Wizard with 5,000 gold and a decent Rare weapon can outfit themselves competitively with an Orc-Warrior who got lucky on drops. The cost scaling means enchanting a Legendary is an *aspiration* requiring sustained gold accumulation — exactly the kind of mid-game goal a years-long passive game wants.

**Access rule** (designer decision): Wizard class can `sq enchant` from anywhere; all other classes must be in `$HOME` (same gate as shop / sell). Flavor justification: arcane mastery — the Wizard carries the workbench mentally; everyone else needs a forge.

**Visual treatment** (designer decision): enchanted items display an `[Enchanted +N]` tag in escalating colors. The color level is a visible trophy of the player's investment.

| Enchant level | Tag style |
|---|---|
| +1 | `[Enchanted +1]` green bold |
| +2 | `[Enchanted +2]` cyan bold |
| +3 | `[Enchanted +3]` blue bold |
| +4 | `[Enchanted +4]` magenta bold |
| +5 | `[Enchanted +5]` rainbow per-character (using `colored` crate's per-char styling), the "endgame trophy" appearance |

**Risk**: schema change — `Item` needs an `enchant_level: u32` field. Backwards-compatible serde default. **Touchpoints**: `character.rs::Item`, `state.rs::save/load`, new `cmd_enchant` in `main.rs`, attack/defense calculations need to add `enchant_level` to the base item power, `display.rs::format_item_rarity` extended for the enchant tag.

---

### 🟡 A1 (demoted) — **First-tick deadliness for fresh squishy characters**

Demoted from P0 to P1 because designer confirmed permadeath is hardcore opt-in. In normal mode, a death wastes a session but doesn't end the character — annoying but not retention-killing.

Math remains: a fresh L1 Wizard-Goblin (30 HP) in `/tmp` (danger 3) rolling an Enraged Buffer Overflow Beast (35.8 ATK) → TOUGH outcome one-shots (~50% probability when both rolls hit).

**Recommendation**: gets solved for free by A12 (tiered monster redesign with zone gating). Vermin tier (HP 1-3, damage 2-4) only spawns in danger 1-2 zones. Player can't roll a Horror in `/etc`.

### 🟡 A5 + A12 (merged) — **Bestiary is shallow + AI-placeholder; combine into a tiered redesign**

A5 (monster combat shallow vs boss combat depth) and A12 (designer-confirmed AI-placeholder bestiary, open to overhaul) are the same problem viewed from two angles. **Solving the bestiary by adding HP pools + zone gating IS the difficulty-curve fix.**

Full proposal in §6.

### 🟡 A6 — **Trap damage doesn't scale**

Fixed 1-3 damage. 10% of max HP at L1 (genuine threat), 0.4% at L150 (literally a rounding error). Either:
- (a) Accept that traps are "flavor only after early game" — keep as-is, document the intent
- (b) Scale: `trap_dmg = max(1, max_hp × rng(0.03..0.06))` — always 3-6% of max HP, consistently flavorful

**Recommendation**: (b). Costs nothing to implement, makes the mechanic consistent across the journey.

### 🟡 A7 — **Class signatures asymmetric in trigger reliability**

| Class | Trigger | Practical fire rate |
|---|---|---|
| Wizard | Always on hit | ~95% of fights |
| Ranger | Boss at full HP | ~0.1% of fights (boss-only) |
| Necromancer | On kill | ~50-95% (depending on hit rate) |
| Warrior | HP < 33% | ~5-10% (rare to be that hurt) |
| Rogue | Nat-1 only | ~5% (insurance) |

Wizard's "arcane burn" is always-on and free; Warrior's "battle frenzy" requires being near-dead, which is suicide in a passive game without a heal mechanism on the spot. Ranger's "mark prey" only fires against bosses (which are 0.1% of encounters).

**Recommendation**:
- **Warrior**: trigger threshold HP<33% → HP<60%. "Battle frenzy" now fires when wounded, not when dying.
- **Ranger**: extend "first strike" to "first 3 strikes per encounter" — fires multiple times in a boss fight; for regular monsters, just fires every fight (since monsters die in 1 hit anyway, this becomes a passive +INT/3 damage like Wizard's but smaller).
- **Wizard**: drop `+INT/4` to `+INT/5` to compensate for being always-on.
- **Necromancer**: keep current; the on-kill timing matches Necromancer fantasy.
- **Rogue**: add a secondary trigger — `dodge_roll == 20` reduces incoming damage to 1 (the "mirror image" defensive identity).

### 🟢 A8 — **Race spread Orc +4 vs Goblin -1 is wide**

±3 max would be tighter. Specifically:
- **Orc**: +4/+1/-1 → **+3/+1/-1** (net -1 stat)
- **Goblin**: -1/+3/+1 → **0/+3/+1** (net +1 stat)

Both end up at +3 net stat points like the others. Removes Orc's runaway advantage and Goblin's "strictly worse than Elf for INT builds" trap.

### 🟢 A9 — **Death penalty scales badly across levels**

L1 normal death: lose 25 XP (current level), 1.5 gold avg. Trivial.
L100 normal death: lose 8,200 XP, ~500 gold. Painful.

**Recommendation**: `xp = 0` → `xp = xp / 2`. Smooths the penalty curve.

### 🟢 A10 — **No gold from regular monster kills** (intentional, leave alone)

Confirmed design intent based on data flow. Gold consolidation through bosses + arena + sell-loot keeps the "interesting events feel rewarding" pattern. Don't add gold to monster kills.

### Note on A13 — `roll_loot` dead `_danger_level` arg

Cosmetic cleanup. Either remove the parameter or make it actually scale. Recommend removal (or merge `roll_loot` and `roll_loot_scaled` into one function with consistent semantics). Not balance-affecting; pure code hygiene.

---

## 4 · Recommendations — prioritized

| Priority | Finding | Concrete change | Risk | Effort |
|---|---|---|---|---|
| 🔴 P0 | A11 | Add `sq enchant <item>` command: +1 power per enchant, max +5/item, cost = `item_price × (current+1)` | Medium (schema change) | 1-2 sessions |
| 🔴 P0 | A2+A5+A12 | Tiered monster redesign (see §6) | High (touches combat feel) | 3-5 sessions |
| 🔴 P0 | A3 | Rarity multipliers `5/10/20/40/100` → `5/8/13/22/35` | Low (single constants in loot.rs) | 1 session |
| 🟡 P1 | (designer-flagged) | Boss spawn `1/1000` → `1/300` or `1/500` | Low (single constant in boss.rs:58) | trivial |
| 🟡 P1 | A6 | Trap dmg `1-3` → `max(1, max_hp × rng(0.03..0.06))` | Low | trivial |
| 🟡 P1 | A7 | Class signature retuning per §3.A7 | Medium (touches class identity) | 1 session |
| 🟢 P2 | A8 | Race spread Orc -1, Goblin +1 STR | Medium (changes existing saves' baseline) | trivial |
| 🟢 P2 | A9 | Death XP penalty `xp=0` → `xp/=2` | Low | trivial |
| 🟢 P2 | A1 | Solved by A12; no separate action needed | — | — |
| 🟢 cleanup | A13 | Remove dead `_danger_level` arg or merge with `roll_loot_scaled` | Low | trivial |
| 🟢 not | A10 | Leave monster-kill gold at 0 (intentional) | — | — |

---

## 5 · Gold-sink analysis (designer's direct question)

**Diagnosis**: Yes, shellquest needs at least one gold sink. The post-v1.17.0 acceleration via `sq sell junk` consolidates Common+Uncommon items into gold ~10× faster than the previous one-at-a-time flow. Arena fees self-regulate wealth at the high end, but the L20-80 mid-game has nothing.

**Existing sinks (insufficient)**:
- Arena fees (good but only matters when you're playing arena)
- Shop daily refresh (capped at Common-Rare, low ceiling)
- Death penalty -15% (rare event, mostly trivial)

**Proposed sink: gear enchantment** ([§3.A11](#)). One-line summary:

> `sq enchant <item-name>` adds +1 power to the targeted equipped item, capped at +5 per item, cost = `item_price × (current_enchant + 1)`. So Common max-enchant total = 225 gold. Legendary max-enchant total = 30,000 gold. Wizard class can enchant from anywhere; all others must be in `$HOME`. Visual `[Enchanted +N]` tag escalates in color from green → cyan → blue → magenta → rainbow at max enchant.

**Why this and not alternatives**:

| Sink option | Why pick or skip |
|---|---|
| **Gear enchant (recommended)** | Direct gameplay value, scales naturally with player wealth and gear quality, gives the squishy classes a real catch-up mechanism |
| Subclass respec | Niche — only useful for players who chose wrong on prestige. Solves a problem most players don't have. |
| Cosmetic title purchase | Pure flex; useful but not gameplay-impactful |
| Inn / healing services | Healer already exists for free in $HOME; would dilute existing mechanic |
| Bank / storage tax | Adds friction without value |
| Auction / trading | Single-player game, no market exists |

Enchantment is the highest-impact single sink. Subclass respec and cosmetic titles can be added later if needed but are unlikely to move the needle as much.

**Risk: schema change.** `Item` needs an `enchant_level: u32` field. Use serde default to keep existing saves loadable:

```rust
pub struct Item {
    pub name: String,
    pub slot: ItemSlot,
    pub power: i32,
    pub rarity: Rarity,
    #[serde(default)]
    pub enchant_level: u32,
}
```

Attack/defense formulas add `enchant_level` to the power contribution from each item. Sell price = `item_price(base) + enchant_level × (5 × rarity_mult)` (the rule of thumb: sell gives back ~10% of enchant investment).

### Post-v1.18 observation (v1.20 surfaced via `sq identify`)

At max enchant (+5), the **sell-enchant-bonus on a Legendary exceeds the base sell value**. Concrete example from a Legendary with `item_price = 2,170`:

- Base sell: `2170 / 2 = 1,085`
- Per-level enchant bonus: `2170 / 5 = 434`
- +5 enchant bonus: `5 × 434 = 2,170`
- Total sell at +5: `1,085 + 2,170 = 3,255`
- Bonus / base ratio: **3.0×**

Enchant-then-sell remains a substantial net loss in absolute terms (recovery ~3,255 on ~32,550 invested ≈ 10% — exactly the design target). But the *proportion* of sell value that's enchant-derived climbs from 0% at +0 to 67% at +5. A v1.22+ rebalance could cap the per-level bonus (e.g. `min(item_price / 5, item_price / 10)` once enchant_level exceeds 3) to keep the breakdown closer to 50/50 — purely a feel-of-the-display concern, not an economy break.

---

## 6 · Bestiary + loot overhaul proposal

Designer confirmed AI-placeholder content + open to overhaul. **Tie the rename to a mechanical redesign** — solves A1 (first-tick deadliness), A2 (difficulty bathtub), A5 (combat shallowness) in one pass.

### 6.1 Monster tier structure

| Tier | Family theme | Spawn zones (danger) | HP pool | ATK range | XP | Player level | Notes |
|---|---|---|---|---|---|---|---|
| **Vermin** | Filesystem creatures | 1-2 | 1-3 (single-hit) | 2-4 | 4-8 | L1-10 intro | Always 1-hit kills |
| **Bruiser** | Process spirits | 2-3 | 8-15 (2-3 hits) | 5-8 | 12-20 | L10-30 | HP pool, multi-turn |
| **Hunter** | Memory dwellers | 3-4 | 25-40 (3-5 hits) | 8-14 | 22-35 | L30-60 | Tactical |
| **Horror** | Kernel terrors | 4-5 | 60-100 (5-8 hits) | 14-22 | 40-65 | L60-100 | Elite-feeling |
| **Boss-adjacent** | Architecture gods | 5+ | 150-220 (10+ hits) | 22-30 | 80-120 | L100+ | Near-boss intensity |

8 monsters per tier × 5 tiers = **40 unique monsters** (vs current 10). Adds depth without bloat.

**Spawn rule**: zone danger gates which tiers can spawn there.
- Danger 1 zones (`$HOME`): Vermin only
- Danger 2 zones (`/etc`, `src/`, `target/`, `test/`): Vermin + Bruiser
- Danger 3 zones (`/tmp`, `/var`, `.git`): Bruiser + Hunter
- Danger 4 zones (`/dev`): Hunter + Horror
- Danger 5 zones (`node_modules`): Horror + Boss-adjacent

Player at L1 in `$HOME` can ONLY encounter Vermin — solves first-tick deadliness without needing a separate grace period. Player at L100 in `node_modules` faces Boss-adjacent monsters that take 10+ hits to kill — solves the trivialization problem.

**Naming sketch (you'll have better instincts; these are vibes-only):**

| Tier | Vibe direction | Sample names |
|---|---|---|
| **Vermin** | Cute-but-annoying small-creature names | Permission Pup, Symlink Slug, Inode Imp, Tmpfile Toad, Pidfile Pixie, Bashrc Beetle |
| **Bruiser** | Process / system spirits | Zombie Daemon, Orphan Thread, Defunct Wraith, Forked Beast, Init Stalker, Cron Crawler |
| **Hunter** | Memory horrors with hunter aesthetic | Heap Hag, Stack Beast, Cache Crawler, Pagefault Phantom, Mmap Marauder, GC Reaper |
| **Horror** | Big serious threats, kernel-level | Page Tiger, Mutex Devourer, Syscall Specter, Schedlock Lich, Race-Cursed Behemoth |
| **Boss-adjacent** | Cosmic / architectural / "you should run" | The Allocator-of-Last-Resort, Init's Shadow, The Reaper of PIDs, Sigchld's Hunger |

### 6.2 Combat depth as a side effect

With HP pools introduced for Bruiser+ tiers, the boss-combat depth mechanics (crits, signatures, multi-turn drama) **automatically apply** to regular combat. Wizard's arcane burn matters. Ranger's "first strike" can be redefined to "first hit of an encounter" — fires every fight against Bruiser+ tiers. The existing crit math (`crit_threshold = max(15, 20 - INT/4)`) suddenly has hundreds of opportunities per session instead of just boss fights.

**This is the biggest leverage point in the whole rebalance.** Adding HP pools to monsters connects the existing combat math (crits, signatures) to 99.9% of gameplay instead of 0.1%.

### 6.3 Loot table overhaul (concurrent)

**Same structure suggestion**: per-slot naming families with intensity-scaled fantasy by rarity.

| Slot | Common | Uncommon | Rare | Epic | Legendary |
|---|---|---|---|---|---|
| Weapon | "Iron Sword", "Rusty Dagger" | "Dwarven Mace", "Silver Blade" | "Frostbite", "Wraithedge" | "Heap-Stalker", "The Forking Edge" | "The Last Allocator's Wrath" |
| Armor | "Padded Vest", "Tattered Robe" | "Chain Mail", "Studded Leather" | "Plate of the Guard" | "Bulwark of the Daemon" | "The Schedlock Shroud" |
| Ring | "Bronze Band", "Tin Loop" | "Silver Sigil" | "Ring of the Adept" | "Cursed Loop of Mutex" | "The Sigil-of-Origin" |
| Potion | "Stale Coffee", "Flat Cola" | "Energy Drink", "Cold Brew" | "Coder's Elixir" | "Vial of Soul-Code" | "Distilled Architect's Tear" |

**Item count target**: 6-8 items per (rarity × slot) cell = 6 × 4 × 5 = 120-160 items total. The current ~150 is roughly right; the redesign is about *thematic coherence*, not bulk increase.

### 6.4 Naming pass: delegated to writing-category task

Designer decision: the bestiary + loot naming work is **delegated to a writing-category agent task** with the §6.1 tier framing as input and the §6.3 per-slot conventions as constraints. The agent returns naming proposals organized by tier × slot; designer approves before any names ship into code.

This decouples the mechanical work (tier data shape, HP pools, zone gating, combat HP-loop) from the content work (the 40 monster names + 120-160 item names). Mechanical first, content second — names can be swapped in after the data shape is locked.

### 6.5 Combat tension model: constant across levels (designer decision)

The journey-not-destination, passive-years-long design intent means combat tension should feel **constant** across the L1 → L150 arc — not escalating. Concretely:

| Level | Build's HP | Typical encounter HP | Typical encounter turns | Typical % HP lost per fight |
|---|---|---|---|---|
| L1 | 30-60 | Vermin tier (1-3) | 1 turn | ~10-20% |
| L30 | 150-205 | Bruiser tier (8-15) | 2-3 turns | ~10-20% |
| L60 | 300-355 | Hunter tier (25-40) | 3-5 turns | ~10-20% |
| L100 | 525-555 | Horror tier (60-100) | 5-8 turns | ~10-20% |
| L150 | 775-805 | Boss-adjacent tier (150-220) | 10+ turns | ~10-20% |

Numbers grow on both sides of the equation. The *feel* — how often you win, how much HP you lose per fight, how engaged you have to be — stays roughly constant. Player power expression is in *what you possess* (gear, enchants, prestige tier, Legendary count) and *what zones you can survive in* (eventually `node_modules` becomes home turf), not in *how easy individual fights become*.

**Why this matters for tuning**: every monster tier should be tuned so that a level-appropriate player wins fights ~70-80% of the time, takes 10-20% HP damage per fight, and uses 3-8 turns on tier-appropriate enemies. The arithmetic in §6.1's tier table should be sanity-checked against this rule.

This rules OUT the alternative "late game is the real challenge" design — shellquest is explicitly NOT a late-game-difficulty-curve game.

### 6.6 Implementation order if you green-light this

1. **First** — change the `monsters` array in `events.rs:792` to 40 entries with tier metadata
2. **Then** — add HP pool to combat resolution (regular combat needs the boss-style HP-tracking loop)
3. **Then** — zone-gated spawn pool selection in `random_monster_for_zone`
4. **Then** — fire writing-category task for naming (per §6.4)
5. **Then** — loot table overhaul (last because it doesn't affect mechanics directly)

This is a multi-session effort. Steps 1-3 are the mechanical lift; steps 4-5 are content passes once the data shape is settled.

---

## 7 · What's already well-balanced (don't touch)

1. Zone-danger XP scaling (`1.0× / 1.25× / 1.5× / 1.75× / 2.0×`) — gentle, rewarding
2. Class affinity command list (Wizard with vim/python, Warrior with cargo/make, etc.) — curated and delightful
3. Arena 5-tier structure with cash-out preview (v1.16 feature) — industry-best UX
4. Healer (1 HP / 30s in $HOME, 30-min cap) — perfect attrition recovery
5. Per-command event probability gates (1/3, 1/5, 1/10) — well-tuned for dev workflows
6. Zone naming (`/tmp` = Wasteland, `node_modules` = Abyss, `.git` = Time Vaults) — iconic, keep
7. Boss design (5 named bosses, 24h stale timeout, single-active enforcement) — only the spawn rate needs review
8. Permadeath as opt-in save flag — correct architectural choice
9. The 6-tier XP curve structure — step-jumps act as informal "act breaks" in the journey
10. Conventional commits + release-notes pipeline (v1.17) — unrelated to balance but excellent craft
11. `sq sell junk` itself — it's the right QoL feature; it just exposed the gold-sink gap

---

## 8 · Answered design decisions

These were open questions in earlier drafts; designer has now locked them in. Captured here so future readers see the final calls, not the deliberation.

| # | Question | Final answer |
|---|---|---|
| 1 | `sq enchant` location restriction? | Wizard class can enchant from anywhere; all other classes must be in `$HOME` (flavored as arcane mastery — the Wizard carries the workbench mentally) |
| 2 | Visual treatment for enchanted items? | `[Enchanted +N]` tag with escalating color: green (+1) → cyan (+2) → blue (+3) → magenta (+4) → rainbow per-character (+5). Visible trophy of player investment. |
| 3 | Bestiary naming pass: in-house or delegated? | Delegated to a writing-category agent task (per §6.4) |
| 4 | Combat tension model: constant or escalating across levels? | **Constant** (per §6.5). Numbers grow on both sides; feel stays ~10-20% HP lost per fight across all levels. The journey is in *what you have*, not *how hard fights get*. |
| 5 | Boss spawn rate retune target? | `1/500` per tick (~1 boss per 5 days at 100 ticks/day) — bosses become weekly notable events, not seasonal. Combined with 3% boss-Legendary rate → ~1 Legendary per ~30 days casual play |

### Remaining open question

- **Class naming**: designer referred to the caster class as "Mage" while shellquest currently uses "Wizard". Flagged for clarification. If a class rename is on the table, that's a separate change with display string updates across messages.rs, help.rs, and existing-save migration considerations.

---

## Method note

- 10 parallel research agents (6 internal explore, 4 external librarian)
- 2 Oracle synthesis attempts; both truncated at the output channel level (~50KB cap). Salvaged Oracle's key catches from thinking traces:
  - Per-level `+5 max_hp` accumulator missed in v1 draft (corrected throughout this v2)
  - `roll_loot_scaled` / `roll_boss_loot` / `roll_shop_loot` family missed in v1 (added §1.8)
  - Boss-Legendary 3% drop rate as the intended path to Legendaries
- Direct source verification: `character.rs`, `events.rs`, `boss.rs`, `loot.rs`, `arena.rs`, `zones.rs`, `main.rs`
- All numerical claims cite code references (file:line) or industry sources
- This is the **diagnosis**; implementation is a separate engagement

---

## Revision history

- **v2.1** (2026-05-23, later): designer locked all 5 open questions. Folded answers into §3.A11 (Wizard-anywhere enchant access + visual color escalation), §5 (enchant spec details), §6 (added §6.4 delegated-naming note + §6.5 constant-tension model + reordered §6.6 implementation steps). Replaced §8 "Open questions" with "Answered design decisions" table. One open item remains: Wizard-vs-Mage class naming (flagged separately).
- **v2** (2026-05-23): incorporated Oracle's max_hp correction (per-level +5 accumulator); discovered four-function loot system (`roll_loot` / `roll_boss_loot` / `roll_shop_loot` / `roll_loot_scaled`); applied designer's 5 constraints (years-long passive, permadeath opt-in, gold abundance, boss spawn untuned, bestiary placeholder); added §5 gold-sink analysis with gear-enchant recommendation; added §6 bestiary + loot overhaul proposal with tiered monster structure. Reframed A2 from "STR compounding dominance" (wrong about endgame) to "difficulty bathtub" (the real finding). Demoted A1 since permadeath is hardcore opt-in.
- **v1** (2026-05-22): initial 10-agent analysis pass. Replaced entirely by v2; comparison available via git log.
