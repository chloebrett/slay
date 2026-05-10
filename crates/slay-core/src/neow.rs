use crate::cards::Card;
use crate::potions::Potion;
use crate::relics::Relic;
use crate::combat::Player;
use crate::run::MapGraph;

#[derive(Debug, Clone, Default)]
pub struct NeowContext {
    pub runs_completed: u32,
    pub prev_run_reached_boss: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NeowBlessing {
    GainMaxHp(i32),
    NeowsLament,
    GainGold(i32),
    GainRelic(Relic),
    GainPotions(Vec<Potion>),
    RemoveCard,
    TransformCard,
    UpgradeCard,
    ChooseRareCard(Vec<Card>),
    LoseHpGainGold { hp_loss: i32, gold: i32 },
    LoseHpRemoveCards { hp_loss: i32, count: usize },
    LoseHpTransformCards { hp_loss: i32, count: usize },
    LoseHpGainRareRelic { hp_loss: i32, relic: Relic },
    ObtainCurseGainRareRelic { curse: Card, relic: Relic },
    SwapStarterRelic(Relic),
}

impl NeowBlessing {
    pub fn describe(&self) -> String {
        match self {
            NeowBlessing::GainMaxHp(n) =>
                format!("Max HP +{n}"),
            NeowBlessing::NeowsLament =>
                "Enemies in the next 3 combats will have 1 HP.".to_string(),
            NeowBlessing::GainGold(n) =>
                format!("Receive {n} gold."),
            NeowBlessing::GainRelic(_) =>
                "Obtain a random common relic.".to_string(),
            NeowBlessing::GainPotions(potions) =>
                format!("Obtain {} random potions.", potions.len()),
            NeowBlessing::RemoveCard =>
                "Remove a card.".to_string(),
            NeowBlessing::TransformCard =>
                "Transform a card.".to_string(),
            NeowBlessing::UpgradeCard =>
                "Upgrade a card.".to_string(),
            NeowBlessing::ChooseRareCard(_) =>
                "Choose a rare card to obtain.".to_string(),
            NeowBlessing::LoseHpGainGold { hp_loss, gold } =>
                format!("Lose {hp_loss} HP. Receive {gold} gold."),
            NeowBlessing::LoseHpRemoveCards { hp_loss, count } =>
                format!("Lose {hp_loss} HP. Remove {count} cards."),
            NeowBlessing::LoseHpTransformCards { hp_loss, count } =>
                format!("Lose {hp_loss} HP. Transform {count} cards."),
            NeowBlessing::LoseHpGainRareRelic { hp_loss, .. } =>
                format!("Lose {hp_loss} HP. Obtain a random rare relic."),
            NeowBlessing::ObtainCurseGainRareRelic { .. } =>
                "Obtain a curse. Obtain a random rare relic.".to_string(),
            NeowBlessing::SwapStarterRelic(_) =>
                "Replace your starter relic with a random Boss Relic.".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NeowState {
    pub player: Player,
    pub graph: MapGraph,
    pub blessings: Vec<NeowBlessing>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Card;
    use crate::potions::Potion;
    use crate::relics::Relic;

    #[test]
    fn describe_simple_blessings() {
        assert_eq!(NeowBlessing::GainMaxHp(8).describe(), "Max HP +8");
        assert_eq!(NeowBlessing::GainGold(100).describe(), "Receive 100 gold.");
        assert_eq!(NeowBlessing::RemoveCard.describe(), "Remove a card.");
        assert_eq!(NeowBlessing::TransformCard.describe(), "Transform a card.");
        assert_eq!(NeowBlessing::UpgradeCard.describe(), "Upgrade a card.");
        assert_eq!(NeowBlessing::ChooseRareCard(vec![]).describe(), "Choose a rare card to obtain.");
        assert_eq!(NeowBlessing::NeowsLament.describe(), "Enemies in the next 3 combats will have 1 HP.");
    }

    #[test]
    fn describe_blessings_with_dynamic_values() {
        assert_eq!(
            NeowBlessing::LoseHpGainGold { hp_loss: 7, gold: 250 }.describe(),
            "Lose 7 HP. Receive 250 gold."
        );
        assert_eq!(
            NeowBlessing::LoseHpRemoveCards { hp_loss: 7, count: 2 }.describe(),
            "Lose 7 HP. Remove 2 cards."
        );
        assert_eq!(
            NeowBlessing::LoseHpTransformCards { hp_loss: 7, count: 2 }.describe(),
            "Lose 7 HP. Transform 2 cards."
        );
    }

    #[test]
    fn describe_relics_are_generic_not_named() {
        assert_eq!(NeowBlessing::GainRelic(Relic::BurningBlood).describe(), "Obtain a random common relic.");
        assert_eq!(
            NeowBlessing::LoseHpGainRareRelic { hp_loss: 7, relic: Relic::BurningBlood }.describe(),
            "Lose 7 HP. Obtain a random rare relic."
        );
        assert_eq!(
            NeowBlessing::SwapStarterRelic(Relic::BurningBlood).describe(),
            "Replace your starter relic with a random Boss Relic."
        );
    }

    #[test]
    fn describe_obtain_curse_gain_rare_relic() {
        assert_eq!(
            NeowBlessing::ObtainCurseGainRareRelic {
                curse: Card::CurseOfTheBell,
                relic: Relic::BurningBlood,
            }.describe(),
            "Obtain a curse. Obtain a random rare relic."
        );
    }

    #[test]
    fn describe_gain_potions_shows_count_not_names() {
        let desc = NeowBlessing::GainPotions(vec![Potion::FirePotion, Potion::BlockPotion, Potion::BlockPotion]).describe();
        assert_eq!(desc, "Obtain 3 random potions.");
    }
}
