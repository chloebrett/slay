#!/usr/bin/env python3
"""
Extract Ironclad cards from spire-codex into plans/ironclad_cards.json.
Run from anywhere:  python3 plans/extract_cards.py
"""
import json
import pathlib

SRC = pathlib.Path.home() / "c/spire-codex/data/eng/cards.json"
DST = pathlib.Path(__file__).parent / "ironclad_cards.json"

KEEP_RARITIES = {"Basic", "Common", "Uncommon", "Rare"}


def clean(c):
    return {
        "id":                  c["id"],
        "name":                c["name"],
        "rarity":              c["rarity"],
        "cost":                c["cost"],
        "type":                c["type"],
        "description":         c["description"],
        "upgrade_description": c.get("upgrade_description"),
        "damage":              c.get("damage"),
        "block":               c.get("block"),
        "cards_draw":          c.get("cards_draw"),
        "powers_applied":      c.get("powers_applied") or [],
        "upgrade":             c.get("upgrade") or {},
    }


def main():
    cards = json.load(open(SRC))
    ironclad = [
        clean(c) for c in cards
        if c["color"] == "ironclad" and c["rarity"] in KEEP_RARITIES
    ]
    ironclad.sort(key=lambda c: (c["rarity"], c["name"]))
    json.dump(ironclad, open(DST, "w"), indent=2)

    print(f"Written {len(ironclad)} cards to {DST}")
    for r in ["Basic", "Common", "Uncommon", "Rare"]:
        print(f"  {r}: {sum(1 for c in ironclad if c['rarity'] == r)}")


if __name__ == "__main__":
    main()
