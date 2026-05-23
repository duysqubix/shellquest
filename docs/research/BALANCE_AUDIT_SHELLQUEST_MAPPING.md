# Shellquest Balance Audit: Mapping Industry Conventions to Current Design

**Date**: May 2026  
**Purpose**: Compare shellquest's current mechanics against idle/passive RPG industry conventions.

---

## Quick Assessment Framework

For each topic, ask:
1. **Does shellquest have this mechanic?** (Yes/No/Partial)
2. **How does it compare to industry conventions?** (Aligned/Divergent/Unique)
3. **What's the retention risk?** (Low/Medium/High)

---

## 1. PRESTIGE LOOPS

### Current Shellquest Design
- **Mechanic**: Permadeath mode (character dies → save file deleted)
- **Reset type**: Hard reset (no prestige currency, no multiplier carryover)
- **Preservation**: None (except achievements/journal entries)

### Industry Convention
- **Fractional exponent prestige** (sqrt formula) with permanent multiplier
- **Multi-layer prestige** (Ascensions, Infinities, Eternities)
- **Preservation**: Trophies, research, unlocks, artifacts

### Assessment
- **Status**: Divergent (permadeath ≠ prestige loop)
- **Retention risk**: HIGH
  - Permadeath creates finality, not replayability
  - No "ladder climbing" effect
  - Players who die once may not return
- **Recommendation**: Consider adding a prestige-like mechanic:
  - Option A: Soft reset with multiplier (e.g., "Legacy Points" that boost next character)
  - Option B: Keep permadeath but add "New Game+" mode with inherited bonuses
  - Option C: Hybrid: Permadeath for hardcore players, prestige loop for casual players

---

## 2. POWER-CURVE THEORY

### Current Shellquest Design
- **XP gain**: Linear (fixed XP per command, scaled by zone danger)
- **Stat growth**: Linear (fixed stat increases per level)
- **Equipment scaling**: Unknown (need to audit src/item.rs)

### Industry Convention
- **Hybrid approach**: Polynomial primary progression + Exponential costs + Logarithmic secondary
- **Multiplier range**: 1.07–1.15 per level (7–15% increase)
- **Cost formula**: `cost = base × 1.15^owned`

### Assessment
- **Status**: Partially aligned (linear XP is simple, but may feel flat)
- **Retention risk**: MEDIUM
  - Linear progression feels predictable (good for early game)
  - May hit a "dead zone" in mid-game where progression feels slow
  - No "power spike" moments to celebrate
- **Recommendation**:
  - Audit current XP curve (is it truly linear?)
  - Consider polynomial XP scaling for levels 10–50 (creates acceleration)
  - Keep early levels linear (feels rewarding)
  - Add equipment multipliers (exponential cost, polynomial power)

---

## 3. ENGAGEMENT-PER-TICK

### Current Shellquest Design
- **Feedback per tick**: Every `sq tick` produces:
  - Combat encounter (25% chance) → visual/text feedback
  - Loot drop → item added to inventory
  - XP gain → journal entry
  - Zone travel → status message
  - Silent tick (no event) → no feedback
- **Feedback cadence**: Estimated 30–50% of ticks produce visible feedback

### Industry Convention
- **Optimal cadence**: 50–70% of ticks produce visible feedback
- **Feedback hierarchy**: Small (60–80%), Medium (5–15%), Large (0.5–2%)
- **Micro-epic moments**: Every 10–12 minutes

### Assessment
- **Status**: Aligned (30–50% is within acceptable range for passive RPG)
- **Retention risk**: LOW
  - Silent ticks are acceptable if offline progress is summarized
  - Journal entries provide summary feedback
- **Recommendation**:
  - Verify that silent ticks don't exceed 50% (audit tick distribution)
  - Ensure journal entries are visible on `sq status` (summary feedback)
  - Consider adding "micro-epic" moments:
    - Boss defeat → special message + loot
    - Level-up → celebratory message
    - Zone transition → new music/flavor text

---

## 4. FIRST-HOUR CURVE

### Current Shellquest Design
- **Time to first command**: Immediate (user types any shell command)
- **Time to first combat**: Depends on zone danger (1–5 commands in safe zone)
- **Time to first level-up**: Unknown (need to audit XP requirements)
- **Time to first boss**: Unknown (need to audit boss spawn rates)

### Industry Convention
- **Time to Freedom (TTF)**: 1–5 minutes
- **First battle**: 5–10 minutes
- **First level-up**: 10–20 minutes
- **First boss**: 30–45 minutes
- **Time to Comfort (TTC)**: 60–120 minutes

### Assessment
- **Status**: Unknown (need to playtest first hour)
- **Retention risk**: MEDIUM
  - If first level-up takes >20 minutes, players may churn
  - If first boss takes <30 minutes, players may feel overwhelmed
