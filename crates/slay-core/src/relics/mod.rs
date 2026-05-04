mod anchor;
mod bag_of_marbles;
mod bag_of_preparation;
mod black_blood;
mod blood_vial;
mod burning_blood;
mod candelabra;
mod captains_wheel;
mod chandelier;
mod cloak_clasp;
mod festive_popper;
mod gremlin_horn;
mod happy_flower;
mod horn_cleat;
mod kunai;
mod kusarigama;
mod lantern;
mod letter_opener;
mod mango;
mod mercury_hourglass;
mod nunchaku;
mod old_coin;
mod orichalcum;
mod ornamental_fan;
mod pantograph;
mod pear;
mod pendulum;
mod pocketwatch;
mod red_mask;
mod regal_pillow;
mod shuriken;
mod stone_calendar;
mod strawberry;
mod tuning_fork;
mod vajra;
mod war_paint;
mod whetstone;

use crate::cards::CardType;
use crate::combat::{deal_damage, CombatPhase, CombatState, Event, Player};
use crate::rng::Rng;
use crate::types::Hp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relic {
    // Tier 1 — pickup effects only
    Strawberry,
    Pear,
    Mango,
    OldCoin,
    Whetstone,
    WarPaint,
    // Tier 2 — end-of-combat heal
    BurningBlood,
    BlackBlood,
    // Tier 2 — combat-start effects
    Anchor,
    Vajra,
    Lantern,
    BloodVial,
    BagOfMarbles,
    RedMask,
    FestivePopper,
    Pantograph,
    BagOfPreparation,
    // Tier 3 — turn-start effects
    MercuryHourglass,
    CaptainsWheel,
    Chandelier,
    Candelabra,
    HornCleat,
    HappyFlower,
    Pendulum,
    StoneCalendar,
    // Tier 3 — turn-end effects
    Orichalcum,
    CloakClasp,
    // Tier 3 — rest effect
    RegalPillow,
    // Tier 4 — card-play counters
    Nunchaku,
    OrnamentalFan,
    Kunai,
    Shuriken,
    Kusarigama,
    LetterOpener,
    TuningFork,
    GremlinHorn,
    Pocketwatch,
}

impl Relic {
    pub fn id(&self) -> &'static str {
        match self {
            Relic::Strawberry       => strawberry::id(),
            Relic::Pear             => pear::id(),
            Relic::Mango            => mango::id(),
            Relic::OldCoin          => old_coin::id(),
            Relic::Whetstone        => whetstone::id(),
            Relic::WarPaint         => war_paint::id(),
            Relic::BurningBlood     => burning_blood::id(),
            Relic::BlackBlood       => black_blood::id(),
            Relic::Anchor           => anchor::id(),
            Relic::Vajra            => vajra::id(),
            Relic::Lantern          => lantern::id(),
            Relic::BloodVial        => blood_vial::id(),
            Relic::BagOfMarbles     => bag_of_marbles::id(),
            Relic::RedMask          => red_mask::id(),
            Relic::FestivePopper    => festive_popper::id(),
            Relic::Pantograph       => pantograph::id(),
            Relic::BagOfPreparation => bag_of_preparation::id(),
            Relic::MercuryHourglass => mercury_hourglass::id(),
            Relic::CaptainsWheel    => captains_wheel::id(),
            Relic::Chandelier       => chandelier::id(),
            Relic::Candelabra       => candelabra::id(),
            Relic::HornCleat        => horn_cleat::id(),
            Relic::HappyFlower      => happy_flower::id(),
            Relic::Pendulum         => pendulum::id(),
            Relic::StoneCalendar    => stone_calendar::id(),
            Relic::Orichalcum       => orichalcum::id(),
            Relic::CloakClasp       => cloak_clasp::id(),
            Relic::RegalPillow      => regal_pillow::id(),
            Relic::Nunchaku         => nunchaku::id(),
            Relic::OrnamentalFan    => ornamental_fan::id(),
            Relic::Kunai            => kunai::id(),
            Relic::Shuriken         => shuriken::id(),
            Relic::Kusarigama       => kusarigama::id(),
            Relic::LetterOpener     => letter_opener::id(),
            Relic::TuningFork       => tuning_fork::id(),
            Relic::GremlinHorn      => gremlin_horn::id(),
            Relic::Pocketwatch      => pocketwatch::id(),
        }
    }

    pub fn all() -> Vec<Relic> {
        vec![
            Relic::Strawberry, Relic::Pear, Relic::Mango, Relic::OldCoin,
            Relic::Whetstone, Relic::WarPaint,
            Relic::BurningBlood, Relic::BlackBlood,
            Relic::Anchor, Relic::Vajra, Relic::Lantern, Relic::BloodVial,
            Relic::BagOfMarbles, Relic::RedMask, Relic::FestivePopper,
            Relic::Pantograph, Relic::BagOfPreparation,
            Relic::MercuryHourglass, Relic::CaptainsWheel, Relic::Chandelier,
            Relic::Candelabra, Relic::HornCleat, Relic::HappyFlower,
            Relic::Pendulum, Relic::StoneCalendar,
            Relic::Orichalcum, Relic::CloakClasp, Relic::RegalPillow,
            Relic::Nunchaku, Relic::OrnamentalFan, Relic::Kunai,
            Relic::Shuriken, Relic::Kusarigama, Relic::LetterOpener,
            Relic::TuningFork, Relic::GremlinHorn, Relic::Pocketwatch,
        ]
    }

    pub fn from_id(s: &str) -> Option<Relic> {
        Self::all().into_iter().find(|r| r.id() == s)
    }
}

