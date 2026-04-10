#![allow(dead_code)]

use crate::character::Class;
use colored::*;
use crate::display::{color_xp, color_gold, color_damage, color_monster};

type Msg = (String, String);

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

pub fn ancient_tome(class: &Class, subject: &str, xp: u32) -> Msg {
    let x = color_xp(xp);
    match class {
        Class::Wizard => (
            format!("You consult the arcane scrolls of {}. Knowledge absorbed. +{} XP", subject, xp),
            format!("You consult the {} of {}. {}", "arcane scrolls".blue().italic(), subject.bold(), x),
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
