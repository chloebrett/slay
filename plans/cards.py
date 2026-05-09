#!/usr/bin/env python3
"""
Generate plans/cards.json — all cards with character class tags.
Reads card IDs from the Rust source and classifies them.

Run from anywhere:  python3 plans/cards.py
"""
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).parent.parent
CARDS_DIR = ROOT / "crates/slay-core/src/cards"
DST = pathlib.Path(__file__).parent / "cards.json"

# Character class for each card in the Rust codebase.
# Neutral = available to all characters (starters, status, curse).
# Ironclad / Silent = character-exclusive.
CLASS_MAP = {
    # --- Neutral (starters, status cards, curses) ---
    "strike":           "neutral",
    "strike-plus":      "neutral",
    "defend":           "neutral",
    "defend-plus":      "neutral",
    # status
    "dazed":            "neutral",
    "slimed":           "neutral",
    "wound":            "neutral",
    "burn":             "neutral",
    # curses
    "injury":           "neutral",
    "clumsy":           "neutral",
    "decay":            "neutral",
    "regret":           "neutral",
    "doubt":            "neutral",
    "shame":            "neutral",
    "parasite":         "neutral",
    "curse-of-the-bell": "neutral",
    "ascenders-bane":   "neutral",

    # --- Ironclad ---
    "bash":             "ironclad",
    "bash-plus":        "ironclad",
    "clothesline":      "ironclad",
    "clothesline-plus": "ironclad",
    "inflame":          "ironclad",
    "inflame-plus":     "ironclad",
    "disarm":           "ironclad",
    "cleave":           "ironclad",
    "cleave-plus":      "ironclad",
    "iron-wave":        "ironclad",
    "iron-wave-plus":   "ironclad",
    "spot-weakness":    "ironclad",
    "spot-weakness-plus": "ironclad",
    "twin-strike":      "ironclad",
    "twin-strike-plus": "ironclad",
    "bludgeon":         "ironclad",
    "bludgeon-plus":    "ironclad",
    "impervious":       "ironclad",
    "impervious-plus":  "ironclad",
    "seeing-red":       "ironclad",
    "seeing-red-plus":  "ironclad",
    "pummel":           "ironclad",
    "pummel-plus":      "ironclad",
    "uppercut":         "ironclad",
    "uppercut-plus":    "ironclad",
    "true-grit":        "ironclad",
    "true-grit-plus":   "ironclad",
    "thunderclap":      "ironclad",
    "thunderclap-plus": "ironclad",
    "pommel-strike":    "ironclad",
    "pommel-strike-plus": "ironclad",
    "shrug-it-off":     "ironclad",
    "shrug-it-off-plus": "ironclad",
    "reckless-charge":  "ironclad",
    "reckless-charge-plus": "ironclad",
    "entrench":         "ironclad",
    "entrench-plus":    "ironclad",
    "bloodletting":     "ironclad",
    "bloodletting-plus": "ironclad",
    "hemokinesis":      "ironclad",
    "hemokinesis-plus": "ironclad",
    "body-slam":        "ironclad",
    "body-slam-plus":   "ironclad",
    "anger":            "ironclad",
    "anger-plus":       "ironclad",
    "carnage":          "ironclad",
    "carnage-plus":     "ironclad",
    "clash":            "ironclad",
    "clash-plus":       "ironclad",
    "wild-strike":      "ironclad",
    "wild-strike-plus": "ironclad",
    "heavy-blade":      "ironclad",
    "heavy-blade-plus": "ironclad",
    "sword-boomerang":  "ironclad",
    "sword-boomerang-plus": "ironclad",
    "barricade":        "ironclad",
    "barricade-plus":   "ironclad",
    "demon-form":       "ironclad",
    "demon-form-plus":  "ironclad",
    "feel-no-pain":     "ironclad",
    "feel-no-pain-plus": "ironclad",
    "dark-embrace":     "ironclad",
    "dark-embrace-plus": "ironclad",
    "juggernaut":       "ironclad",
    "juggernaut-plus":  "ironclad",
    "rupture":          "ironclad",
    "rupture-plus":     "ironclad",
    "berserk":          "ironclad",
    "berserk-plus":     "ironclad",
    "brutality":        "ironclad",
    "brutality-plus":   "ironclad",
    "combust":          "ironclad",
    "combust-plus":     "ironclad",
    "evolve":           "ironclad",
    "evolve-plus":      "ironclad",
    "fire-breathing":   "ironclad",
    "fire-breathing-plus": "ironclad",
    "flex":             "ironclad",
    "flex-plus":        "ironclad",
    "feed":             "ironclad",
    "feed-plus":        "ironclad",
    "fiend-fire":       "ironclad",
    "fiend-fire-plus":  "ironclad",
    "perfected-strike": "ironclad",
    "perfected-strike-plus": "ironclad",
    "power-through":    "ironclad",
    "power-through-plus": "ironclad",
    "burning-pact":     "ironclad",
    "burning-pact-plus": "ironclad",
    "warcry":           "ironclad",
    "warcry-plus":      "ironclad",
    "armaments":        "ironclad",
    "armaments-plus":   "ironclad",
    "ghostly-armor":    "ironclad",
    "ghostly-armor-plus": "ironclad",
    "searing-blow":     "ironclad",
    "second-wind":      "ironclad",
    "second-wind-plus": "ironclad",
    "sentinel":         "ironclad",
    "sentinel-plus":    "ironclad",
    "all-out-attack":   "ironclad",
    "all-out-attack-plus": "ironclad",
    "all-for-one":      "ironclad",
    "all-for-one-plus": "ironclad",
    "reaper":           "ironclad",
    "reaper-plus":      "ironclad",
    "whirlwind":        "ironclad",
    "whirlwind-plus":   "ironclad",
    "immolate":         "ironclad",
    "immolate-plus":    "ironclad",
    "intimidate":       "ironclad",
    "intimidate-plus":  "ironclad",
    "shockwave":        "ironclad",
    "shockwave-plus":   "ironclad",
    "limit-break":      "ironclad",
    "limit-break-plus": "ironclad",

    # --- Silent ---
    "deadly-poison":    "silent",
    "deadly-poison-plus": "silent",
}