pub fn grant_relic(player: &mut Player, relic: Relic, rng: &mut impl Rng) -> Vec<Event> {
    let mut events = Vec::new();
    match &relic {
        Relic::Strawberry => strawberry::on_grant(player, &mut events, rng),
        Relic::Pear       => pear::on_grant(player, &mut events, rng),
        Relic::Mango      => mango::on_grant(player, &mut events, rng),
        Relic::OldCoin    => old_coin::on_grant(player, &mut events, rng),
        Relic::Whetstone  => whetstone::on_grant(player, &mut events, rng),
        Relic::WarPaint   => war_paint::on_grant(player, &mut events, rng),
        _ => {}
    }
    player.relics.push(relic);
    events
}

pub fn apply_end_of_combat_relics(player: &mut Player, events: &mut Vec<Event>) {
    let relics = player.relics.clone();
    for relic in &relics {
        match relic {
            Relic::BurningBlood => burning_blood::on_combat_end(player, events),
            Relic::BlackBlood   => black_blood::on_combat_end(player, events),
            _ => {}
        }
    }
}

pub fn apply_combat_start_relics(
    state: &mut CombatState,
    events: &mut Vec<Event>,
    rng: &mut impl Rng,
    is_boss: bool,
) {
    let relics = state.player.relics.clone();
    for relic in &relics {
        match relic {
            Relic::Anchor           => anchor::on_combat_start(state, events, rng, is_boss),
            Relic::Vajra            => vajra::on_combat_start(state, events, rng, is_boss),
            Relic::Lantern          => lantern::on_combat_start(state, events, rng, is_boss),
            Relic::BloodVial        => blood_vial::on_combat_start(state, events, rng, is_boss),
            Relic::BagOfMarbles     => bag_of_marbles::on_combat_start(state, events, rng, is_boss),
            Relic::RedMask          => red_mask::on_combat_start(state, events, rng, is_boss),
            Relic::FestivePopper    => festive_popper::on_combat_start(state, events, rng, is_boss),
            Relic::Pantograph       => pantograph::on_combat_start(state, events, rng, is_boss),
            Relic::BagOfPreparation => bag_of_preparation::on_combat_start(state, events, rng, is_boss),
            _ => {}
        }
    }
}

pub fn apply_turn_start_relics(
    state: &mut CombatState,
    events: &mut Vec<Event>,
    rng: &mut impl Rng,
) {
    let relics = state.player.relics.clone();
    for relic in &relics {
        match relic {
            Relic::MercuryHourglass => mercury_hourglass::on_turn_start(state, events, rng),
            Relic::CaptainsWheel    => captains_wheel::on_turn_start(state, events, rng),
            Relic::Chandelier       => chandelier::on_turn_start(state, events, rng),
            Relic::Candelabra       => candelabra::on_turn_start(state, events, rng),
            Relic::HornCleat        => horn_cleat::on_turn_start(state, events, rng),
            Relic::HappyFlower      => happy_flower::on_turn_start(state, events, rng),
            Relic::Pendulum         => pendulum::on_turn_start(state, events, rng),
            Relic::StoneCalendar    => stone_calendar::on_turn_start(state, events, rng),
            _ => {}
        }
    }
    if state.enemies.iter().all(|e| e.hp <= Hp(0)) {
        state.phase = CombatPhase::Victory;
    }
}

