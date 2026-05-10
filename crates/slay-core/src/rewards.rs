use crate::cards::Card;
use crate::combat::{Event, Player};
use crate::potions::{Potion, MAX_POTIONS};
use crate::relics::Relic;
use crate::rng::Rng;
use crate::status::StatusMap;
use crate::types::{Block, Energy};

pub(crate) fn generate_rewards(rng: &mut impl Rng) -> Vec<Card> {
    let mut pool = crate::cards::reward_pool();
    rng.shuffle(&mut pool);
    pool.into_iter().take(3).collect()
}

pub(crate) fn random_potion(rng: &mut impl Rng) -> Potion {
    let mut pool = [
        Potion::FirePotion, Potion::ExplosivePotion, Potion::BlockPotion,
        Potion::StrengthPotion, Potion::SwiftPotion, Potion::FearPotion,
        Potion::WeakPotion, Potion::BloodPotion, Potion::EnergyPotion,
    ];
    rng.shuffle(&mut pool);
    pool[0]
}

pub(crate) fn award_potion(player: &mut Player, events: &mut Vec<Event>, rng: &mut impl Rng) -> Option<Potion> {
    let dropped = rng.gen_bool(player.potion_chance);
    if dropped {
        player.potion_chance = (player.potion_chance - 0.10).max(0.0);
    } else {
        player.potion_chance = (player.potion_chance + 0.10).min(1.0);
        return None;
    }
    let potion = random_potion(rng);
    if player.potions.len() < MAX_POTIONS {
        player.potions.push(potion);
        events.push(Event::PotionAwarded { potion });
        None
    } else {
        Some(potion)
    }
}

pub(crate) fn player_after_combat(player: Player, gold_gain: i32) -> Player {
    let mut deck = player.deck;
    deck.extend(player.exhaust_pile);
    // Lantern grants +1 max energy for the combat only; restore original value here.
    let max_energy = if player.relics.contains(&Relic::Lantern) {
        Energy(player.max_energy.0 - 1)
    } else {
        player.max_energy
    };
    Player {
        block: Block(0),
        energy: max_energy,
        max_energy,
        hand: Vec::new(),
        draw_pile: Vec::new(),
        discard_pile: Vec::new(),
        exhaust_pile: Vec::new(),
        statuses: StatusMap::new(),
        gold: player.gold + gold_gain,
        deck,
        ..player
    }
}

pub(crate) fn roll_gold(lo: i32, hi: i32, rng: &mut impl Rng) -> i32 {
    let mut values: Vec<i32> = (lo..=hi).collect();
    rng.shuffle(&mut values);
    values[0]
}

pub(crate) fn combat_gold(is_elite: bool, is_boss: bool, rng: &mut impl Rng) -> i32 {
    if is_boss        { roll_gold(95, 105, rng) }
    else if is_elite  { roll_gold(25,  35, rng) }
    else              { roll_gold(10,  20, rng) }
}
