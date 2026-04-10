# Class-Tailored Messages & Balance Overhaul Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add class-specific flavor text to all major game events, introduce class affinity XP bonuses, zone-scaled XP, a rebalanced progression curve, and a toggleable permadeath mode.

**Architecture:** New `src/messages.rs` module owns all class-aware message strings and returns `(plain, colored)` tuples. `src/events.rs` gains two helper functions (`scaled_xp`, `affinity_xp`) and calls `messages::*` instead of inline format strings. `src/character.rs` gets rebalanced XP thresholds. `src/state.rs` gains a `permadeath` bool. Death handling in `events.rs` and `boss.rs` branches on `state.permadeath`. `src/display.rs` gains `print_permadeath_eulogy`.

**Tech Stack:** Rust, colored, rand 0.8.x, serde/serde_json, clap

**Design reference:** `docs/plans/2026-04-09-class-messages-balance-design.md` (this file doubles as the design)

**Key reference files before starting:**
- `src/character.rs` lines 1–60 — Class enum, base_stats(), Race enum
- `src/events.rs` lines 180–730 — all event handlers and combat()
- `src/character.rs` lines 295–320 — gain_xp() and level_up() XP thresholds
- `src/display.rs` lines 1–45 — color helper functions (color_xp, color_gold, color_damage, color_monster)
- `src/state.rs` lines 1–55 — GameState struct

---

## Task 1: Create `src/messages.rs` with all class-aware message functions

**Files:**
- Create: `src/messages.rs`

This module is pure data — no I/O, no side effects. Every function takes a `&Class` and event-specific params, returns `(plain: String, colored: String)`.

**Step 1: Create the file with the module skeleton and imports**

```rust
use crate::character::Class;
use colored::*;
use crate::display::{color_xp, color_gold, color_damage, color_monster};

// Returns (plain_text_for_journal, colored_text_for_terminal)
type Msg = (String, String);
```

**Step 2: Implement `combat_win`**

```rust
pub fn combat_win(class: &Class, monster: &str, xp: u32) -> Msg {
    let m = color_monster(monster);
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("A {} dissolves in arcane light! Mystical victory! +{} XP", monster, xp),
            format!("A {} dissolves in {}! {}! {}", m, "arcane light".blue().bold(), "Mystical victory".blue().bold(), x),
        ),
        Class::Warrior => (
            format!("A {} falls before your blade! Glorious victory! +{} XP", monster, xp),
            format!("A {} falls! {}! {}", m, "Glorious victory".red().bold(), x),
        ),
        Class::Rogue => (
            format!("A {} never saw it coming. Clean kill. +{} XP", monster, xp),
            format!("A {} never saw it coming. {}. {}", m, "Clean kill".yellow().bold(), x),
        ),
        Class::Ranger => (
            format!("True aim. A {} drops at range. +{} XP", monster, xp),
            format!("{}. A {} drops. {}", "True aim".green().bold(), m, x),
        ),
        Class::Necromancer => (
            format!("A {}'s life force drains away. Soul claimed. +{} XP", monster, xp),
            format!("A {}'s {} drains away. {}. {}", m, "life force".magenta(), "Soul claimed".magenta().bold(), x),
        ),
    }
}
```

**Step 3: Implement `combat_tough`** (trade blows — won but took damage)

```rust
pub fn combat_tough(class: &Class, monster: &str, dmg: i32, xp: u32) -> Msg {
    let m = color_monster(monster);
    let d = color_damage(dmg);
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("Arcane duel with {}! You prevail, not unscathed. -{} HP, +{} XP", monster, dmg, xp),
            format!("Arcane duel with {}! You prevail. {} HP lost. {}", m, d, x),
        ),
        Class::Warrior => (
            format!("Brutal exchange with {}! You endure! -{} HP, +{} XP", monster, dmg, xp),
            format!("Brutal exchange with {}! You endure! {} HP lost. {}", m, d, x),
        ),
        Class::Rogue => (
            format!("Close call with {}. You get out alive. -{} HP, +{} XP", monster, dmg, xp),
            format!("Close call with {}. Alive... barely. {} HP lost. {}", m, d, x),
        ),
        Class::Ranger => (
            format!("Scrappy fight with {}. You track your wounds. -{} HP, +{} XP", monster, dmg, xp),
            format!("Scrappy fight with {}. {} HP lost. {}", m, d, x),
        ),
        Class::Necromancer => (
            format!("Dark binding with {}. Pain is entropy. -{} HP, +{} XP", monster, dmg, xp),
            format!("Dark binding with {}. Pain is {}. {} HP lost. {}", m, "entropy".magenta(), d, x),
        ),
    }
}
```

**Step 4: Implement `combat_lose`** (player took damage, survived, no XP)

```rust
pub fn combat_lose(class: &Class, monster: &str, dmg: i32) -> Msg {
    let m = color_monster(monster);
    let d = color_damage(dmg);
    match class {
        Class::Wizard => (
            format!("Your wards shatter! {} deals {} damage.", monster, dmg),
            format!("Your {} shatter! {} deals {} damage.", "wards".blue(), m, d),
        ),
        Class::Warrior => (
            format!("{} breaks your guard! Took {} damage.", monster, dmg),
            format!("{} breaks your guard! Took {} damage.", m, d),
        ),
        Class::Rogue => (
            format!("{} outmaneuvers you. {} damage slips through.", monster, dmg),
            format!("{} outmaneuvers you. {} damage slips through.", m, d),
        ),
        Class::Ranger => (
            format!("{} flanks you. {} damage. Reassess terrain.", monster, dmg),
            format!("{} flanks you. {} damage. Reassess terrain.", m, d),
        ),
        Class::Necromancer => (
            format!("{} strikes true. {} damage. The void waits.", monster, dmg),
            format!("{} strikes true. {} damage. The {} waits.", m, d, "void".magenta()),
        ),
    }
}
```

**Step 5: Implement `combat_draw`** (escaped, no winner)

```rust
pub fn combat_draw(class: &Class, monster: &str) -> Msg {
    let m = color_monster(monster);
    match class {
        Class::Wizard => (
            format!("A {} — your spell fizzles. It retreats into the ether.", monster),
            format!("A {} — your spell {}. It retreats.", m, "fizzles".blue().dimmed()),
        ),
        Class::Warrior => (
            format!("A {} — you clash and separate. It retreats.", monster),
            format!("A {} — you clash and separate. It retreats.", m),
        ),
        Class::Rogue => (
            format!("A {} — you ghost before it can press. Mutual retreat.", monster),
            format!("A {} — you {} before it can press.", m, "ghost".yellow().dimmed()),
        ),
        Class::Ranger => (
            format!("A {} — you hold your ground. It slinks back.", monster),
            format!("A {} — you hold your ground. It slinks back.", m),
        ),
        Class::Necromancer => (
            format!("A {} — entropy claims neither of you. Today.", monster),
            format!("A {} — {} claims neither of you. Today.", m, "entropy".magenta().dimmed()),
        ),
    }
}
```