pub fn apply_turn_end_relics(
    state: &mut CombatState,
    events: &mut Vec<Event>,
    hand_size_before_discard: usize,
) {
    let relics = state.player.relics.clone();
    for relic in &relics {
        match relic {
            Relic::Orichalcum  => orichalcum::on_turn_end(state, events, hand_size_before_discard),
            Relic::CloakClasp  => cloak_clasp::on_turn_end(state, events, hand_size_before_discard),
            Relic::Pocketwatch => pocketwatch::on_turn_end(state, events, hand_size_before_discard),
            _ => {}
        }
    }
}

pub fn apply_rest_relics(player: &mut Player, events: &mut Vec<Event>) {
    let relics = player.relics.clone();
    for relic in &relics {
        if relic == &Relic::RegalPillow {
            regal_pillow::on_rest(player, events);
        }
    }
}

pub fn apply_card_play_relics(
    state: &mut CombatState,
    events: &mut Vec<Event>,
    card_type: CardType,
    rng: &mut impl Rng,
) {
    let relics = state.player.relics.clone();
    for relic in &relics {
        match relic {
            Relic::Nunchaku      => nunchaku::on_card_play(state, events, card_type, rng),
            Relic::OrnamentalFan => ornamental_fan::on_card_play(state, events, card_type, rng),
            Relic::Shuriken      => shuriken::on_card_play(state, events, card_type, rng),
            Relic::Kunai         => kunai::on_card_play(state, events, card_type, rng),
            Relic::Kusarigama    => kusarigama::on_card_play(state, events, card_type, rng),
            Relic::LetterOpener  => letter_opener::on_card_play(state, events, card_type, rng),
            Relic::TuningFork    => tuning_fork::on_card_play(state, events, card_type, rng),
            _ => {}
        }
    }
}

pub fn apply_enemy_died_relics(
    state: &mut CombatState,
    events: &mut Vec<Event>,
    rng: &mut impl Rng,
) {
    let relics = state.player.relics.clone();
    for relic in &relics {
        if relic == &Relic::GremlinHorn {
            gremlin_horn::on_enemy_died(state, events, rng);
        }
    }
}

fn heal_player(player: &mut Player, amount: i32, events: &mut Vec<Event>) {
    let actual = amount.min(player.max_hp.0 - player.hp.0);
    if actual > 0 {
        player.hp.0 += actual;
        events.push(Event::Healed { amount: actual });
    }
}

fn raise_max_hp(player: &mut Player, amount: i32) {
    player.max_hp.0 += amount;
    player.hp.0 += amount;
}

fn damage_all_enemies(state: &mut CombatState, events: &mut Vec<Event>, amount: i32) {
    for i in 0..state.enemies.len() {
        if state.enemies[i].hp > Hp(0) {
            let e = &mut state.enemies[i];
            let dmg = deal_damage(amount, &mut e.hp, &mut e.block);
            events.push(Event::EnemyAttacked { raw: amount, damage: dmg });
            if state.enemies[i].hp <= Hp(0) {
                events.push(Event::EnemyDied);
            }
        }
    }
}

fn damage_random_living_enemy(
    state: &mut CombatState,
    events: &mut Vec<Event>,
    amount: i32,
    rng: &mut impl Rng,
) {
    let mut living: Vec<usize> = (0..state.enemies.len())
        .filter(|&i| state.enemies[i].hp > Hp(0))
        .collect();
    if living.is_empty() { return; }
    rng.shuffle(&mut living);
    let i = living[0];
    let e = &mut state.enemies[i];
    let dmg = deal_damage(amount, &mut e.hp, &mut e.block);
    events.push(Event::EnemyAttacked { raw: amount, damage: dmg });
    if state.enemies[i].hp <= Hp(0) {
        events.push(Event::EnemyDied);
    }
}

fn upgrade_random_of_type(
    player: &mut Player,
    card_type: CardType,
    count: usize,
    rng: &mut impl Rng,
    events: &mut Vec<Event>,
) {
    let mut indices: Vec<usize> = player
        .deck
        .iter()
        .enumerate()
        .filter(|(_, c)| c.card_type() == card_type && c.upgrade().is_some())
        .map(|(i, _)| i)
        .collect();
    rng.shuffle(&mut indices);
    for &idx in indices.iter().take(count) {
        let from = player.deck[idx].clone();
        let to = from.upgrade().unwrap(); // SAFETY: filtered to upgradeable above
        player.deck[idx] = to.clone();
        events.push(Event::CardUpgraded { from, to });
    }
}

#[cfg(test)]
mod tests;
