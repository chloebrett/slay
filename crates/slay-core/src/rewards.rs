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
    // player.deck is the permanent collection and is never modified during combat —
    // draw_pile is initialised from player.deck.clone(). exhaust_pile is a working
    // subset, not new cards, so we must NOT extend deck with it here.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::Grade;
    use crate::types::Hp;

    fn base_player(deck: Vec<Card>) -> Player {
        Player {
            hp: Hp(80), max_hp: Hp(80),
            block: Block(0),
            energy: Energy(3), max_energy: Energy(3),
            hand: Vec::new(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            statuses: StatusMap::new(),
            deck,
            gold: 0,
            relics: Vec::new(),
            potions: Vec::new(),
            neow_lament_combats_remaining: 0,
            reached_boss: false,
            potion_chance: 0.40,
        }
    }

    #[test]
    fn deck_size_unchanged_after_combat_with_exhausted_cards() {
        let deck = vec![
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Defend(Grade::Base),
        ];
        let mut player = base_player(deck.clone());
        // Simulate a card being exhausted during combat.
        player.exhaust_pile.push(Card::Strike(Grade::Base));

        let after = player_after_combat(player, 10);

        assert_eq!(
            after.deck.len(), deck.len(),
            "deck should not grow when cards were exhausted during combat"
        );
    }

    #[test]
    fn deck_size_unchanged_after_combat_three_times() {
        // Regression: exhaust_pile cards were appended to deck each combat,
        // causing a card to appear multiple times after several runs.
        let deck = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)];
        let mut player = base_player(deck.clone());

        for _ in 0..3 {
            player.exhaust_pile.push(Card::Strike(Grade::Base));
            player = player_after_combat(player, 0);
        }

        assert_eq!(
            player.deck.len(), deck.len(),
            "deck must not accumulate exhausted cards across multiple combats"
        );
    }

    #[test]
    fn combat_piles_cleared_after_combat() {
        let mut player = base_player(vec![Card::Strike(Grade::Base)]);
        player.hand = vec![Card::Defend(Grade::Base)];
        player.draw_pile = vec![Card::Strike(Grade::Base)];
        player.discard_pile = vec![Card::Strike(Grade::Base)];
        player.exhaust_pile = vec![Card::Strike(Grade::Base)];

        let after = player_after_combat(player, 0);

        assert!(after.hand.is_empty());
        assert!(after.draw_pile.is_empty());
        assert!(after.discard_pile.is_empty());
        assert!(after.exhaust_pile.is_empty());
    }
}