def extract_id_from_file(rs_file: pathlib.Path) -> list[str]:
    """Parse pub(super) fn id(...) from a card .rs file to find its IDs."""
    text = rs_file.read_text()
    # Match string literals returned from id() function
    ids = re.findall(r'"([a-z0-9-]+)"', text)
    return ids


def extract_all_card_ids() -> list[dict]:
    """Read all card .rs files and extract their IDs and card name."""
    cards = []
    for rs_file in sorted(CARDS_DIR.glob("*.rs")):
        if rs_file.name in ("mod.rs", "tests.rs"):
            continue
        ids = extract_id_from_file(rs_file)
        name = rs_file.stem.replace("_", "-")
        for card_id in ids:
            if card_id in CLASS_MAP:
                cards.append({
                    "id": card_id,
                    "file": rs_file.stem,
                    "class": CLASS_MAP[card_id],
                    "implemented": True,
                })
    # Deduplicate by id (some files may have multiple string literals)
    seen = set()
    result = []
    for c in cards:
        if c["id"] not in seen:
            seen.add(c["id"])
            result.append(c)
    return result


def planned_silent_cards() -> list[dict]:
    """All Silent cards not yet implemented."""
    planned = [
        # Basic
        {"id": "neutralize",      "name": "Neutralize",      "class": "silent", "rarity": "basic",    "type": "Attack", "cost": 0},
        {"id": "survivor",        "name": "Survivor",        "class": "silent", "rarity": "basic",    "type": "Skill",  "cost": 1},
        # Common
        {"id": "acrobatics",      "name": "Acrobatics",      "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 1},
        {"id": "backflip",        "name": "Backflip",        "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 1},
        {"id": "backstab",        "name": "Backstab",        "class": "silent", "rarity": "common",   "type": "Attack", "cost": 0},
        {"id": "bane",            "name": "Bane",            "class": "silent", "rarity": "common",   "type": "Attack", "cost": 1},
        {"id": "blade-dance",     "name": "Blade Dance",     "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 1},
        {"id": "cloak-and-dagger","name": "Cloak and Dagger","class": "silent", "rarity": "common",   "type": "Skill",  "cost": 1},
        {"id": "dagger-spray",    "name": "Dagger Spray",    "class": "silent", "rarity": "common",   "type": "Attack", "cost": 1},
        {"id": "dagger-throw",    "name": "Dagger Throw",    "class": "silent", "rarity": "common",   "type": "Attack", "cost": 1},
        {"id": "deflect",         "name": "Deflect",         "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 0},
        {"id": "dodge-and-roll",  "name": "Dodge and Roll",  "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 1},
        {"id": "endless-agony",   "name": "Endless Agony",   "class": "silent", "rarity": "common",   "type": "Attack", "cost": 0},
        {"id": "escape-plan",     "name": "Escape Plan",     "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 0},
        {"id": "flying-knee",     "name": "Flying Knee",     "class": "silent", "rarity": "common",   "type": "Attack", "cost": 1},
        {"id": "outmaneuver",     "name": "Outmaneuver",     "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 1},
        {"id": "piercing-wail",   "name": "Piercing Wail",   "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 1},
        {"id": "poisoned-stab",   "name": "Poisoned Stab",   "class": "silent", "rarity": "common",   "type": "Attack", "cost": 1},
        {"id": "prepared",        "name": "Prepared",        "class": "silent", "rarity": "common",   "type": "Skill",  "cost": 0},
        {"id": "quick-slash",     "name": "Quick Slash",     "class": "silent", "rarity": "common",   "type": "Attack", "cost": 1},
        {"id": "slice",           "name": "Slice",           "class": "silent", "rarity": "common",   "type": "Attack", "cost": 0},
        {"id": "sneaky-strike",   "name": "Sneaky Strike",   "class": "silent", "rarity": "common",   "type": "Attack", "cost": 2},
        {"id": "sucker-punch",    "name": "Sucker Punch",    "class": "silent", "rarity": "common",   "type": "Attack", "cost": 1},
        # Uncommon
        {"id": "accuracy",        "name": "Accuracy",        "class": "silent", "rarity": "uncommon", "type": "Power",  "cost": 1},
        {"id": "adrenaline",      "name": "Adrenaline",      "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 0},
        {"id": "after-image",     "name": "After Image",     "class": "silent", "rarity": "uncommon", "type": "Power",  "cost": 1},
        {"id": "blur",            "name": "Blur",            "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 1},
        {"id": "bouncing-flask",  "name": "Bouncing Flask",  "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 2},
        {"id": "calculated-gamble","name":"Calculated Gamble","class":"silent",  "rarity": "uncommon", "type": "Skill",  "cost": 0},
        {"id": "catalyst",        "name": "Catalyst",        "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 1},
        {"id": "choke",           "name": "Choke",           "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 2},
        {"id": "concentrate",     "name": "Concentrate",     "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 0},
        {"id": "corpse-explosion","name": "Corpse Explosion","class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 2},
        {"id": "dash",            "name": "Dash",            "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 2},
        {"id": "die-die-die",     "name": "Die Die Die",     "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 1},
        {"id": "distraction",     "name": "Distraction",     "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 0},
        {"id": "expertise",       "name": "Expertise",       "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 1},
        {"id": "finisher",        "name": "Finisher",        "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 1},
        {"id": "flechettes",      "name": "Flechettes",      "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 1},
        {"id": "footwork",        "name": "Footwork",        "class": "silent", "rarity": "uncommon", "type": "Power",  "cost": 1},
        {"id": "glass-knife",     "name": "Glass Knife",     "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 1},
        {"id": "heel-hook",       "name": "Heel Hook",       "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 1},
        {"id": "infinite-blades", "name": "Infinite Blades", "class": "silent", "rarity": "uncommon", "type": "Power",  "cost": 1},
        {"id": "leg-sweep",       "name": "Leg Sweep",       "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 2},
        {"id": "masterful-stab",  "name": "Masterful Stab",  "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 0},
        {"id": "noxious-fumes",   "name": "Noxious Fumes",   "class": "silent", "rarity": "uncommon", "type": "Power",  "cost": 1},
        {"id": "predator",        "name": "Predator",        "class": "silent", "rarity": "uncommon", "type": "Attack", "cost": 2},
        {"id": "riddle-with-holes","name":"Riddle With Holes","class":"silent",  "rarity": "uncommon", "type": "Attack", "cost": 2},
        {"id": "setup",           "name": "Setup",           "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 1},
        {"id": "storm-of-steel",  "name": "Storm of Steel",  "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 1},
        {"id": "tactician",       "name": "Tactician",       "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 0},
        {"id": "terror",          "name": "Terror",          "class": "silent", "rarity": "uncommon", "type": "Skill",  "cost": 1},
        {"id": "well-laid-plans", "name": "Well-Laid Plans", "class": "silent", "rarity": "uncommon", "type": "Power",  "cost": 1},
        # Rare
        {"id": "a-thousand-cuts", "name": "A Thousand Cuts", "class": "silent", "rarity": "rare",     "type": "Power",  "cost": 2},
        {"id": "alchemize",       "name": "Alchemize",       "class": "silent", "rarity": "rare",     "type": "Skill",  "cost": 1},
        {"id": "bullet-time",     "name": "Bullet Time",     "class": "silent", "rarity": "rare",     "type": "Skill",  "cost": 3},
        {"id": "burst",           "name": "Burst",           "class": "silent", "rarity": "rare",     "type": "Skill",  "cost": 1},
        {"id": "doppelganger",    "name": "Doppelganger",    "class": "silent", "rarity": "rare",     "type": "Skill",  "cost": "X"},
        {"id": "envenom",         "name": "Envenom",         "class": "silent", "rarity": "rare",     "type": "Power",  "cost": 2},
        {"id": "grand-finale",    "name": "Grand Finale",    "class": "silent", "rarity": "rare",     "type": "Attack", "cost": 0},
        {"id": "malaise",         "name": "Malaise",         "class": "silent", "rarity": "rare",     "type": "Skill",  "cost": "X"},
        {"id": "nightmare",       "name": "Nightmare",       "class": "silent", "rarity": "rare",     "type": "Skill",  "cost": 2},
        {"id": "phantasmal-killer","name":"Phantasmal Killer","class":"silent",  "rarity": "rare",     "type": "Skill",  "cost": 1},
        {"id": "tools-of-the-trade","name":"Tools of the Trade","class":"silent","rarity": "rare",    "type": "Power",  "cost": 1},
        {"id": "unload",          "name": "Unload",          "class": "silent", "rarity": "rare",     "type": "Attack", "cost": 1},
    ]
    planned_ids = {c["id"] for c in planned}
    return [dict(c, implemented=False) for c in planned]


def main():
    implemented = extract_all_card_ids()
    planned = planned_silent_cards()

    # Remove planned cards that are already implemented
    impl_ids = {c["id"] for c in implemented}
    planned = [c for c in planned if c["id"] not in impl_ids]

    all_cards = implemented + planned

    # Sort: implemented first (by class, id), then planned silent (by rarity order, name)
    rarity_order = {"basic": 0, "common": 1, "uncommon": 2, "rare": 3}
    class_order = {"neutral": 0, "ironclad": 1, "silent": 2}

    def sort_key(c):
        impl = 0 if c["implemented"] else 1
        cls = class_order.get(c["class"], 9)
        rarity = rarity_order.get(c.get("rarity", ""), 9)
        return (impl, cls, rarity, c["id"])

    all_cards.sort(key=sort_key)

    json.dump(all_cards, open(DST, "w"), indent=2)
    print(f"Written {len(all_cards)} cards to {DST}")

    impl_ic = sum(1 for c in all_cards if c["implemented"] and c["class"] == "ironclad")
    impl_s  = sum(1 for c in all_cards if c["implemented"] and c["class"] == "silent")
    impl_n  = sum(1 for c in all_cards if c["implemented"] and c["class"] == "neutral")
    plan_s  = sum(1 for c in all_cards if not c["implemented"] and c["class"] == "silent")

    print(f"  Implemented — Ironclad: {impl_ic}, Silent: {impl_s}, Neutral: {impl_n}")
    print(f"  Planned    — Silent: {plan_s}")


if __name__ == "__main__":
    main()
