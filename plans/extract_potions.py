#!/usr/bin/env python3
"""
Extract Ironclad-eligible STS1 potions from the Steam installation into plans/potions.json.
Reads directly from the game JAR's localization files and class bytecode.
Run from anywhere:  python3 plans/extract_potions.py
"""
import json
import pathlib
import re
import zipfile

JAR = (
    pathlib.Path.home()
    / "Library/Application Support/Steam/steamapps/common/SlayTheSpire"
    / "SlayTheSpire.app/Contents/Resources/desktop-1.0.jar"
)
DST = pathlib.Path(__file__).parent / "potions.json"

# Pool assignments extracted from PotionHelper.class bytecode (class ref ordering).
# Only ironclad and shared are eligible for an Ironclad run.
IRONCLAD_POOL = {"BloodPotion", "HeartOfIron"}
SILENT_POOL   = {"PoisonPotion", "CunningPotion", "GhostInAJar"}
DEFECT_POOL   = {"FocusPotion", "PotionOfCapacity", "EssenceOfDarkness"}
WATCHER_POOL  = {"BottledMiracle", "StancePotion", "Ambrosia"}
SKIP          = {"ElixirPotion", "Elixir", "CultistPotion"}  # deprecated or enemy-only

# Class names whose localization key doesn't match the class name pattern
LOC_KEY_OVERRIDES = {
    "WeakenPotion":        "Weak Potion",
    "DistilledChaosPotion": "DistilledChaos",
}

RARITIES = {"COMMON", "UNCOMMON", "RARE"}

# Canonical class names for all non-deprecated potions in the JAR
ALL_CLASSES = [
    "AncientPotion", "AttackPotion", "BlessingOfTheForge", "BlockPotion", "BloodPotion",
    "BottledMiracle", "ColorlessPotion", "CultistPotion", "CunningPotion", "DexterityPotion",
    "DistilledChaosPotion", "DuplicationPotion", "EnergyPotion", "EntropicBrew",
    "EssenceOfDarkness", "EssenceOfSteel", "ExplosivePotion", "FairyPotion", "FearPotion",
    "FirePotion", "FocusPotion", "FruitJuice", "GamblersBrew", "GhostInAJar", "HeartOfIron",
    "LiquidBronze", "LiquidMemories", "PoisonPotion", "PotionOfCapacity", "PowerPotion",
    "RegenPotion", "SkillPotion", "SmokeBomb", "SneckoOil", "SpeedPotion", "StancePotion",
    "SteroidPotion", "StrengthPotion", "SwiftPotion", "WeakenPotion",
]


def pool(cls: str) -> str:
    if cls in IRONCLAD_POOL:
        return "ironclad"
    if cls in SILENT_POOL | DEFECT_POOL | WATCHER_POOL:
        return "other"
    return "shared"


def rarity_from_class(data: bytes) -> str:
    strings = [s.decode("ascii", errors="replace") for s in re.findall(rb"[\x20-\x7e]{3,}", data)]
    return next((s for s in strings if s in RARITIES), "UNKNOWN")


def main():
    with zipfile.ZipFile(JAR) as z:
        loc = json.loads(z.read("localization/eng/potions.json"))
        # Build id -> name + description from localization
        loc_by_id = {k: v for k, v in loc.items()}

        results = []
        for cls in ALL_CLASSES:
            if cls in SKIP:
                continue
            p = pool(cls)
            if p == "other":
                continue

            class_data = z.read(f"com/megacrit/cardcrawl/potions/{cls}.class")
            rar = rarity_from_class(class_data).capitalize()

            # Find name/description from localization (keys are inconsistent in the file)
            loc_key = LOC_KEY_OVERRIDES.get(cls, cls)
            entry = loc_by_id.get(loc_key) or next(
                (v for k, v in loc_by_id.items() if k.replace(" ", "") == cls),
                None,
            )
            name = entry["NAME"] if entry else cls
            desc = " ".join(entry["DESCRIPTIONS"]) if entry else ""

            results.append({
                "id":          cls,
                "name":        name,
                "rarity":      rar,
                "pool":        p,
                "description": desc,
            })

    results.sort(key=lambda p: (p["rarity"], p["name"]))
    json.dump(results, open(DST, "w"), indent=2)

    print(f"Written {len(results)} potions to {DST}")
    for rarity in ["Common", "Uncommon", "Rare"]:
        count = sum(1 for p in results if p["rarity"] == rarity)
        pools = sorted(set(p["pool"] for p in results if p["rarity"] == rarity))
        print(f"  {rarity}: {count}  (pools: {', '.join(pools)})")


if __name__ == "__main__":
    main()
