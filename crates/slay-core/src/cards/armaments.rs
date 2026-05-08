use super::{CardDef, CardDescription, CardType, Grade};
use crate::combat::{ChooseCardContext, CombatPhase, CombatState, Event, gain_player_block};
use crate::rng::Rng;
use crate::status::resolve_block;
use crate::types::Energy;

pub fn apply(state: &mut CombatState, events: &mut Vec<Event>, grade: Grade, rng: &mut impl Rng) {
    let actual = resolve_block(5, &state.player.statuses);
    gain_player_block(state, events, actual, rng);
    match grade {
        Grade::Base => {
            state.phase = CombatPhase::ChooseCard(ChooseCardContext::Armaments);
        }
        Grade::Plus => {
            let upgrades: Vec<_> = state.player.hand.iter()
                .filter_map(|c| c.upgrade().map(|u| (c.clone(), u)))
                .collect();
            for (from, to) in upgrades {
                events.push(Event::CardUpgraded { from: from.clone(), to: to.clone() });
                if let Some(slot) = state.player.hand.iter().position(|c| *c == from) {
                    state.player.hand[slot] = to;
                }
            }
        }
    }
}

pub(super) fn def(grade: Grade) -> CardDef {
    let (name, description) = match grade {
        Grade::Base => ("Armaments",  CardDescription::Static("Gain 5 Block. Upgrade a card in your hand for the rest of combat.")),
        Grade::Plus => ("Armaments+", CardDescription::Static("Gain 5 Block. Upgrade ALL cards in your hand for the rest of combat.")),
    };
    CardDef { name, description, energy_cost: Energy(1), card_type: CardType::Skill }
}

pub(super) fn id(grade: Grade) -> &'static str {
    match grade { Grade::Base => "armaments", Grade::Plus => "armaments-plus" }
}
