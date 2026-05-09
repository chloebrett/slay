#!/usr/bin/env python3
"""
Extract Ironclad-eligible potions from spire-codex into plans/potions.json.
Run from anywhere:  python3 plans/extract_potions.py
"""
import json
import pathlib

SRC = pathlib.Path.home() / "c/spire-codex/data/eng/potions.json"
DST = pathlib.Path(__file__).parent / "potions.json"

KEEP_RARITIES = {"Common", "Uncommon", "Rare"}
KEEP_POOLS = {"shared", "ironclad"}


def clean(p):
    return {
        "id":          p["id"],
        "name":        p["name"],
        "rarity":      p["rarity_key"],
        "pool":        p["pool"],
        "description": p["description"],
    }


def main():
    potions = json.load(open(SRC))
    filtered = [
        clean(p) for p in potions
        if p["rarity_key"] in KEEP_RARITIES and p["pool"] in KEEP_POOLS
    ]
    filtered.sort(key=lambda p: (p["rarity"], p["name"]))
    json.dump(filtered, open(DST, "w"), indent=2)

    print(f"Written {len(filtered)} potions to {DST}")
    for rarity in ["Common", "Uncommon", "Rare"]:
        count = sum(1 for p in filtered if p["rarity"] == rarity)
        pools = sorted(set(p["pool"] for p in filtered if p["rarity"] == rarity))
        print(f"  {rarity}: {count}  (pools: {', '.join(pools)})")


if __name__ == "__main__":
    main()
