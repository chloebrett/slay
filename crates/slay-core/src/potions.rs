use crate::combat::{CombatPhase, CombatState, Event, Target, apply_status, damage_all_enemies, deal_damage, draw_cards};
use crate::rng::Rng;
use crate::status::{StatusEffect, StatusMap, resolve_block, resolve_damage};
use crate::types::Hp;

pub const MAX_POTIONS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Potion {
    FirePotion,
    ExplosivePotion,
    BlockPotion,
    StrengthPotion,
    SwiftPotion,
    FearPotion,
    WeakPotion,
    BloodPotion,
    EnergyPotion,
    DexterityPotion,
    FruitJuice,
    RegenPotion,
    LiquidBronze,
    EssenceOfSteel,
    HeartOfIron,
    SteroidPotion,
    SpeedPotion,
    AncientPotion,
    DuplicationPotion,
}

pub struct PotionDef {
    pub name: &'static str,
    pub targeted: bool,
}

pub fn random_potions(rng: &mut impl Rng, count: usize) -> Vec<Potion> {
    let mut pool = vec![
        Potion::FirePotion, Potion::ExplosivePotion, Potion::BlockPotion,
        Potion::StrengthPotion, Potion::SwiftPotion, Potion::FearPotion,
        Potion::WeakPotion, Potion::BloodPotion, Potion::EnergyPotion,
        Potion::DexterityPotion, Potion::FruitJuice,
        Potion::RegenPotion, Potion::LiquidBronze,
        Potion::EssenceOfSteel, Potion::HeartOfIron,
        Potion::SteroidPotion, Potion::SpeedPotion,
        Potion::AncientPotion, Potion::DuplicationPotion,
    ];
    rng.shuffle(&mut pool);
    pool.into_iter().take(count).collect()
}

impl Potion {
    pub fn def(self) -> PotionDef {
        match self {
            Potion::FirePotion      => PotionDef { name: "Fire Potion",      targeted: true  },
            Potion::ExplosivePotion => PotionDef { name: "Explosive Potion", targeted: false },
            Potion::BlockPotion     => PotionDef { name: "Block Potion",     targeted: false },
            Potion::StrengthPotion  => PotionDef { name: "Strength Potion",  targeted: false },
            Potion::SwiftPotion     => PotionDef { name: "Swift Potion",     targeted: false },
            Potion::FearPotion      => PotionDef { name: "Fear Potion",      targeted: true  },
            Potion::WeakPotion      => PotionDef { name: "Weak Potion",      targeted: true  },
            Potion::BloodPotion      => PotionDef { name: "Blood Potion",      targeted: false },
            Potion::EnergyPotion     => PotionDef { name: "Energy Potion",     targeted: false },
            Potion::DexterityPotion  => PotionDef { name: "Dexterity Potion",  targeted: false },
            Potion::FruitJuice       => PotionDef { name: "Fruit Juice",       targeted: false },
            Potion::RegenPotion      => PotionDef { name: "Regen Potion",      targeted: false },
            Potion::LiquidBronze     => PotionDef { name: "Liquid Bronze",     targeted: false },
            Potion::EssenceOfSteel   => PotionDef { name: "Essence of Steel",  targeted: false },
            Potion::HeartOfIron      => PotionDef { name: "Heart of Iron",     targeted: false },
            Potion::SteroidPotion    => PotionDef { name: "Flex Potion",         targeted: false },
            Potion::SpeedPotion      => PotionDef { name: "Speed Potion",        targeted: false },
            Potion::AncientPotion    => PotionDef { name: "Ancient Potion",      targeted: false },
            Potion::DuplicationPotion => PotionDef { name: "Duplication Potion", targeted: false },
        }
    }

    pub fn name(self)        -> &'static str { self.def().name }
    pub fn is_targeted(self) -> bool         { self.def().targeted }

