# Shellquest Balance Audit: Industry Conventions & Design Principles

**Date**: May 2026  
**Status**: Complete  
**Total Research**: 50+ sources, 7 topics, 841 lines of analysis

---

## 📋 Documents in This Audit

### 1. **IDLE_RPG_DESIGN_AUDIT.md** (603 lines, 33.7 KB)
   **The Reference Document**
   
   Comprehensive synthesis of idle/passive RPG design principles from:
   - Academic research (neuroscience, behavioral psychology)
   - GDC talks and game design articles
   - Game-specific wikis (Realm Grinder, Anti-Idle, Exponential Idle, Revolution Idle)
   - 2026 sources (current year, ensuring recency)
   
   **Seven Sections**:
   1. **Prestige Loops** — Reset mechanics, formulas, retention impact
   2. **Power-Curve Theory** — Linear, exponential, polynomial with trade-offs
   3. **Engagement-Per-Tick** — Feedback frequency, juiciness, micro-epic moments
   4. **First-Hour Curve** — XP pacing, level-up cadence, empirical JRPG data
   5. **Late-Game Wall** — Endgame design patterns, competence starvation
   6. **Drop-Rate Psychology** — Skinner, near-miss, Diablo vs. PoE, dopamine
   7. **Anti-Frustration Thresholds** — Pity systems, variance, quit signals
   
   **Key Features**:
   - 50+ citations with URLs and dates
   - Numerical conventions (e.g., multiplier 1.07–1.15 per level)
   - Conflicting guidance flagged with resolution notes
   - Confidence levels for each topic (High/Medium/Low)
   - Tables comparing design approaches

### 2. **BALANCE_AUDIT_SHELLQUEST_MAPPING.md** (238 lines, 8.7 KB)
   **The Shellquest-Specific Comparison**
   
   Point-by-point mapping of shellquest's current design against industry conventions.
   
   **For Each Topic**:
   - Current shellquest design (what we have)
   - Industry convention (what works elsewhere)
   - Assessment (Aligned/Divergent/Unknown)
   - Retention risk (Low/Medium/High)
   - Specific recommendations for shellquest
   
   **Priority Matrix**:
   - **Critical (HIGH RISK)**: Prestige loops, late-game wall
   - **Important (MEDIUM RISK)**: Power curves, first-hour pacing, loot psychology, anti-frustration
   - **Nice-to-have (LOW RISK)**: Engagement-per-tick feedback

---

## 🎯 Quick Start: Key Findings

### What Works for Idle/Passive RPGs

| Topic | Industry Consensus | Shellquest Status | Risk |
|---|---|---|---|
| **Prestige Loops** | Essential for retention (15–25% Day-30) | Permadeath (no prestige) | 🔴 HIGH |
| **Power Curves** | Hybrid (polynomial + exponential + logarithmic) | Linear XP | 🟡 MEDIUM |
| **Engagement-per-Tick** | 50–70% of ticks produce feedback | 30–50% (estimated) | 🟢 LOW |
| **First-Hour Pacing** | Level-ups every 10–20 min | Unknown (need to playtest) | 🟡 MEDIUM |
| **Late-Game Design** | Horizontal progression, access rewards | Unknown (need to plan) | 🔴 HIGH |
| **Loot Psychology** | Soft pity, rarity tiers, dopamine | Unknown (need to audit) | 🟡 MEDIUM |
| **Anti-Frustration** | Pity systems, variance budgeting | No pity mechanics | 🟡 MEDIUM |

---

## 📊 Numerical Conventions (Ready to Use)

### Prestige Cycles
- **Early game**: 5–15 minutes
- **Mid-game**: 30–60 minutes
- **Late game**: 2–8 hours
- **Retention impact**: +50% to +200% prestige currency per reset

### Power Curves
- **Cost multiplier**: 1.07–1.15 per level (7–15% increase)
- **Prestige formula**: `sqrt(max_currency)` or fractional exponent
- **Encounters per level-up**: 5–10 early, 15–30 mid, 50–200+ late
- **Critical threshold**: >100 encounters per level = "grindy" (15–25% churn)

### Feedback Cadence
- **Optimal**: 50–70% of ticks produce visible feedback
- **Feedback hierarchy**: Small (60–80%), Medium (5–15%), Large (0.5–2%)
- **Micro-epic moments**: Every 10–12 minutes