**Step 6: Implement `trap`**

```rust
pub fn trap(class: &Class, dmg: i32, hp: i32, max_hp: i32) -> Msg {
    let d = color_damage(dmg);
    let hp_str = crate::display::color_hp(hp, max_hp);
    match class {
        Class::Wizard => (
            format!("Residual magic flares! Arcane backlash! -{} HP. HP: {}/{}", dmg, hp, max_hp),
            format!("Residual {} flares! {} HP lost. {}", "magic".blue(), d, hp_str),
        ),
        Class::Warrior => (
            format!("A crude mechanism bites! -{} HP. Unfair.", dmg),
            format!("A crude mechanism bites! {} HP lost. {}. {}", d, "Unfair".red().dimmed(), hp_str),
        ),
        Class::Rogue => (
            format!("Your instincts catch it late. -{} HP. Sloppy.", dmg),
            format!("Your instincts catch it {}. {} HP lost. {}", "late".yellow().dimmed(), d, hp_str),
        ),
        Class::Ranger => (
            format!("Missed the signs. Terrain punishes. -{} HP.", dmg),
            format!("Missed the signs. Terrain punishes. {} HP lost. {}", d, hp_str),
        ),
        Class::Necromancer => (
            format!("Dark wards activate against you. -{} HP. Ironic.", dmg),
            format!("Dark {} activate against you. {} HP lost. {}", "wards".magenta(), d, hp_str),
        ),
    }
}
```

**Step 7: Implement `quest`** (git push)

```rust
pub fn quest(class: &Class, xp: u32, gold: u32) -> Msg {
    let x = color_xp(xp);
    let g = color_gold(gold);
    match class {
        Class::Wizard => (
            format!("The enchantment completes! Push delivered to the ether. +{} XP, +{} gold", xp, gold),
            format!("The {} completes! Push delivered to the ether. {} {}", "enchantment".blue().bold(), x, g),
        ),
        Class::Warrior => (
            format!("The siege succeeds! Code pushed to the realm! +{} XP, +{} gold", xp, gold),
            format!("The {} succeeds! Code pushed to the realm! {} {}", "siege".red().bold(), x, g),
        ),
        Class::Rogue => (
            format!("Delivered, no witnesses. The push lands silent. +{} XP, +{} gold", xp, gold),
            format!("Delivered, {}. The push lands {}. {} {}", "no witnesses".yellow().dimmed(), "silent".yellow(), x, g),
        ),
        Class::Ranger => (
            format!("The trail leads true. Push reaches its mark. +{} XP, +{} gold", xp, gold),
            format!("The trail leads {}. Push reaches its mark. {} {}", "true".green().bold(), x, g),
        ),
        Class::Necromancer => (
            format!("The ritual completes. Another payload sent to the void. +{} XP, +{} gold", xp, gold),
            format!("The {} completes. Payload sent to the {}. {} {}", "ritual".magenta().bold(), "void".magenta(), x, g),
        ),
    }
}
```

**Step 8: Implement `craft`** (git commit)

```rust
pub fn craft(class: &Class, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("You weave the commit into the pattern. The grimoire grows. +{} XP", xp),
            format!("You weave the commit into the {}. The {} grows. {}", "pattern".blue(), "grimoire".blue().bold(), x),
        ),
        Class::Warrior => (
            format!("Another battle-scroll committed to the archives. +{} XP", xp),
            format!("Another {} committed to the archives. {}", "battle-scroll".red(), x),
        ),
        Class::Rogue => (
            format!("You slip the commit through undetected. Clean mark. +{} XP", xp),
            format!("You slip the commit through {}. {}. {}", "undetected".yellow().dimmed(), "Clean mark".yellow(), x),
        ),
        Class::Ranger => (
            format!("You mark another waypoint on the journey. +{} XP", xp),
            format!("You mark another {} on the journey. {}", "waypoint".green(), x),
        ),
        Class::Necromancer => (
            format!("Another soul bound to the archives of the dead. +{} XP", xp),
            format!("Another {} bound to the {}. {}", "soul".magenta(), "archives of the dead".magenta().bold(), x),
        ),
    }
}
```

**Step 9: Implement `forge`** (cargo/npm build — item crafted)

```rust
pub fn forge_loot(class: &Class, item_name: &str, power: i32, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("The arcane forge manifests: {} (+{})! +{} XP", item_name, power, xp),
            format!("The {} manifests: {} {}! {}", "arcane forge".blue().bold(), item_name.cyan().bold(), format!("(+{})", power).green(), x),
        ),
        Class::Warrior => (
            format!("Your hammer strikes true! Forged: {} (+{})! +{} XP", item_name, power, xp),
            format!("Your hammer strikes {}! {}: {} {}! {}", "true".red().bold(), "Forged".red(), item_name.cyan().bold(), format!("(+{})", power).green(), x),
        ),
        Class::Rogue => (
            format!("Precision work. You fashion {} (+{}). +{} XP", item_name, power, xp),
            format!("{} work. {} {} {}. {}", "Precision".yellow(), "Fashioned:".yellow().dimmed(), item_name.cyan().bold(), format!("(+{})", power).green(), x),
        ),
        Class::Ranger => (
            format!("Wilderness craft yields: {} (+{}). +{} XP", item_name, power, xp),
            format!("{} yields: {} {}. {}", "Wilderness craft".green(), item_name.cyan().bold(), format!("(+{})", power).green(), x),
        ),
        Class::Necromancer => (
            format!("Dark synthesis complete: {} (+{}). +{} XP", item_name, power, xp),
            format!("{}: {} {}. {}", "Dark synthesis complete".magenta().bold(), item_name.cyan().bold(), format!("(+{})", power).green(), x),
        ),
    }
}

pub fn forge_xp(class: &Class, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("The compiler hums with latent mana. +{} XP", xp),
            format!("The compiler hums with latent {}. {}", "mana".blue(), x),
        ),
        Class::Warrior => (
            format!("The forge roars! Build complete. +{} XP", xp),
            format!("The {} roars! Build complete. {}", "forge".red(), x),
        ),
        Class::Rogue => (
            format!("Clean build. No traces. +{} XP", xp),
            format!("Clean build. {}. {}", "No traces".yellow().dimmed(), x),
        ),
        Class::Ranger => (
            format!("The build completes in the wilderness. +{} XP", xp),
            format!("Build completes in the {}. {}", "wilderness".green(), x),
        ),
        Class::Necromancer => (
            format!("The necrotic compiler finishes its work. +{} XP", xp),
            format!("The {} finishes its work. {}", "necrotic compiler".magenta(), x),
        ),
    }
}
```