    pub fn id(self) -> &'static str {
        match self {
            Potion::FirePotion      => "fire-potion",
            Potion::ExplosivePotion => "explosive-potion",
            Potion::BlockPotion     => "block-potion",
            Potion::StrengthPotion  => "strength-potion",
            Potion::SwiftPotion     => "swift-potion",
            Potion::FearPotion      => "fear-potion",
            Potion::WeakPotion      => "weak-potion",
            Potion::BloodPotion      => "blood-potion",
            Potion::EnergyPotion     => "energy-potion",
            Potion::DexterityPotion  => "dexterity-potion",
            Potion::FruitJuice       => "fruit-juice",
            Potion::RegenPotion      => "regen-potion",
            Potion::LiquidBronze     => "liquid-bronze",
            Potion::EssenceOfSteel   => "essence-of-steel",
            Potion::HeartOfIron      => "heart-of-iron",
            Potion::SteroidPotion     => "steroid-potion",
            Potion::SpeedPotion       => "speed-potion",
            Potion::AncientPotion     => "ancient-potion",
            Potion::DuplicationPotion => "duplication-potion",
        }
    }

    pub fn all() -> Vec<Potion> {
        vec![
            Potion::FirePotion, Potion::ExplosivePotion, Potion::BlockPotion,
            Potion::StrengthPotion, Potion::SwiftPotion, Potion::FearPotion,
            Potion::WeakPotion, Potion::BloodPotion, Potion::EnergyPotion,
            Potion::DexterityPotion, Potion::FruitJuice,
            Potion::RegenPotion, Potion::LiquidBronze,
            Potion::EssenceOfSteel, Potion::HeartOfIron,
            Potion::SteroidPotion, Potion::SpeedPotion,
            Potion::AncientPotion, Potion::DuplicationPotion,
        ]
    }

    pub fn from_id(s: &str) -> Option<Potion> {
        match s {
            "fire-potion"      => Some(Potion::FirePotion),
            "explosive-potion" => Some(Potion::ExplosivePotion),
            "block-potion"     => Some(Potion::BlockPotion),
            "strength-potion"  => Some(Potion::StrengthPotion),
            "swift-potion"     => Some(Potion::SwiftPotion),
            "fear-potion"      => Some(Potion::FearPotion),
            "weak-potion"      => Some(Potion::WeakPotion),
            "blood-potion"      => Some(Potion::BloodPotion),
            "energy-potion"     => Some(Potion::EnergyPotion),
            "dexterity-potion"  => Some(Potion::DexterityPotion),
            "fruit-juice"       => Some(Potion::FruitJuice),
            "regen-potion"      => Some(Potion::RegenPotion),
            "liquid-bronze"     => Some(Potion::LiquidBronze),
            "essence-of-steel"  => Some(Potion::EssenceOfSteel),
            "heart-of-iron"     => Some(Potion::HeartOfIron),
            "steroid-potion"      => Some(Potion::SteroidPotion),
            "speed-potion"        => Some(Potion::SpeedPotion),
            "ancient-potion"      => Some(Potion::AncientPotion),
            "duplication-potion"  => Some(Potion::DuplicationPotion),
            _                     => None,
        }
    }
}

