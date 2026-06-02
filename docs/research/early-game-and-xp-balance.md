# Early-game power and XP pacing investigation

Date: 2026-05-30  
Scope: investigation/proposal only; no master game-code edits.

## Inputs verified in code

- `src/character.rs`
  - New characters start at `xp_to_next: 25`.
  - After level-up, current early curve is `1..=10 => self.level * 15 + 10` where `self.level` is the new level. Effective early costs are therefore:
    - L1→L2: 25
    - L2→L3: 40
    - L3→L4: 55
    - L4→L5: 70
    - L5→L6: 85
    - cumulative to L6: 275
  - Attack: `strength + dexterity / 2 + weapon_power`.
  - Defense: `dexterity / 3 + armor + ring`.
  - Creation HP: `20 + strength * 2`; level-up adds `+1 STR/DEX/INT` and `+5 max_hp`.
- `src/events.rs`
  - `final_xp = scaled_xp(base, zone.danger_level) * affinity_multiplier(class, cmd)`.
  - Zone danger multiplier: danger 1 = 1.0×, then +0.25× per danger up to danger 5 = 2.0×.
  - Affinity multiplier: 1.5× for class-preferred commands.
  - Major event bases include craft/quest `15..=35`, discovery/forge `8..=20`, power surge `15..=30`, many other handlers in roughly `5..=30` bands.

## Simulator runs

The `just sim-custom` recipe snapshots every 50 ticks by default, too sparse to measure exact per-level intervals. I ran the requested matrix, then re-ran the same matrix directly through `runner.py` with `--snapshot-every 1`.

Baseline labels in `/Users/duan.uys/.repos/shellquest/dev-tools/balance-sim/runs.db`:

- `xp-pacing-baseline`: requested recipe, 90 runs, sparse snapshots.
- `xp-pacing-baseline-snap1`: same matrix, 90 runs, per-tick snapshots.
- `xp-pacing-baseline-snap1-plus`: same matrix, 300 runs, per-tick snapshots, used for the main baseline tables below.

A/B test ran in a throwaway copy at `/var/folders/mb/qc6_4__n2_qc7xflzz13rzy80000gn/T/opencode/shellquest-xp-ab`, not on master. Label:

- `xp-curve-30x-plus40-snap1-exact-seeded`: same 90-run matrix, per-tick snapshots, with proposed early curve and simulator seed adjusted to the same initial L1 cost.

Command used for the exact baseline pattern:

```bash
SQ_BIN_HOST="/Users/duan.uys/.repos/shellquest/dev-tools/balance-sim/.sq-linux/sq" \
SIM_IMAGE=shellquest-sim \
python3 "/Users/duan.uys/.repos/shellquest/dev-tools/balance-sim/runner.py" \
  --runs 6 \
  --classes Wizard Warrior Rogue Ranger Necromancer \
  --strategies greedy balanced conservative \
  --races Human \
  --target-level 8 \
  --parallel 4 \
  --tuning-label xp-pacing-baseline-snap1 \
  --min-arena-tier 1 \
  --snapshot-every 1
```

## A) XP pacing

### SQL: commands/ticks per early level

```sql
WITH reaches AS (
  SELECT
    r.id run_id,
    r.class,
    MIN(CASE WHEN ts.level >= 2 THEN ts.tick_no END) l2,
    MIN(CASE WHEN ts.level >= 3 THEN ts.tick_no END) l3,
    MIN(CASE WHEN ts.level >= 4 THEN ts.tick_no END) l4,
    MIN(CASE WHEN ts.level >= 5 THEN ts.tick_no END) l5,
    MIN(CASE WHEN ts.level >= 6 THEN ts.tick_no END) l6
  FROM run r
  JOIN tick_snapshot ts ON ts.run_id = r.id
  WHERE r.tuning_label = 'xp-pacing-baseline-snap1-plus'
  GROUP BY r.id, r.class
), intervals AS (
  SELECT
    class,
    l2 l1_l2,
    l3 - l2 l2_l3,
    l4 - l3 l3_l4,
    l5 - l4 l4_l5,
    l6 - l5 l5_l6,
    l6 l1_l6
  FROM reaches
)
SELECT
  'ALL' AS class,
  COUNT(*) runs,
  ROUND(AVG(l1_l2), 1) L1_to_L2,
  ROUND(AVG(l2_l3), 1) L2_to_L3,
  ROUND(AVG(l3_l4), 1) L3_to_L4,
  ROUND(AVG(l4_l5), 1) L4_to_L5,
  ROUND(AVG(l5_l6), 1) L5_to_L6,
  ROUND(AVG(l1_l6), 1) L1_to_L6,
  ROUND(275.0 / AVG(l1_l6), 1) implied_xp_per_command
FROM intervals
UNION ALL
SELECT
  class,
  COUNT(*) runs,
  ROUND(AVG(l1_l2), 1),
  ROUND(AVG(l2_l3), 1),
  ROUND(AVG(l3_l4), 1),
  ROUND(AVG(l4_l5), 1),
  ROUND(AVG(l5_l6), 1),
  ROUND(AVG(l1_l6), 1),
  ROUND(275.0 / AVG(l1_l6), 1)
FROM intervals
GROUP BY class
ORDER BY class;
```