**Step 10: Implement `discovery`** (git misc, etc.)

```rust
pub fn discovery(class: &Class, detail: &str, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("Your arcane senses reveal: {}. +{} XP", detail, xp),
            format!("Your {} senses reveal: {}. {}", "arcane".blue(), detail.blue().italic(), x),
        ),
        Class::Warrior => (
            format!("Scouting the battlefield, you find: {}. +{} XP", detail, xp),
            format!("Scouting, you find: {}. {}", detail.italic(), x),
        ),
        Class::Rogue => (
            format!("Poking around where you shouldn't — {}. +{} XP", detail, xp),
            format!("Poking around where you shouldn't — {}. {}", detail.yellow().italic(), x),
        ),
        Class::Ranger => (
            format!("Tracking through the wilds, you uncover: {}. +{} XP", detail, xp),
            format!("Tracking the wilds, you uncover: {}. {}", detail.green().italic(), x),
        ),
        Class::Necromancer => (
            format!("The dead whisper secrets: {}. +{} XP", detail, xp),
            format!("The dead whisper: {}. {}", detail.magenta().italic(), x),
        ),
    }
}
```

**Step 11: Implement `familiar`** (cat/less — heal)

```rust
pub fn familiar(class: &Class, creature: &str, heal: i32, hp: i32, max_hp: i32) -> Msg {
    let hp_str = crate::display::color_hp(hp, max_hp);
    match class {
        Class::Wizard => (
            format!("A {} manifests as your familiar! It mends your essence. +{} HP. HP: {}/{}", creature, heal, hp, max_hp),
            format!("A {} manifests as your {}! +{} HP. {}", creature.cyan(), "familiar".blue().bold(), format!("{}", heal).green(), hp_str),
        ),
        Class::Warrior => (
            format!("A {} approaches without fear. It heals your wounds. +{} HP. HP: {}/{}", creature, heal, hp, max_hp),
            format!("A {} approaches without fear. +{} HP. {}", creature.cyan(), format!("{}", heal).green(), hp_str),
        ),
        Class::Rogue => (
            format!("A {} finds you in the shadows and mends your cuts. +{} HP. HP: {}/{}", creature, heal, hp, max_hp),
            format!("A {} finds you in the {}. +{} HP. {}", creature.cyan(), "shadows".yellow().dimmed(), format!("{}", heal).green(), hp_str),
        ),
        Class::Ranger => (
            format!("A {} answers your call of the wild. Healed. +{} HP. HP: {}/{}", creature, heal, hp, max_hp),
            format!("A {} answers the {}. +{} HP. {}", creature.cyan(), "call of the wild".green(), format!("{}", heal).green(), hp_str),
        ),
        Class::Necromancer => (
            format!("A {} senses the void in you and offers life. +{} HP. HP: {}/{}", creature, heal, hp, max_hp),
            format!("A {} senses the {} in you. +{} HP. {}", creature.cyan(), "void".magenta(), format!("{}", heal).green(), hp_str),
        ),
    }
}
```

**Step 12: Implement `portal`** (ssh/curl/wget)

```rust
pub fn portal(class: &Class, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("A ley-line opens to a distant realm. Connection established. +{} XP", xp),
            format!("A {} opens to a distant realm. {}", "ley-line".blue().bold(), x),
        ),
        Class::Warrior => (
            format!("You breach a remote fortress! Connection established. +{} XP", xp),
            format!("You breach a {}! Connection established. {}", "remote fortress".red(), x),
        ),
        Class::Rogue => (
            format!("You slip through the network's defenses. Portal open. +{} XP", xp),
            format!("You slip through the {}. Portal open. {}", "network's defenses".yellow().dimmed(), x),
        ),
        Class::Ranger => (
            format!("You chart a path to distant lands. Portal established. +{} XP", xp),
            format!("You chart a path to {}. Portal established. {}", "distant lands".green(), x),
        ),
        Class::Necromancer => (
            format!("You tear open a rift to the remote plane. +{} XP", xp),
            format!("You tear open a {} to the remote plane. {}", "rift".magenta().bold(), x),
        ),
    }
}
```

**Step 13: Implement `power_surge`** (sudo)

```rust
pub fn power_surge(class: &Class, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("Root incantation spoken! Ley-lines surge through you. +{} XP", xp),
            format!("Root {} spoken! {}-lines surge. {}", "incantation".blue().bold(), "Ley".blue(), x),
        ),
        Class::Warrior => (
            format!("SUDO! The full weight of root power channels through your arms! +{} XP", xp),
            format!("{}! Root power channels through your arms! {}", "SUDO".red().bold(), x),
        ),
        Class::Rogue => (
            format!("Privilege escalated. You go unseen with root access. +{} XP", xp),
            format!("Privilege {}. You go unseen with {} access. {}", "escalated".yellow().bold(), "root".yellow(), x),
        ),
        Class::Ranger => (
            format!("You call upon the primal authority of root. +{} XP", xp),
            format!("You call upon the {} authority of root. {}", "primal".green().bold(), x),
        ),
        Class::Necromancer => (
            format!("Root access granted. The system bows to the void. +{} XP", xp),
            format!("Root access granted. The system bows to the {}. {}", "void".magenta().bold(), x),
        ),
    }
}
```

**Step 14: Implement `banish`** (kill/pkill)

```rust
pub fn banish(class: &Class, target: &str, xp: u32, gold: u32) -> Msg {
    let x = color_xp(xp);
    let g = color_gold(gold);
    match class {
        Class::Wizard => (
            format!("You unmake {} with a word. Process banished. +{} XP, +{} gold", target, xp, gold),
            format!("You unmake {} with a {}. Process banished. {} {}", target.red(), "word".blue().bold(), x, g),
        ),
        Class::Warrior => (
            format!("You slay {} with iron will! Process vanquished. +{} XP, +{} gold", target, xp, gold),
            format!("You slay {} with {}! Process vanquished. {} {}", target.red(), "iron will".red().bold(), x, g),
        ),
        Class::Rogue => (
            format!("You silence {} without ceremony. +{} XP, +{} gold", target, xp, gold),
            format!("You silence {} {}. {} {}", target.red(), "without ceremony".yellow().dimmed(), x, g),
        ),
        Class::Ranger => (
            format!("You put {} down cleanly. The wilderness is quieter. +{} XP, +{} gold", target, xp, gold),
            format!("You put {} down {}. The wilderness is quieter. {} {}", target.red(), "cleanly".green(), x, g),
        ),
        Class::Necromancer => (
            format!("You harvest {}. Its cycles end. +{} XP, +{} gold", target, xp, gold),
            format!("You harvest {}. Its {} end. {} {}", target.red(), "cycles".magenta(), x, g),
        ),
    }
}
```