pub(crate) fn apply(potion: Potion, target_idx: usize, state: &mut CombatState, events: &mut Vec<Event>, rng: &mut impl Rng) {
    match potion {
        Potion::FirePotion => {
            let target = target_idx.min(state.enemies.len().saturating_sub(1));
            let dmg = resolve_damage(20, &StatusMap::new(), &state.enemies[target].statuses);
            let e = &mut state.enemies[target];
            let dealt = deal_damage(dmg, &mut e.hp, &mut e.block);
            events.push(Event::PlayerAttacked { raw: dmg, damage: dealt });
            if state.enemies[target].hp <= Hp(0) {
                events.push(Event::EnemyDied);
            }
        }
        Potion::ExplosivePotion => {
            damage_all_enemies(&mut state.enemies, events, 10);
        }
        Potion::BlockPotion => {
            let gained = resolve_block(12, &state.player.statuses);
            state.player.block.0 += gained;
            events.push(Event::PlayerBlocked { amount: gained });
        }
        Potion::StrengthPotion => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, 2, events);
        }
        Potion::SwiftPotion => {
            draw_cards(&mut state.player, 3, rng);
            events.push(Event::CardsDrawn { count: 3 });
        }
        Potion::FearPotion => {
            let target = target_idx.min(state.enemies.len().saturating_sub(1));
            apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Vulnerable, 3, events);
        }
        Potion::WeakPotion => {
            let target = target_idx.min(state.enemies.len().saturating_sub(1));
            apply_status(&mut state.enemies[target].statuses, Target::Enemy, StatusEffect::Weak, 3, events);
        }
        Potion::BloodPotion => {
            let heal = (state.player.max_hp.0 * 20 / 100).max(1);
            state.player.hp.0 = (state.player.hp.0 + heal).min(state.player.max_hp.0);
            events.push(Event::Healed { amount: heal });
        }
        Potion::EnergyPotion => {
            state.player.energy.0 += 2;
            events.push(Event::EnergyGained { amount: 2 });
        }
        Potion::DexterityPotion => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Dexterity, 2, events);
        }
        Potion::FruitJuice => {
            state.player.max_hp.0 += 5;
            state.player.hp.0 = (state.player.hp.0 + 5).min(state.player.max_hp.0);
            events.push(Event::Healed { amount: 5 });
        }
        Potion::RegenPotion => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Regen, 5, events);
        }
        Potion::LiquidBronze => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Thorns, 3, events);
        }
        Potion::EssenceOfSteel => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Metallicize, 2, events);
        }
        Potion::HeartOfIron => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Metallicize, 3, events);
        }
        Potion::SteroidPotion => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, 5, events);
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::StrengthDown, 5, events);
        }
        Potion::SpeedPotion => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Dexterity, 5, events);
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::DexterityDown, 5, events);
        }
        Potion::AncientPotion => {
            apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Artifact, 1, events);
        }
        Potion::DuplicationPotion => {
            state.duplication_pending = true;
        }
    }
    if state.enemies.iter().all(|e| e.hp <= Hp(0)) {
        state.phase = CombatPhase::Victory;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_potions_is_3() {
        assert_eq!(MAX_POTIONS, 3);
    }

    #[test]
    fn fire_potion_name() {
        assert_eq!(Potion::FirePotion.name(), "Fire Potion");
    }

    #[test]
    fn fire_potion_is_targeted() {
        assert!(Potion::FirePotion.is_targeted());
    }

    #[test]
    fn fear_potion_is_targeted() {
        assert!(Potion::FearPotion.is_targeted());
    }

    #[test]
    fn weak_potion_is_targeted() {
        assert!(Potion::WeakPotion.is_targeted());
    }

    #[test]
    fn block_potion_is_not_targeted() {
        assert!(!Potion::BlockPotion.is_targeted());
    }

    #[test]
    fn explosive_potion_is_not_targeted() {
        assert!(!Potion::ExplosivePotion.is_targeted());
    }

    #[test]
    fn all_potion_ids_round_trip() {
        let potions = [
            Potion::FirePotion, Potion::ExplosivePotion, Potion::BlockPotion,
            Potion::StrengthPotion, Potion::SwiftPotion, Potion::FearPotion,
            Potion::WeakPotion, Potion::BloodPotion, Potion::EnergyPotion,
            Potion::DexterityPotion, Potion::FruitJuice,
            Potion::RegenPotion, Potion::LiquidBronze,
            Potion::EssenceOfSteel, Potion::HeartOfIron,
            Potion::SteroidPotion, Potion::SpeedPotion,
            Potion::AncientPotion, Potion::DuplicationPotion,
        ];
        for p in potions {
            assert_eq!(Potion::from_id(p.id()), Some(p), "round-trip failed for {:?}", p);
        }
    }

    #[test]
    fn unknown_potion_id_returns_none() {
        assert_eq!(Potion::from_id("dragon-juice"), None);
    }
}
