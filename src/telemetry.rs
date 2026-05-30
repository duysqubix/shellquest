pub(crate) const SQ_DEBUG_ENV: &str = "SQ_DEBUG";

pub(crate) fn sq_debug_value_enabled(env: Option<&str>) -> bool {
    env.is_some()
}

pub(crate) fn sq_debug_enabled() -> bool {
    let env_value = std::env::var(SQ_DEBUG_ENV).ok();
    sq_debug_value_enabled(env_value.as_deref())
}

pub(crate) fn emit_encounter(
    kind: &str,
    enemy: &str,
    elite: bool,
    dmg_dealt: i32,
    dmg_taken: i32,
    outcome: &str,
    xp: u32,
    gold: u32,
) {
    if !sq_debug_enabled() {
        return;
    }

    eprintln!(
        "{}",
        format_encounter_line(kind, enemy, elite, dmg_dealt, dmg_taken, outcome, xp, gold)
    );
}

pub(crate) fn format_encounter_line(
    kind: &str,
    enemy: &str,
    elite: bool,
    dmg_dealt: i32,
    dmg_taken: i32,
    outcome: &str,
    xp: u32,
    gold: u32,
) -> String {
    format!(
        "SQ_ENCOUNTER kind={} enemy={} elite={} dmg_dealt={} dmg_taken={} outcome={} xp={} gold={}",
        kind,
        hex_encode_utf8(enemy),
        i32::from(elite),
        dmg_dealt,
        dmg_taken,
        outcome,
        xp,
        gold
    )
}

pub(crate) fn hex_encode_utf8(text: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(text.len() * 2);
    for byte in text.as_bytes() {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_decode_utf8(encoded: &str) -> String {
        let mut bytes = Vec::with_capacity(encoded.len() / 2);
        let chars: Vec<char> = encoded.chars().collect();
        for pair in chars.chunks(2) {
            let high = pair[0].to_digit(16).unwrap();
            let low = pair[1].to_digit(16).unwrap();
            bytes.push(((high << 4) + low) as u8);
        }
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn encounter_line_uses_machine_parseable_hex_enemy() {
        let line = format_encounter_line("mob", "Pointer Panther", true, 42, 7, "kill", 99, 0);

        assert_eq!(
            line,
            "SQ_ENCOUNTER kind=mob enemy=506f696e7465722050616e74686572 elite=1 dmg_dealt=42 dmg_taken=7 outcome=kill xp=99 gold=0"
        );
    }

    #[test]
    fn hex_enemy_roundtrips_utf8_names() {
        let enemy = "Lord of /dev/null ⚔️";
        let encoded = hex_encode_utf8(enemy);

        assert_eq!(hex_decode_utf8(&encoded), enemy);
    }
}
