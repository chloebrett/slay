#!/usr/bin/env python3
"""
Extract Ironclad-eligible relics from spire-codex into plans/relics.json.
Run from anywhere:  python3 plans/extract_relics.py
"""
import json
import pathlib

SRC = pathlib.Path.home() / "c/spire-codex/data/eng/relics.json"
DST = pathlib.Path(__file__).parent / "relics.json"

KEEP_RARITIES = {"Starter", "Common", "Uncommon", "Rare"}
KEEP_POOLS = {"shared", "ironclad"}


def clean(r):
    return {
        "id":          r["id"],
        "name":        r["name"],
        "rarity":      r["rarity_key"],
        "pool":        r["pool"],
        "description": r["description"],
        "flavor":      r.get("flavor"),
    }


def main():
    relics = json.load(open(SRC))
    filtered = [
        clean(r) for r in relics
        if r["rarity_key"] in KEEP_RARITIES and r["pool"] in KEEP_POOLS
    ]
    filtered.sort(key=lambda r: (r["rarity"], r["name"]))
    json.dump(filtered, open(DST, "w"), indent=2)

    print(f"Written {len(filtered)} relics to {DST}")
    for rarity in ["Starter", "Common", "Uncommon", "Rare"]:
        count = sum(1 for r in filtered if r["rarity"] == rarity)
        pools = sorted(set(r["pool"] for r in filtered if r["rarity"] == rarity))
        print(f"  {rarity}: {count}  (pools: {', '.join(pools)})")


if __name__ == "__main__":
    main()
