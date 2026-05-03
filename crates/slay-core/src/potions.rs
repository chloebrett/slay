pub const MAX_POTIONS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

pub struct PotionDef {
    pub name: &'static str,
    pub targeted: bool,
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
            Potion::BloodPotion     => PotionDef { name: "Blood Potion",     targeted: false },
            Potion::EnergyPotion    => PotionDef { name: "Energy Potion",    targeted: false },
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
            Potion::BloodPotion     => "blood-potion",
            Potion::EnergyPotion    => "energy-potion",
        }
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
            "blood-potion"     => Some(Potion::BloodPotion),
            "energy-potion"    => Some(Potion::EnergyPotion),
            _                  => None,
        }
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
