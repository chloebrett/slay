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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NeowState {
    pub player: Player,
    pub graph: MapGraph,
    pub blessings: Vec<NeowBlessing>,
}