### First-Hour Milestones
- **Time to Freedom**: 1–5 minutes
- **First battle**: 5–10 minutes
- **First level-up**: 10–20 minutes
- **First boss**: 30–45 minutes
- **Time to Comfort**: 60–120 minutes

### Loot Rarity
- **Common**: 70%
- **Uncommon**: 20%
- **Rare**: 8%
- **Legendary**: 2%

### Pity Systems
- **Hard pity**: Guarantee after N attempts (e.g., 90 pulls in Genshin Impact)
- **Soft pity**: Gradual rate increase (e.g., 0.6% → 2.0% by pull 80)
- **Variance budgeting**: Plan for 2–3× expected attempts

---

## 🚀 Next Steps for Shellquest

### Immediate (This Week)
1. Read both audit documents
2. Audit current mechanics (src/ files)
3. Playtest first hour (measure time to milestones)

### Short-term (Next 1–2 Weeks)
1. Define max level and endgame content
2. Audit loot system (drop rates, rarity distribution)
3. Decide on prestige mechanic (soft reset vs. permadeath)

### Medium-term (Next 3–4 Weeks)
1. A/B test prestige frequency
2. Tune power curves (polynomial vs. linear)
3. Add pity mechanics (prevent frustration)
4. Plan endgame content (don't treat as afterthought)

---

## 📚 Sources at a Glance

### Academic & Research
- Schultz, W. (1998). Dopamine neuroscience
- Clark, L., et al. (2009). Gambling near-misses and brain reward
- Aalto University (2024). Game feel and emotional impact
- Springer Nature (2019). Near-miss effect meta-analysis

### Game Design (GDC, Game Developer)
- "The Math of Idle Games" (3-part series, 2016–2017)
- "Quantitative Design - XP Thresholds" (2018)
- "The JRPG Startup Cost" (2019)
- "Idle Chatter" (GDC 2016, Anthony Pecorella)
- "Designing Path of Exile to Be Played Forever" (GDC 2019, Chris Wilson)

### Game-Specific Wikis
- Realm Grinder (Reincarnation, Ascension)
- Anti-Idle (Level, Ascension, Reforged)
- Revolution Idle (Infinity, Prestige)
- Exponential Idle (Theories 1–8, Custom Theories)

### 2026 Sources (Current Year)
- Anastasia (2026). Idle game development and retention
- LevelCap News (2026). Endgame cliff problem
- Massively Overpowered (2026). Late-game paradox in MMORPGs
- Gamers Den (2026). Loot reward psychology
- DualMedia (2026). Diablo 4 loot system psychology
- onlinegaming.biz (2026). ARPG session design and retention

---

## ⚠️ Conflicting Guidance (Flagged)

| Topic | Consensus | Conflicting View | Resolution |
|---|---|---|---|
| **Near-miss effect** | Weak in experiments | Strong in player perception | Design for perception, not statistics |
| **Daily quests** | Effective for retention | Manipulative/fatiguing | Use sparingly; prefer optional content |
| **Prestige frequency** | 5–15 min early, 30–60 min mid | Varies wildly by game | Tune via A/B testing |
| **Endgame design** | Horizontal progression works | Vertical progression works | Both work; depends on player motivation |
| **Loot rarity** | High volume (Diablo) works | Low volume (PoE) works | Choose based on design philosophy |

---

## 🎓 Confidence Levels

- **High**: Prestige loops, power curves, loot psychology
- **Medium**: Endgame design (highly game-dependent), first-hour pacing
- **Low**: Exact engagement-per-tick thresholds (varies by genre)

---

## 📖 How to Use This Audit

1. **Start with this README** (you are here)
2. **Read BALANCE_AUDIT_SHELLQUEST_MAPPING.md** (understand shellquest's position)
3. **Dive into IDLE_RPG_DESIGN_AUDIT.md** (deep reference for each topic)
4. **Use numerical conventions** (apply to shellquest's design)
5. **Flag divergences** (where shellquest differs from industry norms)
6. **Plan experiments** (A/B test, playtest, audit)

---

**Research Completed**: May 22, 2026  
**Methodology**: Web search (7 parallel queries), academic papers, GDC talks, game postmortems, 2026 sources  
**Total Sources**: 50+ citations with URLs and dates  
**Scope**: Prestige loops, power curves, engagement mechanics, progression pacing, endgame design, loot psychology, anti-frustration thresholds