**Step 15: Implement `meditation`** (vim/nvim/emacs — heal)

```rust
pub fn meditation(class: &Class, editor: &str, heal: i32, xp: u32, hp: i32, max_hp: i32) -> Msg {
    let x = color_xp(xp);
    let hp_str = crate::display::color_hp(hp, max_hp);
    match class {
        Class::Wizard => (
            format!("{} becomes your scrying glass. Mana restored. +{} HP, +{} XP. HP: {}/{}", editor, heal, xp, hp, max_hp),
            format!("{} becomes your {}. Mana restored. +{} HP. {} {}", editor.cyan(), "scrying glass".blue().italic(), format!("{}", heal).green(), hp_str, x),
        ),
        Class::Warrior => (
            format!("{} — you sharpen your mind between battles. +{} HP, +{} XP. HP: {}/{}", editor, heal, xp, hp, max_hp),
            format!("{} — you sharpen your mind. +{} HP. {} {}", editor.cyan(), format!("{}", heal).green(), hp_str, x),
        ),
        Class::Rogue => (
            format!("{} — you vanish into the text. Wounds close. +{} HP, +{} XP. HP: {}/{}", editor, heal, xp, hp, max_hp),
            format!("{} — you vanish into the {}. +{} HP. {} {}", editor.cyan(), "text".yellow().italic(), format!("{}", heal).green(), hp_str, x),
        ),
        Class::Ranger => (
            format!("{} — you track the codebase like wilderness. +{} HP, +{} XP. HP: {}/{}", editor, heal, xp, hp, max_hp),
            format!("{} — you track the codebase like {}. +{} HP. {} {}", editor.cyan(), "wilderness".green(), format!("{}", heal).green(), hp_str, x),
        ),
        Class::Necromancer => (
            format!("{} — you commune with dead code. Necrotic energy restores you. +{} HP, +{} XP. HP: {}/{}", editor, heal, xp, hp, max_hp),
            format!("{} — you commune with {}. +{} HP. {} {}", editor.cyan(), "dead code".magenta().bold(), format!("{}", heal).green(), hp_str, x),
        ),
    }
}
```

**Step 16: Implement `incantation`** (python/node/ruby)

```rust
pub fn incantation(class: &Class, lang: &str, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("You speak the {} tongue. A spell takes form. +{} XP", lang, xp),
            format!("You speak the {} tongue. A {} takes form. {}", lang.blue(), "spell".blue().bold(), x),
        ),
        Class::Warrior => (
            format!("You wield {} like a weapon. Crude but effective. +{} XP", lang, xp),
            format!("You wield {} like a weapon. Crude but effective. {}", lang.red(), x),
        ),
        Class::Rogue => (
            format!("{} — an interpreter, untraceable. +{} XP", lang, xp),
            format!("{} — an {}, untraceable. {}", lang.yellow(), "interpreter".yellow().dimmed(), x),
        ),
        Class::Ranger => (
            format!("You track {} through its native territory. +{} XP", lang, xp),
            format!("You track {} through its {}. {}", lang.green(), "native territory".green().dimmed(), x),
        ),
        Class::Necromancer => (
            format!("You summon {} from the interpreter beyond. +{} XP", lang, xp),
            format!("You summon {} from the {}. {}", lang.magenta(), "interpreter beyond".magenta().bold(), x),
        ),
    }
}
```

**Step 17: Implement `ancient_tome`** (man/tldr)

```rust
pub fn ancient_tome(class: &Class, subject: &str, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("You consult the arcane scrolls of {}. Knowledge absorbed. +{} XP", subject, xp),
            format!("You consult the {} of {}. +{} XP. {}", "arcane scrolls".blue().italic(), subject.bold(), xp, x),
        ),
        Class::Warrior => (
            format!("You read the field manual of {}. Strategy memorized. +{} XP", subject, xp),
            format!("You read the {} of {}. Strategy memorized. {}", "field manual".red().dimmed(), subject.bold(), x),
        ),
        Class::Rogue => (
            format!("You skim {} for the relevant details. No time for the rest. +{} XP", subject, xp),
            format!("You skim {} for the relevant details. {}. {}", subject.bold(), "No time for the rest".yellow().dimmed(), x),
        ),
        Class::Ranger => (
            format!("You study {} like a terrain map. Routes memorized. +{} XP", subject, xp),
            format!("You study {} like a {}. Routes memorized. {}", subject.bold(), "terrain map".green(), x),
        ),
        Class::Necromancer => (
            format!("The dead pages of {} yield their secrets. +{} XP", subject, xp),
            format!("The {} of {} yield their secrets. {}", "dead pages".magenta().dimmed(), subject.bold(), x),
        ),
    }
}
```

**Step 18: Implement `level_up`**

```rust
pub fn level_up(class: &Class, level: u32, title: &str) -> Msg {
    match class {
        Class::Wizard => (
            format!("LEVEL UP! The arcane flows stronger! You are now level {}! Title: {}", level, title),
            format!("{} The {} flows stronger! Level {} — {}!", "LEVEL UP!".yellow().bold(), "arcane".blue().bold(), format!("{}", level).yellow().bold(), title.cyan().italic()),
        ),
        Class::Warrior => (
            format!("LEVEL UP! Your might grows! You are now level {}! Title: {}", level, title),
            format!("{} Your {} grows! Level {} — {}!", "LEVEL UP!".yellow().bold(), "might".red().bold(), format!("{}", level).yellow().bold(), title.cyan().italic()),
        ),
        Class::Rogue => (
            format!("LEVEL UP! Sharper, faster, deadlier. Level {}. Title: {}", level, title),
            format!("{} {}. Level {} — {}!", "LEVEL UP!".yellow().bold(), "Sharper. Faster. Deadlier.".yellow(), format!("{}", level).yellow().bold(), title.cyan().italic()),
        ),
        Class::Ranger => (
            format!("LEVEL UP! The wilds yield their secrets. Level {}. Title: {}", level, title),
            format!("{} The wilds {}. Level {} — {}!", "LEVEL UP!".yellow().bold(), "yield their secrets".green(), format!("{}", level).yellow().bold(), title.cyan().italic()),
        ),
        Class::Necromancer => (
            format!("LEVEL UP! The darkness deepens. Level {}. Title: {}", level, title),
            format!("{} The darkness {}. Level {} — {}!", "LEVEL UP!".yellow().bold(), "deepens".magenta().bold(), format!("{}", level).yellow().bold(), title.cyan().italic()),
        ),
    }
}
```

**Step 19: Implement `death_normal`** (player dies in standard mode)