### Baseline result: current curve

| class | runs | L1→L2 | L2→L3 | L3→L4 | L4→L5 | L5→L6 | L1→L6 | implied XP/command |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| ALL | 300 | 3.8 | 4.0 | 5.2 | 6.8 | 8.7 | 28.5 | 9.6 |
| Necromancer | 60 | 3.3 | 3.3 | 3.7 | 5.2 | 6.6 | 22.1 | 12.5 |
| Ranger | 60 | 3.7 | 4.2 | 5.7 | 6.7 | 9.9 | 30.1 | 9.1 |
| Rogue | 60 | 4.0 | 4.6 | 6.3 | 8.4 | 8.1 | 31.4 | 8.8 |
| Warrior | 60 | 4.2 | 3.4 | 5.2 | 6.9 | 9.0 | 28.7 | 9.6 |
| Wizard | 60 | 3.8 | 4.8 | 5.3 | 6.6 | 9.9 | 30.3 | 9.1 |

Verdict: the complaint is supported. The average character reaches L3 in ~7.8 commands and L6 in ~28.5 commands. Necromancer is the fastest bucket, reaching L6 in ~22 commands, consistent with frequent `git` affinity. This is too fast for a passive terminal RPG if the target is roughly:

- L1→L2: ~5–10 meaningful commands/events, not 1–4.
- L1→L6: ~60+ commands, not ~20–30.

The root cause is not only one handler. The whole early XP cost band is too low relative to normal event income. Even danger-1 affinity craft can roll roughly 23–53 XP after affinity, and the L1→L2 cost is only 25.

### Lever evaluation

#### Lever 1: steepen the early curve

Recommended concrete change:

```rust
// Initial L1 cost should match the same formula too:
xp_to_next: 70,

// In level_up_core, for levels 1..=10 after increment:
1..=10 => self.level * 30 + 40,
```

Effective proposed early costs:

| transition | current cost | proposed cost |
|---|---:|---:|
| L1→L2 | 25 | 70 |
| L2→L3 | 40 | 100 |
| L3→L4 | 55 | 130 |
| L4→L5 | 70 | 160 |
| L5→L6 | 85 | 190 |

New cumulative XP table:

| reach level | current cumulative XP | proposed cumulative XP |
|---|---:|---:|
| L2 | 25 | 70 |
| L3 | 65 | 170 |
| L4 | 120 | 300 |
| L5 | 190 | 460 |
| L6 | 275 | 650 |

#### Lever 2: cut income

Cutting craft/quest from `15..=35` to something like `6..=14` would fix the single most obvious one-shot level-up case, but it would not address the broader issue: many handlers still pay `8..=20`, `15..=30`, `15..=25`, combat XP, and zone/affinity scaling still amplify them. Lowering affinity globally would also punish the class identity system and still leave low non-affinity level costs.

Recommendation: first steepen early XP costs, not event income. Keep event messages/rewards feeling noticeable, but make level thresholds absorb them. Revisit income only if a later sweep still shows specific command families as outliers.

### A/B SQL

Same query as above, against the throwaway A/B DB and label `xp-curve-30x-plus40-snap1-exact-seeded`, with `650.0 / AVG(l1_l6)` for implied XP/command because proposed cumulative L1→L6 is 650.

### A/B result: proposed curve

