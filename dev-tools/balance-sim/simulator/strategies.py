from __future__ import annotations

import random
from dataclasses import dataclass
from typing import Callable

TIER_REQUIREMENTS = [
    {"index": 1, "name": "Pit",       "min_level": 0,   "min_prestige": 0, "or_unlock": False},
    {"index": 2, "name": "Gauntlet",  "min_level": 25,  "min_prestige": 1, "or_unlock": True},
    {"index": 3, "name": "Colosseum", "min_level": 60,  "min_prestige": 1, "or_unlock": True},
    {"index": 4, "name": "Abyssal",   "min_level": 100, "min_prestige": 2, "or_unlock": True},
    {"index": 5, "name": "Godslayer", "min_level": 150, "min_prestige": 3, "or_unlock": False},
]


def tier_unlocked(level: int, prestige: int, tier: dict) -> bool:
    level_ok = level >= tier["min_level"]
    prestige_ok = prestige >= tier["min_prestige"]
    return (level_ok or prestige_ok) if tier["or_unlock"] else (level_ok and prestige_ok)


def best_unlocked_tier(level: int, prestige: int) -> dict | None:
    best = None
    for tier in TIER_REQUIREMENTS:
        if tier_unlocked(level, prestige, tier):
            best = tier
    return best


def estimate_arena_fee(level: int, prestige: int, gold: int, tier_index: int) -> int:
    if tier_index == 1:
        return max(100, level * 10, gold // 8)
    if tier_index == 2:
        return max(100, level * 18 + prestige * 50, gold // 8)
    if tier_index == 3:
        return max(100, level * 28 + prestige * 75, gold // 8)
    if tier_index == 4:
        return max(100, level * 40 + prestige * 100, gold // 8)
    return max(100, level * 60 + prestige * 200, gold // 8)


@dataclass
class Decision:
    action: str
    payload: dict


class Strategy:
    name: str = "base"

    def decide(self, state: dict, save: dict, rng: random.Random) -> Decision:
        raise NotImplementedError


class GreedyStrategy(Strategy):
    name = "greedy"

    def decide(self, state, save, rng):
        c = save["character"]
        gold = state["gold"]
        level = state["level"]
        prestige = state.get("prestige", 0)

        for idx, item in enumerate(c.get("inventory", [])):
            slot = item["slot"].lower()
            equipped = c.get(slot)
            cur_total = (equipped["power"] + equipped["enchant_level"]) if equipped else 0
            new_total = item["power"] + item["enchant_level"]
            if new_total > cur_total:
                return Decision("equip", {"index": idx + 1, "item": item})

        for slot_name in ("weapon", "armor", "ring"):
            item = c.get(slot_name)
            if item is None:
                continue
            enchant_cost = (item["enchant_level"] + 1) * 50 + item["power"] * 8
            if gold >= enchant_cost * 2 and item["enchant_level"] < 5:
                return Decision("enchant", {"slot": slot_name, "item": item, "cost": enchant_cost})

        tier = best_unlocked_tier(level, prestige)
        if tier and state["hp"] >= state["max_hp"] * 0.9:
            fee = estimate_arena_fee(level, prestige, gold, tier["index"])
            if gold >= fee * 2:
                return Decision("arena", {"tier_index": tier["index"], "tier_name": tier["name"], "fee": fee})

        return Decision("tick", {"cmd_kind": rng.choices(["craft", "benign"], weights=[7, 3])[0], "danger": rng.choices([1, 2, 3, 4], weights=[1, 4, 4, 1])[0]})


class BalancedStrategy(Strategy):
    name = "balanced"

    def decide(self, state, save, rng):
        c = save["character"]
        gold = state["gold"]
        level = state["level"]
        prestige = state.get("prestige", 0)

        for idx, item in enumerate(c.get("inventory", [])):
            slot = item["slot"].lower()
            equipped = c.get(slot)
            cur_total = (equipped["power"] + equipped["enchant_level"]) if equipped else 0
            new_total = item["power"] + item["enchant_level"]
            if new_total > cur_total + 1:
                return Decision("equip", {"index": idx + 1, "item": item})

        for slot_name in ("weapon", "armor", "ring"):
            item = c.get(slot_name)
            if item is None:
                continue
            enchant_cost = (item["enchant_level"] + 1) * 50 + item["power"] * 8
            if gold >= enchant_cost * 4 and item["enchant_level"] < 3:
                return Decision("enchant", {"slot": slot_name, "item": item, "cost": enchant_cost})

        tier = best_unlocked_tier(level, prestige)
        if tier and state["hp"] >= state["max_hp"] * 0.95:
            fee = estimate_arena_fee(level, prestige, gold, tier["index"])
            if gold >= fee * 3 and tier["index"] >= 1:
                return Decision("arena", {"tier_index": tier["index"], "tier_name": tier["name"], "fee": fee})

        return Decision("tick", {"cmd_kind": rng.choices(["craft", "benign", "fail"], weights=[6, 3, 1])[0], "danger": rng.choices([1, 2, 3], weights=[2, 4, 2])[0]})


class ConservativeStrategy(Strategy):
    name = "conservative"

    def decide(self, state, save, rng):
        c = save["character"]
        gold = state["gold"]
        level = state["level"]
        prestige = state.get("prestige", 0)

        for idx, item in enumerate(c.get("inventory", [])):
            slot = item["slot"].lower()
            equipped = c.get(slot)
            cur_total = (equipped["power"] + equipped["enchant_level"]) if equipped else 0
            new_total = item["power"] + item["enchant_level"]
            if new_total > cur_total + 2:
                return Decision("equip", {"index": idx + 1, "item": item})

        tier = best_unlocked_tier(level, prestige)
        if tier and state["hp"] >= state["max_hp"] * 0.98:
            fee = estimate_arena_fee(level, prestige, gold, tier["index"])
            if gold >= fee * 5:
                return Decision("arena", {"tier_index": tier["index"], "tier_name": tier["name"], "fee": fee})

        return Decision("tick", {"cmd_kind": rng.choices(["craft", "benign"], weights=[5, 5])[0], "danger": rng.choices([1, 2], weights=[3, 7])[0]})


STRATEGIES: dict[str, type[Strategy]] = {
    GreedyStrategy.name: GreedyStrategy,
    BalancedStrategy.name: BalancedStrategy,
    ConservativeStrategy.name: ConservativeStrategy,
}