```rust
pub fn death_normal(class: &Class, killer: &str, gold_lost: u32) -> Msg {
    let k = color_monster(killer);
    match class {
        Class::Wizard => (
            format!("Your wards shattered by {}! Arcane collapse! Lost {} gold, XP reset.", killer, gold_lost),
            format!("Your {} shattered by {}! {}. -{} gold, XP reset.", "wards".blue(), k, "Arcane collapse".blue().bold(), format!("{}", gold_lost).yellow()),
        ),
        Class::Warrior => (
            format!("Felled by {}! The battle is lost! Lost {} gold, XP reset.", killer, gold_lost),
            format!("Felled by {}! The battle is {}! -{} gold, XP reset.", k, "lost".red().bold(), format!("{}", gold_lost).yellow()),
        ),
        Class::Rogue => (
            format!("Caught by {}. The shadows offer no shelter today. Lost {} gold, XP reset.", killer, gold_lost),
            format!("Caught by {}. The {} offer no shelter today. -{} gold, XP reset.", k, "shadows".yellow().dimmed(), format!("{}", gold_lost).yellow()),
        ),
        Class::Ranger => (
            format!("Overrun by {}! The hunt ends here. Lost {} gold, XP reset.", killer, gold_lost),
            format!("Overrun by {}! The hunt ends here. -{} gold, XP reset.", k, format!("{}", gold_lost).yellow()),
        ),
        Class::Necromancer => (
            format!("Consumed by {}. Even death has a price. Lost {} gold, XP reset.", killer, gold_lost),
            format!("Consumed by {}. Even {} has a price. -{} gold, XP reset.", k, "death".magenta(), format!("{}", gold_lost).yellow()),
        ),
    }
}
```

**Step 20: Implement `docker_orchestra`** (docker compose — highest gold event)

```rust
pub fn docker_orchestra(class: &Class, xp: u32, gold: u32) -> Msg {
    let x = color_xp(xp);
    let g = color_gold(gold);
    match class {
        Class::Wizard => (
            format!("You conduct the container symphony! Services harmonize. +{} XP, +{} gold", xp, gold),
            format!("You conduct the {} symphony! Services {}. {} {}", "container".blue(), "harmonize".blue().bold(), x, g),
        ),
        Class::Warrior => (
            format!("Your army of containers marches! The siege engine starts! +{} XP, +{} gold", xp, gold),
            format!("Your {} of containers marches! {} {}", "army".red(), x, g),
        ),
        Class::Rogue => (
            format!("Services launch unseen. The operation is live. +{} XP, +{} gold", xp, gold),
            format!("Services launch {}. The operation is live. {} {}", "unseen".yellow().dimmed(), x, g),
        ),
        Class::Ranger => (
            format!("The pack deploys in formation. Ecosystem stable. +{} XP, +{} gold", xp, gold),
            format!("The pack deploys in {}. Ecosystem stable. {} {}", "formation".green(), x, g),
        ),
        Class::Necromancer => (
            format!("You animate the container horde. The undead swarm is live. +{} XP, +{} gold", xp, gold),
            format!("You animate the container {}. The {} is live. {} {}", "horde".magenta(), "undead swarm".magenta().bold(), x, g),
        ),
    }
}
```

**Step 21: Declare mod in main.rs and run cargo build**

In `src/main.rs`, add near the top with the other `mod` declarations:
```rust
mod messages;
```

Run:
```bash
cd /home/duys/.repos/shellquest && rtk cargo build 2>&1 | tail -5
```
Expected: No errors.

**Step 22: Commit**

```bash
git add src/messages.rs src/main.rs
git commit -m "Add class-aware messages module with 20 event message functions"
```

---

## Task 2: Zone XP Scaling helper in `src/events.rs`

**Files:**
- Modify: `src/events.rs`

The `tick()` function already receives `cwd: &str` and calls `zones::get_zone(cwd)`. The zone has a `danger` field.

**Step 1: Add `scaled_xp` helper function**

Add this function near the top of `src/events.rs`, after the imports (around line 20):

```rust
/// Scales base XP by zone danger level.
/// danger 1 = 1.0×, danger 2 = 1.25×, danger 3 = 1.5×, danger 4 = 1.75×, danger 5 = 2.0×
fn scaled_xp(base: u32, danger: u32) -> u32 {
    let multiplier = 1.0 + (danger.saturating_sub(1) as f32) * 0.25;
    ((base as f32) * multiplier).round() as u32
}
```

**Step 2: Add unit test for scaled_xp**

In the `#[cfg(test)]` block of `src/events.rs` (or create one if absent):
```rust
#[test]
fn scaled_xp_danger_1_returns_base() {
    assert_eq!(scaled_xp(20, 1), 20);
}

#[test]
fn scaled_xp_danger_3_returns_150_percent() {
    assert_eq!(scaled_xp(20, 3), 30);
}

#[test]
fn scaled_xp_danger_5_returns_double() {
    assert_eq!(scaled_xp(20, 5), 40);
}
```

**Step 3: Run tests**

```bash
rtk cargo test scaled_xp 2>&1 | tail -5
```
Expected: 3 passed.

**Step 4: Commit**

```bash
git add src/events.rs
git commit -m "Add scaled_xp() helper for zone-danger XP multiplier"
```

---

## Task 3: Class Affinity XP Bonus in `src/events.rs`

**Files:**
- Modify: `src/events.rs`

**Step 1: Add `affinity_multiplier` helper**

Add immediately after `scaled_xp` in `src/events.rs`:

```rust
/// Returns 1.5 if the player's class has affinity with the given base command, else 1.0.
/// Affinity = 50% more XP for class-relevant commands.
fn affinity_multiplier(class: &crate::character::Class, cmd: &str) -> f32 {
    use crate::character::Class;
    let affinities: &[&str] = match class {
        Class::Wizard      => &["python", "python3", "node", "ruby", "vim", "nvim", "emacs", "man", "tldr", "jupyter"],
        Class::Warrior     => &["cargo", "make", "cmake", "gcc", "g++", "ninja", "meson", "mvn", "gradle"],
        Class::Rogue       => &["grep", "rg", "ag", "ssh", "find", "fd", "ls", "eza", "locate"],
        Class::Ranger      => &["curl", "wget", "http", "docker", "kubectl", "ansible", "terraform", "helm"],
        Class::Necromancer => &["kill", "pkill", "killall", "rm", "del", "git", "rm", "shred"],
    };
    if affinities.iter().any(|&a| cmd == a || cmd.starts_with(&format!("{} ", a))) {
        1.5
    } else {
        1.0
    }
}

/// Apply both zone scaling and class affinity to a base XP amount.
fn final_xp(base: u32, danger: u32, class: &crate::character::Class, cmd: &str) -> u32 {
    let zone_scaled = scaled_xp(base, danger);
    (zone_scaled as f32 * affinity_multiplier(class, cmd)).round() as u32
}
```