| class | runs | L1→L2 | L2→L3 | L3→L4 | L4→L5 | L5→L6 | L1→L6 | implied XP/command |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| ALL | 90 | 8.1 | 8.0 | 12.5 | 16.1 | 18.5 | 63.1 | 10.3 |
| Necromancer | 18 | 5.3 | 5.7 | 10.7 | 15.7 | 15.8 | 53.2 | 12.2 |
| Ranger | 18 | 8.9 | 10.3 | 13.5 | 16.4 | 17.8 | 66.8 | 9.7 |
| Rogue | 18 | 9.1 | 8.6 | 13.5 | 15.2 | 17.7 | 64.1 | 10.1 |
| Warrior | 18 | 7.8 | 7.8 | 10.9 | 14.3 | 21.2 | 62.1 | 10.5 |
| Wizard | 18 | 9.3 | 7.4 | 13.7 | 18.8 | 20.0 | 69.3 | 9.4 |

This lands the target: L1→L2 becomes ~8 commands overall, and L1→L6 becomes ~63 commands overall. Necromancer remains the fastest due to affinity/strategy, but even it moves from ~22 commands to ~53 commands for L1→L6.

## B) Early-game power

### SQL: early overworld encounters

```sql
SELECT
  r.class,
  e.character_level AS level,
  COUNT(*) AS encounters,
  SUM(CASE WHEN e.outcome IN ('kill','win') THEN 1 ELSE 0 END) AS wins,
  ROUND(100.0 * SUM(CASE WHEN e.outcome IN ('kill','win') THEN 1 ELSE 0 END) / COUNT(*), 1) AS win_pct,
  SUM(CASE WHEN e.outcome IN ('death','loss') THEN 1 ELSE 0 END) AS deaths,
  ROUND(100.0 * SUM(CASE WHEN e.outcome IN ('death','loss') THEN 1 ELSE 0 END) / COUNT(*), 1) AS death_pct,
  SUM(CASE WHEN e.outcome = 'draw' THEN 1 ELSE 0 END) AS draws,
  ROUND(100.0 * SUM(CASE WHEN e.outcome = 'draw' THEN 1 ELSE 0 END) / COUNT(*), 1) AS draw_pct,
  ROUND(AVG(e.dmg_dealt), 1) AS avg_dmg_dealt,
  ROUND(AVG(e.dmg_taken), 1) AS avg_dmg_taken
FROM overworld_encounter e
JOIN run r ON r.id = e.run_id
WHERE r.tuning_label = 'xp-pacing-baseline-snap1-plus'
  AND e.character_level BETWEEN 1 AND 4
  AND e.kind = 'mob'
GROUP BY r.class, e.character_level
ORDER BY r.class, e.character_level;
```

### Early combat result, levels 1–4

| class | level | encounters | wins | win % | deaths | death % | draws | draw % | avg dmg dealt | avg dmg taken |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Necromancer | 1 | 1 | 1 | 100.0 | 0 | 0.0 | 0 | 0.0 | 20.0 | 0.0 |
| Necromancer | 3 | 3 | 3 | 100.0 | 0 | 0.0 | 0 | 0.0 | 21.3 | 7.3 |
| Necromancer | 4 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 20.0 | 7.0 |
| Ranger | 1 | 1 | 1 | 100.0 | 0 | 0.0 | 0 | 0.0 | 16.0 | 0.0 |
| Ranger | 2 | 3 | 3 | 100.0 | 0 | 0.0 | 0 | 0.0 | 22.3 | 3.3 |
| Ranger | 3 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 21.0 | 0.0 |
| Ranger | 4 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 28.0 | 0.0 |
| Rogue | 2 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 11.5 | 0.0 |
| Rogue | 3 | 3 | 3 | 100.0 | 0 | 0.0 | 0 | 0.0 | 16.0 | 0.0 |
| Warrior | 1 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 19.0 | 0.0 |
| Warrior | 3 | 5 | 5 | 100.0 | 0 | 0.0 | 0 | 0.0 | 20.0 | 1.0 |
| Warrior | 4 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 19.5 | 0.0 |
| Wizard | 2 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 11.0 | 0.0 |
| Wizard | 3 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 36.5 | 0.0 |
| Wizard | 4 | 2 | 2 | 100.0 | 0 | 0.0 | 0 | 0.0 | 24.0 | 0.0 |

Class aggregate, levels 1–4:

| class | encounters | win % | death % | draw % | avg dmg dealt | avg dmg taken |
|---|---:|---:|---:|---:|---:|---:|
| Necromancer | 6 | 100.0 | 0.0 | 0.0 | 20.7 | 6.0 |
| Ranger | 8 | 100.0 | 0.0 | 0.0 | 22.6 | 1.3 |
| Rogue | 5 | 100.0 | 0.0 | 0.0 | 14.2 | 0.0 |
| Warrior | 9 | 100.0 | 0.0 | 0.0 | 19.7 | 0.6 |
| Wizard | 6 | 100.0 | 0.0 | 0.0 | 23.8 | 0.0 |

Because early levels are so short, the sim still produces few level-1–4 overworld encounters. As a sanity extension, levels 1–6 aggregate to 62 encounters and still show 100% wins, 0% deaths, 0% draws:

| class | encounters | win % | death % | draw % | avg dmg dealt | avg dmg taken |
|---|---:|---:|---:|---:|---:|---:|
| Necromancer | 11 | 100.0 | 0.0 | 0.0 | 22.6 | 3.7 |
| Ranger | 17 | 100.0 | 0.0 | 0.0 | 22.7 | 0.6 |
| Rogue | 10 | 100.0 | 0.0 | 0.0 | 16.4 | 0.0 |
| Warrior | 13 | 100.0 | 0.0 | 0.0 | 20.5 | 0.4 |
| Wizard | 11 | 100.0 | 0.0 | 0.0 | 22.0 | 0.0 |

### Early-power verdict

The complaint is directionally supported, with the caveat that level 1–4 encounter count is sparse because XP pacing is too fast. Every observed early overworld fight is a win, with no deaths and no 30-turn draws. Damage taken is usually zero or near-zero.

For the specific Dwarf Ranger example:

- Ranger base: STR 10 / DEX 14 / INT 6.
- Dwarf bonus: +3 STR / +0 DEX / +1 INT.
- L1 no gear: STR 13 / DEX 14 / INT 7, HP `20 + 13*2 = 46`, ATK `13 + 14/2 = 20`.
- L2 no gear: STR 14 / DEX 15 / INT 8, HP `46 + 5 = 51`, ATK `14 + 15/2 = 21`.

DEX classes do benefit from DEX in two places: ATK via `dexterity / 2`, defense via `dexterity / 3`, and hit/dodge via `dex_mod()`. However, the observed 100% early win rate is not only a Ranger/Rogue problem; all classes show it.

Recommendation: do not change combat power in the same patch as XP pacing. Slowing XP will produce more level-1–4 encounter rows and keep characters at lower stats longer, so rerun the early-power query after the XP curve change before changing attack math. If that rerun still shows near-100% wins with negligible damage taken, the most surgical follow-up is:

```rust
// current
let base = self.strength + (self.dexterity / 2);

// candidate follow-up
let base = self.strength + (self.dexterity / 3);
```

Recomputed Dwarf Ranger no-gear ATK with `DEX / 3`:

| state | current ATK (`STR + DEX/2`) | candidate ATK (`STR + DEX/3`) |
|---|---:|---:|
| L1 Dwarf Ranger | 20 | 17 |
| L2 Dwarf Ranger | 21 | 19 |

That would reduce the reported L2 Dwarf Ranger from ATK 21 to ATK 19 without touching HP. But it should be validated in a separate combat-focused sim after the XP fix; otherwise we risk stacking two nerfs based on sparse early-combat rows.

## Final recommendation

1. Primary fix: steepen early XP curve.
   - Set initial/prestige L1 `xp_to_next` to 70.
   - Change early band to `1..=10 => self.level * 30 + 40`.
   - Sim proof: L1→L6 pacing improves from 28.5 commands to 63.1 commands overall; L1→L2 improves from 3.8 to 8.1 commands.
2. Do not cut event XP yet.
   - Event income feels meaningful and supports command flavor.
   - The problem is systemic low thresholds; reducing one or two event ranges would be incomplete.
3. Defer combat formula nerf until after the XP fix is simulated.
   - Current early data show 100% wins, 0 deaths, 0 draws, but encounter count at levels 1–4 is sparse due to the XP bug itself.
   - If still too easy after XP pacing is fixed, test `attack_power = strength + dexterity / 3 + gear` as a focused A/B.