- **Recommendation**:
  - Playtest first hour and measure:
    - Time to first combat
    - Time to first level-up
    - Time to first boss
    - Time to first major loot drop
  - Adjust XP requirements if needed
  - Ensure milestones are spaced 10–20 minutes apart

---

## 5. LATE-GAME WALL

### Current Shellquest Design
- **Max level**: Unknown (need to audit src/character.rs)
- **Endgame content**: Unknown (need to audit boss.rs, arena.rs)
- **Post-max-level progression**: Unknown

### Industry Convention
- **Problem**: 40–60% retention drop within weeks of max level
- **Solutions**:
  - Horizontal progression (cosmetics, collection)
  - Access rewards (unlock new zones, NPCs, mechanics)
  - Vertical progression with soft caps (rare item hunting)
  - Daily/seasonal content (quests, events)

### Assessment
- **Status**: Unknown (need to audit endgame design)
- **Retention risk**: HIGH (if endgame is not planned)
- **Recommendation**:
  - Define max level and what happens after
  - Plan endgame content NOW (don't treat it as an afterthought)
  - Consider adding:
    - Prestige loop (see #1)
    - Collection system (rare items, cosmetics)
    - Arena tiers (escalating difficulty)
    - Daily challenges (optional, not mandatory)

---

## 6. DROP-RATE PSYCHOLOGY

### Current Shellquest Design
- **Loot drops**: Combat encounters produce loot (rarity unknown)
- **Rarity tiers**: Unknown (need to audit src/loot.rs)
- **Drop rates**: Unknown (need to audit src/combat.rs)
- **Pity mechanics**: Unknown

### Industry Convention
- **Optimal rarity**: Common (70%), Uncommon (20%), Rare (8%), Legendary (2%)
- **Drop rate**: 2–4% legendary (Diablo 4) or 0.5–1% unique (Path of Exile)
- **Pity system**: Hard pity (guarantee after N attempts) or soft pity (gradual increase)
- **Dopamine**: Anticipation > receipt; extend reveal moment with animations

### Assessment
- **Status**: Unknown (need to audit loot system)
- **Retention risk**: MEDIUM
  - If loot feels too common, drops lose meaning
  - If loot feels too rare, players feel frustrated
- **Recommendation**:
  - Audit current drop rates and rarity distribution
  - Ensure legendary drops are rare enough to feel special (1–5%)
  - Consider adding soft pity (increase drop rate after N failed attempts)
  - Add visual/audio feedback for rare drops (color-coded text, sound effect)

---

## 7. ANTI-FRUSTRATION THRESHOLDS

### Current Shellquest Design
- **Trap encounters**: 25% chance per tick (can be frustrating if unlucky)
- **Boss encounters**: Spawn rate unknown
- **Pity mechanics**: None (as far as I can tell)
- **Quit thresholds**: Unknown

### Industry Convention
- **Hard pity**: Guarantee rare drop after N attempts
- **Soft pity**: Gradual increase in drop rate
- **Variance budgeting**: Plan for 2–3× expected attempts
- **Quit signals**: 3+ rage taps in 30s, 2+ errors in session, <30s first-session length

### Assessment
- **Status**: Divergent (no pity mechanics)
- **Retention risk**: MEDIUM
  - If players hit a "dry spell" (no loot, no level-ups), they may quit
  - Permadeath + no pity = high frustration
- **Recommendation**:
  - Add soft pity for rare drops (e.g., increase drop rate after 10 failed attempts)
  - Add guaranteed boss spawn after N ticks without boss (prevents dry spells)
  - Monitor player feedback for frustration signals
  - Consider adding "bad luck protection" (see Genshin Impact model)

---

## Summary: Shellquest vs. Industry Conventions

| Topic | Status | Risk | Priority |
|---|---|---|---|
| **Prestige Loops** | Divergent (permadeath) | HIGH | 1 (Critical) |
| **Power Curves** | Partially aligned | MEDIUM | 2 (Important) |
| **Engagement-per-Tick** | Aligned | LOW | 3 (Nice-to-have) |
| **First-Hour Curve** | Unknown | MEDIUM | 2 (Important) |
| **Late-Game Wall** | Unknown | HIGH | 1 (Critical) |
| **Drop-Rate Psychology** | Unknown | MEDIUM | 2 (Important) |
| **Anti-Frustration** | Divergent (no pity) | MEDIUM | 2 (Important) |

---

## Next Steps

1. **Audit current mechanics** (read src/ files)
2. **Playtest first hour** (measure time to milestones)
3. **Define endgame** (what happens at max level?)
4. **A/B test prestige** (soft reset vs. permadeath)
5. **Tune drop rates** (ensure rarity feels right)
6. **Add pity mechanics** (prevent frustration)

---

**Document**: `/home/duys/.repos/shellquest/docs/IDLE_RPG_DESIGN_AUDIT.md` (full reference)