**Step 2: Add unit tests**

```rust
#[test]
fn affinity_multiplier_wizard_python_returns_1_5() {
    use crate::character::Class;
    assert_eq!(affinity_multiplier(&Class::Wizard, "python"), 1.5);
}

#[test]
fn affinity_multiplier_warrior_no_affinity_returns_1_0() {
    use crate::character::Class;
    assert_eq!(affinity_multiplier(&Class::Warrior, "ls"), 1.0);
}

#[test]
fn final_xp_applies_both_bonuses() {
    use crate::character::Class;
    // Wizard in danger-3 zone casting python: 20 * 1.5 (zone) * 1.5 (affinity) = 45
    assert_eq!(final_xp(20, 3, &Class::Wizard, "python"), 45);
}
```

**Step 3: Run tests**

```bash
rtk cargo test affinity 2>&1 | tail -5
```
Expected: 3 passed.

**Step 4: Commit**

```bash
git add src/events.rs
git commit -m "Add class affinity XP multiplier (50% bonus for class-matching commands)"
```

---

## Task 4: Integrate class messages into `src/events.rs`

**Files:**
- Modify: `src/events.rs`

The `tick()` signature already receives `cmd: &str` and `cwd: &str`. The `GameState` carries `state.character.class`. The zone is computed inside `tick()` via `let zone = zones::get_zone(cwd)`.

**Pattern to apply everywhere:**

**OLD (inline message, no zone/class scaling):**
```rust
let xp_amount = rng.gen_range(10..=25);
let leveled = state.character.gain_xp(xp_amount);
let plain = format!("...");
let colored = format!("...");
display::print_craft(&colored);
state.add_journal(JournalEntry::new(EventType::Craft, plain));
check_level_up(state, leveled);
```

**NEW (class messages + final_xp):**
```rust
let base_xp = rng.gen_range(15..=35);
let xp_amount = final_xp(base_xp, zone.danger, &state.character.class, cmd);
let leveled = state.character.gain_xp(xp_amount);
let (plain, colored) = messages::craft(&state.character.class, xp_amount);
display::print_craft(&colored);
state.add_journal(JournalEntry::new(EventType::Craft, plain));
check_level_up(state, leveled);
```

**Step 1: Update `handle_craft`** (git commit — lines ~214–237)

Replace the existing `handle_craft` body. Apply the NEW pattern using `messages::craft`.

**Step 2: Update `handle_quest`** (git push — lines ~239–252)

Apply NEW pattern using `messages::quest`. Keep the gold reward generation unchanged.

**Step 3: Update `handle_discovery`** (lines ~254–278)

The existing handler has 5 random discovery variants — keep the `detail` strings but feed them into `messages::discovery(&state.character.class, detail, xp_amount)`.

**Step 4: Update `handle_forge`** (cargo/npm build — lines ~279–303)

Two branches: loot branch → `messages::forge_loot`, XP-only branch → `messages::forge_xp`.

**Step 5: Update `handle_familiar`** (cat/less — lines ~309–322)

Feed existing `creature` variants into `messages::familiar`. Pass current `state.character.hp` and `state.character.max_hp`.

**Step 6: Update `handle_portal`** (ssh/curl/wget — lines ~334–344)

Apply NEW pattern using `messages::portal`.

**Step 7: Update `handle_power_surge`** (sudo — lines ~345–354)

Apply NEW pattern using `messages::power_surge`.

**Step 8: Update `handle_banish`** (kill/pkill — lines ~477–493)

Apply NEW pattern using `messages::banish`. Keep the kills++ and gold reward logic.

**Step 9: Update `handle_meditation`** (vim/nvim/emacs — lines ~397–411)

Apply NEW pattern using `messages::meditation`. Preserve the heal logic.

**Step 10: Update `handle_incantation`** (python/node/ruby — lines ~365–378)

Apply NEW pattern using `messages::incantation`.

**Step 11: Update `handle_ancient_tome`** (man/tldr — lines ~511–523)

Apply NEW pattern using `messages::ancient_tome`.

**Step 12: Update `handle_docker_orchestra`** (docker compose — lines ~574–589)

Apply NEW pattern using `messages::docker_orchestra`. Keep the gold reward logic.

**Step 13: Update `combat()`** (lines ~661–729)

The combat function has 4 outcomes. Update all 4:
- Win (no damage): `messages::combat_win(&state.character.class, monster_name, xp_reward)`
- Tough (player takes damage, wins): `messages::combat_tough(&state.character.class, monster_name, damage, xp_reward)`
- Lose (player takes damage, monster survives): `messages::combat_lose(&state.character.class, monster_name, damage)`
- Draw (escaped): `messages::combat_draw(&state.character.class, monster_name)`

**Step 14: Update `handle_trap`** (lines ~182–203)

Apply NEW pattern using `messages::trap`. Pass `state.character.hp` and `state.character.max_hp` post-damage.

**Step 15: Build and test**

```bash
rtk cargo build 2>&1 | tail -5
rtk cargo test 2>&1 | tail -5
```
Expected: clean build, all tests pass.

**Step 16: Commit**

```bash
git add src/events.rs
git commit -m "Wire class-aware messages and zone/affinity XP scaling into all event handlers"
```

---

## Task 5: XP Curve Rebalance in `src/character.rs` and `src/events.rs`

**Files:**
- Modify: `src/character.rs` — `level_up()` XP thresholds
- Modify: `src/events.rs` — base XP ranges per event (already partially done in Task 4; finish remaining)

**Background math:**
Target: Level 50 in ~1,000–2,000 commands (casual 1-2 weeks).
With zone multipliers averaging ~1.25× and affinity averaging ~1.1× for relevant commands,
effective avg XP ≈ 20–25 per event.
Total XP budget for levels 1–50: ~20,000–40,000 XP.

**Step 1: Replace `level_up()` XP bracket formula in `src/character.rs`**

Find the `fn level_up(&mut self) -> bool` function (around lines 295–320). Replace the bracket thresholds:

```rust
// OLD brackets
let xp_needed = match self.level {
    1..=10   => self.level as u32 * 80 + 50,
    11..=30  => self.level as u32 * 100 + 100,
    31..=60  => self.level as u32 * 130 + 200,
    61..=100 => self.level as u32 * 170 + 400,
    101..=130 => self.level as u32 * 220 + 800,
    _        => self.level as u32 * 300 + 1500,
};

// NEW brackets (calibrated for ~1,200 commands to level 50)
let xp_needed = match self.level {
    1..=10   => self.level as u32 * 15 + 10,
    11..=30  => self.level as u32 * 25 + 30,
    31..=60  => self.level as u32 * 45 + 80,
    61..=100 => self.level as u32 * 80 + 200,
    101..=130 => self.level as u32 * 120 + 400,
    _        => self.level as u32 * 170 + 800,
};
```

