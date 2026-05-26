CREATE TABLE IF NOT EXISTS run (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    seed INTEGER NOT NULL,
    class TEXT NOT NULL,
    race TEXT NOT NULL,
    strategy TEXT NOT NULL,
    tuning_label TEXT NOT NULL,
    target_level INTEGER NOT NULL,
    max_ticks INTEGER NOT NULL,
    started_at REAL NOT NULL,
    ended_at REAL,
    final_level INTEGER,
    final_xp INTEGER,
    final_gold INTEGER,
    final_kills INTEGER,
    final_deaths INTEGER,
    final_prestige INTEGER,
    final_max_hp INTEGER,
    final_attack_power INTEGER,
    final_defense INTEGER,
    total_ticks INTEGER,
    ended_reason TEXT
);

CREATE TABLE IF NOT EXISTS tick_snapshot (
    run_id INTEGER NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    tick_no INTEGER NOT NULL,
    level INTEGER NOT NULL,
    xp INTEGER NOT NULL,
    hp INTEGER NOT NULL,
    max_hp INTEGER NOT NULL,
    gold INTEGER NOT NULL,
    kills INTEGER NOT NULL,
    deaths INTEGER NOT NULL,
    strength INTEGER NOT NULL,
    dexterity INTEGER NOT NULL,
    intelligence INTEGER NOT NULL,
    attack_power INTEGER NOT NULL,
    defense INTEGER NOT NULL,
    inventory_count INTEGER NOT NULL,
    weapon_power INTEGER,
    armor_power INTEGER,
    ring_power INTEGER,
    weapon_rarity TEXT,
    armor_rarity TEXT,
    ring_rarity TEXT,
    PRIMARY KEY(run_id, tick_no)
);

CREATE TABLE IF NOT EXISTS action_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    tick_no INTEGER NOT NULL,
    action TEXT NOT NULL,
    details TEXT,
    outcome TEXT
);

CREATE TABLE IF NOT EXISTS arena_attempt (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    tick_no INTEGER NOT NULL,
    character_level INTEGER NOT NULL,
    tier TEXT NOT NULL,
    tier_index INTEGER NOT NULL,
    entry_fee INTEGER NOT NULL,
    rounds_attempted INTEGER NOT NULL,
    rounds_won INTEGER NOT NULL,
    outcome TEXT NOT NULL,
    gold_earned INTEGER NOT NULL,
    xp_earned INTEGER NOT NULL,
    dmg_dealt INTEGER NOT NULL,
    dmg_taken INTEGER NOT NULL,
    enemy_crits INTEGER NOT NULL,
    player_crits INTEGER NOT NULL,
    player_swings INTEGER NOT NULL,
    enemy_swings INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS item_event (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL REFERENCES run(id) ON DELETE CASCADE,
    tick_no INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    item_name TEXT NOT NULL,
    rarity TEXT NOT NULL,
    slot TEXT NOT NULL,
    power INTEGER NOT NULL,
    enchant_level INTEGER NOT NULL,
    gold_cost INTEGER,
    was_equipped INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_run_class_strategy
    ON run(class, strategy, tuning_label);
CREATE INDEX IF NOT EXISTS idx_run_tuning
    ON run(tuning_label);
CREATE INDEX IF NOT EXISTS idx_tick_snapshot_level
    ON tick_snapshot(run_id, level);
CREATE INDEX IF NOT EXISTS idx_arena_attempt_tier
    ON arena_attempt(run_id, tier);
CREATE INDEX IF NOT EXISTS idx_action_log_run
    ON action_log(run_id, action);
CREATE INDEX IF NOT EXISTS idx_item_event_run
    ON item_event(run_id, event_type);