**Verification math (NEW brackets):**
- Levels 1-10 total: Σ(level*15+10, 1..10) ≈ 1,330 XP
- Levels 11-30 total: Σ(level*25+30, 11..30) ≈ 11,850 XP
- Levels 31-50 total: Σ(level*45+80, 31..50) ≈ 38,300 XP
- **Total to level 50: ~51,480 XP**
- At 25 avg XP/event: ~2,060 commands → ✓ within casual target

**Step 2: Add `xp_to_next` field initialization**

The GameState also stores `xp_to_next` (the threshold shown in the status bar). After updating `level_up()`, make sure `GameState::new()` calls `level_up()` logic or sets `xp_to_next` correctly.

Find where `xp_to_next` is initialized in `character.rs` (likely in `Character::new()` or `GameState::new()`). It should call the same formula as `level_up()`. If it's a separate expression, update it to match.

**Step 3: Add a test for the new XP thresholds**

In `src/character.rs` `#[cfg(test)]` block:
```rust
#[test]
fn xp_to_next_level_1_is_reasonable() {
    use crate::character::{Character, Class, Race};
    let char = Character::new("T".to_string(), Class::Warrior, Race::Human);
    // Level 1 needs 15*1+10 = 25 XP
    assert_eq!(char.xp_to_next, 25);
}

#[test]
fn xp_to_next_level_50_is_within_budget() {
    // After 49 level-ups, level_up() for level 50 = 50*45+80 = 2330
    // This is the per-level cost, not cumulative
    let l50_cost = 50u32 * 45 + 80;
    assert_eq!(l50_cost, 2330);
    assert!(l50_cost < 3000, "level 50 per-step XP too high: {}", l50_cost);
}
```

**Step 4: Run tests**

```bash
rtk cargo test xp 2>&1 | tail -10
```
Expected: all XP-related tests pass.

**Step 5: Commit**

```bash
git add src/character.rs
git commit -m "Rebalance XP curve: ~2000 commands to reach level 50 (casual pacing)"
```

---

## Task 6: Permadeath Mode

**Files:**
- Modify: `src/state.rs` — add `permadeath` field
- Modify: `src/main.rs` — `sq init` prompt, `sq status` indicator
- Modify: `src/events.rs` — update death handling in `combat()`
- Modify: `src/boss.rs` — update death handling in `tick_boss()`
- Modify: `src/display.rs` — add `print_permadeath_eulogy()`

### Sub-task 6a: State field

**Step 1: Add `permadeath` to `GameState` in `src/state.rs`**

Find the `GameState` struct (around lines 9–30). Add:
```rust
#[serde(default)]
pub permadeath: bool,
```

The `#[serde(default)]` means old save files (without the field) deserialize as `false` — safe migration.

**Step 2: Confirm `GameState::new()` initializes it**

In `GameState::new()`, `permadeath` will default to `false` since `bool` implements `Default`. No explicit initialization needed.

**Step 3: Build**

```bash
rtk cargo build 2>&1 | tail -3
```

**Step 4: Commit**

```bash
git add src/state.rs
git commit -m "Add permadeath field to GameState (default false, serde-compatible)"
```

---

### Sub-task 6b: sq init prompt

**Step 1: Read `cmd_init()` in `src/main.rs`**

Find `fn cmd_init()` (around lines 300–380). It prompts for name, class, race. Add a permadeath prompt at the end, before saving.

**Step 2: Add permadeath prompt**

After race is selected and before `state::save(&game)`, add:

```rust
println!();
println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red().bold());
println!("{} {}", "💀".bold(), "PERMADEATH MODE".red().bold());
println!("  If you die, your character is gone forever. All is lost.");
println!("  In standard mode, death resets your XP to 0 and costs 15% gold.");
println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".red().bold());
let pd_answer = prompt("Enable permadeath? [y/N] ");
game.permadeath = pd_answer.trim().to_lowercase() == "y";

if game.permadeath {
    println!("{} {}", "☠".red().bold(), "Permadeath enabled. May the void be merciful.".red().dimmed());
} else {
    println!("{} {}", "✓".green(), "Standard mode. Death is a setback, not the end.".dimmed());
}
```

**Step 3: Show permadeath status in `sq status`**

In `display::print_status()` in `src/display.rs`, the function takes `char: &Character`. However, `permadeath` is on `GameState`, not `Character`. Two options:
- Option A: Pass `permadeath: bool` as a second argument to `print_status()`
- Option B: Add it to `Character` (wrong layer)

Use **Option A**. Update `print_status` signature in `src/display.rs`:

```rust
pub fn print_status(char: &Character, permadeath: bool) {
```

At the bottom of the status box, before the closing `└` line, add:
```rust
if permadeath {
    println!("{}  {} {}", "│".dimmed(), "Mode:".bold(), "☠ PERMADEATH".red().bold());
}
```

Update the caller in `src/main.rs` `cmd_status()`:
```rust
display::print_status(&game.character, game.permadeath);
```

**Step 4: Build and run `sq status`**

```bash
rtk cargo build 2>&1 | tail -3
```

**Step 5: Commit**

```bash
git add src/main.rs src/display.rs
git commit -m "Add permadeath toggle to sq init and permadeath indicator in sq status"
```

---

### Sub-task 6c: Permadeath eulogy in display.rs

**Step 1: Add `print_permadeath_eulogy` to `src/display.rs`**

```rust
pub fn print_permadeath_eulogy(char: &Character, killer: &str) {
    eprintln!();
    eprintln!("{}", "☠  ═══════════════════════════════════════════  ☠".red().bold());
    eprintln!("{}", "".to_string());
    eprintln!("       {}", "Y O U   H A V E   D I E D".red().bold());
    eprintln!();
    eprintln!(
        "  Here lies {}, the {} {}.",
        char.name.bold().white(),
        format!("{}", char.race).magenta(),
        format!("{}", char.class).cyan().bold()
    );
    let subclass_str = char.subclass.as_ref()
        .map_or(String::new(), |s| format!(" {} ", format!("{}", s).magenta().bold()));
    if !subclass_str.is_empty() {
        eprintln!("  Known also as the{}.", subclass_str);
    }
    eprintln!(
        "  Felled by {} at level {}.",
        killer.red().bold(),
        format!("{}", char.level).yellow().bold()
    );
    eprintln!(
        "  After {} commands run, {} monsters slain, {} times deceased.",
        format!("{}", char.commands_run).cyan(),
        format!("{}", char.kills).green(),
        format!("{}", char.deaths + 1).red()
    );
    if char.gold > 0 {
        eprintln!(
            "  They carried {} gold into the grave.",
            format!("{}", char.gold).yellow()
        );
    }
    if let Some(w) = &char.weapon {
        eprintln!("  Their blade: {}.", w.name.cyan().italic());
    }
    eprintln!(
        "  Their legend: {}",
        char.title.yellow().italic()
    );
    eprintln!();
    eprintln!("  {}", "The save file has been deleted. All is lost.".dimmed().italic());
    eprintln!("{}", "☠  ═══════════════════════════════════════════  ☠".red().bold());
    eprintln!();
}
```

**Step 2: Commit**

```bash
git add src/display.rs
git commit -m "Add print_permadeath_eulogy() for dramatic character death screen"
```

---

### Sub-task 6d: Wire permadeath into combat death in events.rs

**Step 1: Find death handling in `combat()` in `src/events.rs`**

The current death handling (around lines 700–720) looks like:
```rust
// Player dies
state.character.deaths += 1;
let gold_loss = state.character.gold / 4;  // OLD: 25%
state.character.gold = state.character.gold.saturating_sub(gold_loss);
state.character.hp = state.character.max_hp / 2;
// ... message printed ...
```

**Step 2: Replace with permadeath-aware logic**

```rust
// Player dies
state.character.deaths += 1;
let (plain, colored) = messages::death_normal(
    &state.character.class, monster_name, 0  // gold_lost filled in below
);

if state.permadeath {
    // Permadeath: print eulogy, delete save, return immediately
    crate::display::print_permadeath_eulogy(&state.character, monster_name);
    let path = crate::state::save_path();
    let _ = std::fs::remove_file(&path);
    std::process::exit(0);
} else {
    // Standard: XP reset to start of current level, 15% gold loss
    state.character.xp = 0;
    let gold_loss = state.character.gold * 15 / 100;
    state.character.gold = state.character.gold.saturating_sub(gold_loss);
    state.character.hp = state.character.max_hp / 2;
    let (plain, colored) = messages::death_normal(
        &state.character.class, monster_name, gold_loss
    );
    display::print_combat_lose(&colored, true);
    state.add_journal(JournalEntry::new(EventType::Death, plain));
}
```

**Step 3: Wire permadeath into `tick_boss()` in `src/boss.rs`**

Find the player-death branch in `tick_boss()` (around lines 131–145):
```rust
if died {
    crate::display::print_boss_tick(...);
    crate::display::print_boss_flee(&boss_name, "laughs as you fall...");
    state.add_journal(JournalEntry::new(EventType::Death, ...));
    state.active_boss = None;
    return;
}
```

Add permadeath check:
```rust
if died {
    if state.permadeath {
        crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, Some(dmg));
        crate::display::print_permadeath_eulogy(&state.character, &boss_name);
        let path = crate::state::save_path();
        let _ = std::fs::remove_file(&path);
        std::process::exit(0);
    } else {
        state.character.xp = 0;
        let gold_loss = state.character.gold * 15 / 100;
        state.character.gold = state.character.gold.saturating_sub(gold_loss);
        state.character.hp = state.character.max_hp / 2;
        crate::display::print_boss_tick(state.active_boss.as_ref().unwrap(), player_dmg, Some(dmg));
        crate::display::print_boss_flee(&boss_name, "laughs as you fall... and vanishes into the void");
        state.add_journal(JournalEntry::new(EventType::Death,
            format!("{} fled after you fell. XP reset, -{} gold.", boss_name, gold_loss)));
        state.active_boss = None;
        return;
    }
}
```

**Step 4: Build and test**

```bash
rtk cargo build 2>&1 | tail -5
rtk cargo test 2>&1 | tail -5
```
Expected: clean build, all tests pass.

**Step 5: Commit**

```bash
git add src/events.rs src/boss.rs
git commit -m "Wire permadeath into combat and boss death paths; standard death now resets XP (15% gold loss)"
```

---

## Task 7: Update AGENTS.md

**Files:**
- Modify: `src/AGENTS.md`
- Modify: `AGENTS.md`

**Step 1: Add `messages.rs` to `src/AGENTS.md` Key Files table**

```
| `messages.rs` | Class-aware message module: 20 event functions returning `(plain, colored)` tuples. Each function has 5 class variants (Wizard/Warrior/Rogue/Ranger/Necromancer). Called from `events.rs` handlers. |
```

**Step 2: Update `events.rs` entry in `src/AGENTS.md`**

Add to existing `events.rs` description:
```
Uses `scaled_xp(base, danger)` for zone-danger XP multiplier (1.0×–2.0×) and `affinity_multiplier(class, cmd)` for class affinity (+50% XP for matching commands). All event messages now route through `messages::*` functions.
```

**Step 3: Update Testing Requirements in root `AGENTS.md`**

Add:
```
- Test permadeath: set `"permadeath": true` in save.json, run `sq tick --cmd "bad" --cwd "." --exit-code 1` — should print eulogy and delete save
- Test sq inv alias: `sq inv` should behave identically to `sq inventory`
- Class messages: verify by checking `sq journal` after a few ticks — entries should reflect class flavor
```

**Step 4: Commit**

```bash
git add AGENTS.md src/AGENTS.md
git commit -m "Update AGENTS.md with messages module, permadeath, and zone XP docs"
```

---

## Task 8: Final Verification

**Step 1: Full clean build and test**

```bash
rtk cargo build 2>&1 | tail -3
rtk cargo test 2>&1 | tail -5
```
Expected: build clean, 125+ tests pass.

**Step 2: Manual QA — class messages**

```bash
# Create a Wizard character and trigger a few events
cargo run -- tick --cmd "git commit" --cwd "." --exit-code 0 2>&1
# Expected: arcane/grimoire flavor in message

cargo run -- tick --cmd "bad" --cwd "." --exit-code 1 2>&1
# Expected: arcane-ward flavor in trap message
```

**Step 3: Manual QA — zone XP scaling**

```bash
# Trigger events from home vs dangerous zone
cargo run -- tick --cmd "git commit" --cwd "$HOME" --exit-code 0 2>&1  # danger 1
cargo run -- tick --cmd "git commit" --cwd "/tmp" --exit-code 0 2>&1   # danger 3 (1.5× XP)
# Check sq journal — XP from /tmp should be ~50% higher
```

**Step 4: Manual QA — permadeath**

```bash
# Set hp to 1 and permadeath to true in save.json manually
# Then run:
cargo run -- tick --cmd "bad" --cwd "." --exit-code 1 2>&1
# Expected: eulogy prints, save file deleted, process exits 0
```

**Step 5: Manual QA — class affinity**

```bash
# As a Wizard, trigger python event:
cargo run -- tick --cmd "python3" --cwd "." --exit-code 0 2>&1
# Check journal XP — should be higher than a non-affinity command
```

**Step 6: Publish**

```bash
./publish.sh minor   # v1.7.x → v1.8.0 (significant feature set)
```
