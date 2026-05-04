use crate::cards::CardType;
use crate::combat::{
    apply_status, deal_damage, draw_cards,
    CombatPhase, CombatState, Event, Player, Target,
};
use crate::rng::Rng;
use crate::status::StatusEffect;
use crate::types::{Block, Hp};

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
            Relic::Strawberry       => "strawberry",
            Relic::Pear             => "pear",
            Relic::Mango            => "mango",
            Relic::OldCoin          => "old-coin",
            Relic::Whetstone        => "whetstone",
            Relic::WarPaint         => "war-paint",
            Relic::BurningBlood     => "burning-blood",
            Relic::BlackBlood       => "black-blood",
            Relic::Anchor           => "anchor",
            Relic::Vajra            => "vajra",
            Relic::Lantern          => "lantern",
            Relic::BloodVial        => "blood-vial",
            Relic::BagOfMarbles     => "bag-of-marbles",
            Relic::RedMask          => "red-mask",
            Relic::FestivePopper    => "festive-popper",
            Relic::Pantograph       => "pantograph",
            Relic::BagOfPreparation => "bag-of-preparation",
            Relic::MercuryHourglass => "mercury-hourglass",
            Relic::CaptainsWheel    => "captains-wheel",
            Relic::Chandelier       => "chandelier",
            Relic::Candelabra       => "candelabra",
            Relic::HornCleat        => "horn-cleat",
            Relic::HappyFlower      => "happy-flower",
            Relic::Pendulum         => "pendulum",
            Relic::StoneCalendar    => "stone-calendar",
            Relic::Orichalcum       => "orichalcum",
            Relic::CloakClasp       => "cloak-clasp",
            Relic::RegalPillow      => "regal-pillow",
            Relic::Nunchaku         => "nunchaku",
            Relic::OrnamentalFan    => "ornamental-fan",
            Relic::Kunai            => "kunai",
            Relic::Shuriken         => "shuriken",
            Relic::Kusarigama       => "kusarigama",
            Relic::LetterOpener     => "letter-opener",
            Relic::TuningFork       => "tuning-fork",
            Relic::GremlinHorn      => "gremlin-horn",
            Relic::Pocketwatch      => "pocketwatch",
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
        Relic::Strawberry => raise_max_hp(player, 7),
        Relic::Pear       => raise_max_hp(player, 10),
        Relic::Mango      => raise_max_hp(player, 14),
        Relic::OldCoin    => player.gold += 300,
        Relic::Whetstone  => upgrade_random_of_type(player, CardType::Attack, 2, rng, &mut events),
        Relic::WarPaint   => upgrade_random_of_type(player, CardType::Skill, 2, rng, &mut events),
        _ => {}
    }
    player.relics.push(relic);
    events
}

pub fn apply_end_of_combat_relics(player: &mut Player, events: &mut Vec<Event>) {
    if player.relics.contains(&Relic::BurningBlood) {
        heal_player(player, 6, events);
    }
    if player.relics.contains(&Relic::BlackBlood) {
        heal_player(player, 12, events);
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
            Relic::Anchor => {
                state.player.block.0 += 10;
                events.push(Event::PlayerBlocked { amount: 10 });
            }
            Relic::Vajra => {
                apply_status(
                    &mut state.player.statuses,
                    Target::Player,
                    StatusEffect::Strength,
                    1,
                    events,
                );
            }
            Relic::Lantern => {
                state.player.max_energy.0 += 1;
                state.player.energy.0 += 1;
                events.push(Event::EnergyGained { amount: 1 });
            }
            Relic::BloodVial => {
                heal_player(&mut state.player, 2, events);
            }
            Relic::BagOfMarbles => {
                for i in 0..state.enemies.len() {
                    if state.enemies[i].hp > Hp(0) {
                        apply_status(
                            &mut state.enemies[i].statuses,
                            Target::Enemy,
                            StatusEffect::Vulnerable,
                            1,
                            events,
                        );
                    }
                }
            }
            Relic::RedMask => {
                for i in 0..state.enemies.len() {
                    if state.enemies[i].hp > Hp(0) {
                        apply_status(
                            &mut state.enemies[i].statuses,
                            Target::Enemy,
                            StatusEffect::Weak,
                            1,
                            events,
                        );
                    }
                }
            }
            Relic::FestivePopper => {
                for i in 0..state.enemies.len() {
                    if state.enemies[i].hp > Hp(0) {
                        let e = &mut state.enemies[i];
                        let dmg = deal_damage(9, &mut e.hp, &mut e.block);
                        events.push(Event::EnemyAttacked { raw: 9, damage: dmg });
                        if state.enemies[i].hp <= Hp(0) {
                            events.push(Event::EnemyDied);
                        }
                    }
                }
            }
            Relic::Pantograph if is_boss => {
                heal_player(&mut state.player, 25, events);
            }
            Relic::BagOfPreparation => {
                draw_cards(&mut state.player, 2, rng);
                events.push(Event::CardsDrawn { count: 2 });
            }
            _ => {}
        }
    }
}

pub fn apply_turn_start_relics(
    state: &mut CombatState,
    events: &mut Vec<Event>,
    rng: &mut impl Rng,
) {
    let turn = state.turn;
    let relics = state.player.relics.clone();
    for relic in &relics {
        match relic {
            Relic::MercuryHourglass => {
                damage_all_enemies(state, events, 3);
            }
            Relic::CaptainsWheel if turn == 3 => {
                state.player.block.0 += 18;
                events.push(Event::PlayerBlocked { amount: 18 });
            }
            Relic::Chandelier if turn == 3 => {
                state.player.energy.0 += 3;
                events.push(Event::EnergyGained { amount: 3 });
            }
            Relic::Candelabra if turn == 2 => {
                state.player.energy.0 += 2;
                events.push(Event::EnergyGained { amount: 2 });
            }
            Relic::HornCleat if turn == 2 => {
                state.player.block.0 += 14;
                events.push(Event::PlayerBlocked { amount: 14 });
            }
            Relic::HappyFlower if turn.is_multiple_of(3) => {
                state.player.energy.0 += 1;
                events.push(Event::EnergyGained { amount: 1 });
            }
            Relic::Pendulum if turn.is_multiple_of(3) => {
                draw_cards(&mut state.player, 1, rng);
                events.push(Event::CardsDrawn { count: 1 });
            }
            Relic::StoneCalendar if turn == 7 => {
                damage_all_enemies(state, events, 52);
            }
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
            Relic::Orichalcum if state.player.block == Block(0) => {
                state.player.block.0 += 6;
                events.push(Event::PlayerBlocked { amount: 6 });
            }
            Relic::CloakClasp => {
                let gain = hand_size_before_discard as i32;
                if gain > 0 {
                    state.player.block.0 += gain;
                    events.push(Event::PlayerBlocked { amount: gain });
                }
            }
            Relic::Pocketwatch if state.cards_played_this_turn <= 3 => {
                state.extra_draws_next_turn += 3;
            }
            _ => {}
        }
    }
}

pub fn apply_rest_relics(player: &mut Player, events: &mut Vec<Event>) {
    if player.relics.contains(&Relic::RegalPillow) {
        heal_player(player, 15, events);
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
        match (relic, card_type) {
            (Relic::Nunchaku, CardType::Attack) if state.attacks_this_combat.is_multiple_of(10) => {
                state.player.energy.0 += 1;
                events.push(Event::EnergyGained { amount: 1 });
            }
            (Relic::OrnamentalFan, CardType::Attack) if state.attacks_this_turn.is_multiple_of(3) => {
                state.player.block.0 += 4;
                events.push(Event::PlayerBlocked { amount: 4 });
            }
            (Relic::Shuriken, CardType::Attack) if state.attacks_this_turn.is_multiple_of(3) => {
                apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, 1, events);
            }
            (Relic::Kunai, CardType::Attack) if state.attacks_this_turn.is_multiple_of(3) => {
                apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Dexterity, 1, events);
            }
            (Relic::Kusarigama, CardType::Attack) if state.attacks_this_turn.is_multiple_of(3) => {
                damage_random_living_enemy(state, events, 6, rng);
            }
            (Relic::LetterOpener, CardType::Skill) if state.skills_this_turn.is_multiple_of(3) => {
                damage_all_enemies(state, events, 5);
            }
            (Relic::TuningFork, CardType::Skill) if state.skills_this_combat.is_multiple_of(10) => {
                state.player.block.0 += 7;
                events.push(Event::PlayerBlocked { amount: 7 });
            }
            _ => {}
        }
    }
}

pub fn apply_enemy_died_relics(
    state: &mut CombatState,
    events: &mut Vec<Event>,
    rng: &mut impl Rng,
) {
    if state.player.relics.contains(&Relic::GremlinHorn) {
        state.player.energy.0 += 1;
        events.push(Event::EnergyGained { amount: 1 });
        draw_cards(&mut state.player, 1, rng);
        events.push(Event::CardsDrawn { count: 1 });
    }
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
mod tests {
    use super::*;
    use crate::cards::{Card, Grade};
    use crate::combat::{combat_with_hand, combat_with_two_enemies};
    use crate::rng::NoOpRng;
    use crate::status::StatusMap;
    use crate::types::{Block, Energy};

    fn rng() -> NoOpRng { NoOpRng }

    fn test_player() -> Player {
        Player {
            hp: Hp(80),
            max_hp: Hp(80),
            block: Block(0),
            energy: Energy(3),
            max_energy: Energy(3),
            hand: vec![],
            draw_pile: vec![],
            discard_pile: vec![],
            exhaust_pile: vec![],
            statuses: StatusMap::new(),
            deck: vec![],
            gold: 0,
            relics: vec![],
            potions: vec![],
        }
    }

    // --- Tier 1: Strawberry / Pear / Mango ---

    #[test]
    fn strawberry_raises_max_hp_by_7() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Strawberry, &mut rng());
        assert_eq!(player.max_hp, Hp(87));
    }

    #[test]
    fn strawberry_raises_current_hp_by_7() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Strawberry, &mut rng());
        assert_eq!(player.hp, Hp(87));
    }

    #[test]
    fn strawberry_when_damaged_still_raises_hp_by_7() {
        let mut player = test_player();
        player.hp = Hp(50);
        grant_relic(&mut player, Relic::Strawberry, &mut rng());
        assert_eq!(player.hp, Hp(57));
        assert_eq!(player.max_hp, Hp(87));
    }

    #[test]
    fn pear_raises_max_hp_by_10() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Pear, &mut rng());
        assert_eq!(player.max_hp, Hp(90));
        assert_eq!(player.hp, Hp(90));
    }

    #[test]
    fn mango_raises_max_hp_by_14() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Mango, &mut rng());
        assert_eq!(player.max_hp, Hp(94));
        assert_eq!(player.hp, Hp(94));
    }

    #[test]
    fn strawberry_is_recorded_in_player_relics() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::Strawberry, &mut rng());
        assert!(player.relics.contains(&Relic::Strawberry));
    }

    // --- Tier 1: OldCoin ---

    #[test]
    fn old_coin_grants_300_gold() {
        let mut player = test_player();
        grant_relic(&mut player, Relic::OldCoin, &mut rng());
        assert_eq!(player.gold, 300);
    }

    #[test]
    fn old_coin_stacks_with_existing_gold() {
        let mut player = test_player();
        player.gold = 50;
        grant_relic(&mut player, Relic::OldCoin, &mut rng());
        assert_eq!(player.gold, 350);
    }

    // --- Tier 1: Whetstone ---

    #[test]
    fn whetstone_upgrades_2_attack_cards_in_deck() {
        let mut player = test_player();
        player.deck = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Defend(Grade::Base)];
        grant_relic(&mut player, Relic::Whetstone, &mut rng());
        assert_eq!(
            player.deck.iter().filter(|c| **c == Card::Strike(Grade::Plus)).count(),
            2
        );
        assert!(player.deck.contains(&Card::Defend(Grade::Base))); // skill unchanged
    }

    #[test]
    fn whetstone_upgrades_fewer_than_2_when_not_enough_attacks() {
        let mut player = test_player();
        player.deck = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base), Card::Defend(Grade::Base)];
        grant_relic(&mut player, Relic::Whetstone, &mut rng());
        assert_eq!(
            player.deck.iter().filter(|c| **c == Card::Strike(Grade::Plus)).count(),
            1
        );
    }

    #[test]
    fn whetstone_emits_card_upgraded_events() {
        let mut player = test_player();
        player.deck = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let events = grant_relic(&mut player, Relic::Whetstone, &mut rng());
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::CardUpgraded { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn whetstone_skips_already_upgraded_attacks() {
        let mut player = test_player();
        player.deck = vec![Card::Strike(Grade::Plus), Card::Bash(Grade::Base)]; // StrikePlus can't upgrade
        let events = grant_relic(&mut player, Relic::Whetstone, &mut rng());
        assert!(player.deck.contains(&Card::Bash(Grade::Plus)));
        assert!(player.deck.contains(&Card::Strike(Grade::Plus))); // unchanged
        assert_eq!(
            events.iter().filter(|e| matches!(e, Event::CardUpgraded { .. })).count(),
            1
        );
    }

    // --- Tier 1: WarPaint ---

    #[test]
    fn warpaint_upgrades_2_skill_cards_in_deck() {
        let mut player = test_player();
        player.deck = vec![Card::Defend(Grade::Base), Card::Bloodletting(Grade::Base), Card::Strike(Grade::Base)];
        grant_relic(&mut player, Relic::WarPaint, &mut rng());
        assert!(player.deck.contains(&Card::Defend(Grade::Plus)));
        assert!(player.deck.contains(&Card::Bloodletting(Grade::Plus)));
        assert!(player.deck.contains(&Card::Strike(Grade::Base))); // attack unchanged
    }

    #[test]
    fn warpaint_skips_non_upgradeable_skills() {
        let mut player = test_player();
        player.deck = vec![Card::Disarm, Card::Defend(Grade::Base)]; // Disarm can't be upgraded
        let events = grant_relic(&mut player, Relic::WarPaint, &mut rng());
        assert!(player.deck.contains(&Card::Disarm)); // unchanged
        assert!(player.deck.contains(&Card::Defend(Grade::Plus)));
        assert_eq!(
            events.iter().filter(|e| matches!(e, Event::CardUpgraded { .. })).count(),
            1
        );
    }

    #[test]
    fn warpaint_emits_card_upgraded_events() {
        let mut player = test_player();
        player.deck = vec![Card::Defend(Grade::Base), Card::Defend(Grade::Base)];
        let events = grant_relic(&mut player, Relic::WarPaint, &mut rng());
        assert_eq!(
            events.iter().filter(|e| matches!(e, Event::CardUpgraded { .. })).count(),
            2
        );
    }

    // --- Tier 2: BurningBlood ---

    #[test]
    fn burning_blood_heals_6_hp_at_end_of_combat() {
        let mut player = test_player();
        player.hp = Hp(60);
        player.relics.push(Relic::BurningBlood);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(66));
    }

    #[test]
    fn burning_blood_emits_healed_event() {
        let mut player = test_player();
        player.hp = Hp(60);
        player.relics.push(Relic::BurningBlood);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert!(events.contains(&Event::Healed { amount: 6 }));
    }

    #[test]
    fn burning_blood_cannot_overheal() {
        let mut player = test_player();
        player.hp = Hp(77);
        player.relics.push(Relic::BurningBlood);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(80));
        assert!(events.contains(&Event::Healed { amount: 3 }));
    }

    #[test]
    fn burning_blood_at_full_hp_does_nothing() {
        let mut player = test_player();
        player.relics.push(Relic::BurningBlood);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(80));
        assert!(!events.iter().any(|e| matches!(e, Event::Healed { .. })));
    }

    // --- Tier 2: BlackBlood ---

    #[test]
    fn black_blood_heals_12_hp_at_end_of_combat() {
        let mut player = test_player();
        player.hp = Hp(50);
        player.relics.push(Relic::BlackBlood);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(62));
    }

    #[test]
    fn black_blood_cannot_overheal() {
        let mut player = test_player();
        player.hp = Hp(75);
        player.relics.push(Relic::BlackBlood);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(80));
    }

    #[test]
    fn black_blood_emits_healed_event() {
        let mut player = test_player();
        player.hp = Hp(50);
        player.relics.push(Relic::BlackBlood);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert!(events.contains(&Event::Healed { amount: 12 }));
    }

    #[test]
    fn both_burning_and_black_blood_both_heal() {
        let mut player = test_player();
        player.hp = Hp(50);
        player.relics.push(Relic::BurningBlood);
        player.relics.push(Relic::BlackBlood);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(68)); // 50 + 6 + 12
    }

    #[test]
    fn without_end_combat_relic_no_heal() {
        let mut player = test_player();
        player.hp = Hp(60);
        let mut events = vec![];
        apply_end_of_combat_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(60));
        assert!(events.is_empty());
    }

    // --- Tier 2: Anchor ---

    #[test]
    fn anchor_gives_10_block_at_combat_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Anchor);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.player.block, Block(10));
    }

    #[test]
    fn anchor_emits_player_blocked_event() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Anchor);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert!(events.contains(&Event::PlayerBlocked { amount: 10 }));
    }

    // --- Tier 2: Vajra ---

    #[test]
    fn vajra_grants_1_strength_at_combat_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Vajra);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(
            state.player.statuses.get(&StatusEffect::Strength),
            Some(&1)
        );
    }

    #[test]
    fn vajra_emits_status_applied_event() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Vajra);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert!(events.contains(&Event::StatusApplied {
            target: Target::Player,
            status: StatusEffect::Strength,
            stacks: 1,
        }));
    }

    // --- Tier 2: Lantern ---

    #[test]
    fn lantern_grants_plus_1_energy_and_max_energy_at_combat_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Lantern);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.player.energy, Energy(4));
        assert_eq!(state.player.max_energy, Energy(4));
    }

    #[test]
    fn lantern_emits_energy_gained_event() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Lantern);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert!(events.contains(&Event::EnergyGained { amount: 1 }));
    }

    // --- Tier 2: BloodVial ---

    #[test]
    fn blood_vial_heals_2_hp_at_combat_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.hp = Hp(70);
        state.player.relics.push(Relic::BloodVial);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.player.hp, Hp(72));
    }

    #[test]
    fn blood_vial_cannot_overheal() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::BloodVial); // already full at 80
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.player.hp, Hp(80));
        assert!(!events.iter().any(|e| matches!(e, Event::Healed { .. })));
    }

    // --- Tier 2: BagOfMarbles ---

    #[test]
    fn bag_of_marbles_applies_1_vulnerable_to_all_enemies() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.relics.push(Relic::BagOfMarbles);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(
            state.enemies[0].statuses.get(&StatusEffect::Vulnerable),
            Some(&1)
        );
        assert_eq!(
            state.enemies[1].statuses.get(&StatusEffect::Vulnerable),
            Some(&1)
        );
    }

    #[test]
    fn bag_of_marbles_emits_status_applied_for_each_enemy() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.relics.push(Relic::BagOfMarbles);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::StatusApplied { status: StatusEffect::Vulnerable, .. }))
                .count(),
            2
        );
    }

    // --- Tier 2: RedMask ---

    #[test]
    fn red_mask_applies_1_weak_to_all_enemies() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.relics.push(Relic::RedMask);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&1));
        assert_eq!(state.enemies[1].statuses.get(&StatusEffect::Weak), Some(&1));
    }

    // --- Tier 2: FestivePopper ---

    #[test]
    fn festive_popper_deals_9_damage_to_all_enemies_at_combat_start() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.relics.push(Relic::FestivePopper);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.enemies[0].hp, Hp(11));
        assert_eq!(state.enemies[1].hp, Hp(11));
    }

    #[test]
    fn festive_popper_emits_enemy_attacked_for_each_enemy() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.relics.push(Relic::FestivePopper);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(
            events.iter().filter(|e| matches!(e, Event::EnemyAttacked { .. })).count(),
            2
        );
    }

    #[test]
    fn festive_popper_emits_enemy_died_when_killing_enemy() {
        let mut state = combat_with_hand(vec![]);
        state.enemies[0].hp = Hp(5);
        state.player.relics.push(Relic::FestivePopper);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.enemies[0].hp, Hp(0));
        assert!(events.contains(&Event::EnemyDied));
    }

    // --- Tier 2: Pantograph ---

    #[test]
    fn pantograph_heals_25_at_boss_combat_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.hp = Hp(50);
        state.player.relics.push(Relic::Pantograph);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), true);
        assert_eq!(state.player.hp, Hp(75));
    }

    #[test]
    fn pantograph_does_not_heal_at_normal_combat_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.hp = Hp(50);
        state.player.relics.push(Relic::Pantograph);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.player.hp, Hp(50));
    }

    #[test]
    fn pantograph_cannot_overheal() {
        let mut state = combat_with_hand(vec![]);
        state.player.hp = Hp(70);
        state.player.relics.push(Relic::Pantograph);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), true);
        assert_eq!(state.player.hp, Hp(80));
    }

    // --- Tier 2: BagOfPreparation ---

    #[test]
    fn bag_of_preparation_draws_2_extra_cards_at_combat_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        state.player.relics.push(Relic::BagOfPreparation);
        let hand_before = state.player.hand.len();
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert_eq!(state.player.hand.len(), hand_before + 2);
    }

    #[test]
    fn bag_of_preparation_emits_cards_drawn_event() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        state.player.relics.push(Relic::BagOfPreparation);
        let mut events = vec![];
        apply_combat_start_relics(&mut state, &mut events, &mut rng(), false);
        assert!(events.contains(&Event::CardsDrawn { count: 2 }));
    }

    // --- Tier 3: MercuryHourglass ---

    #[test]
    fn mercury_hourglass_deals_3_damage_to_all_enemies_at_turn_start() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.relics.push(Relic::MercuryHourglass);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(17));
        assert_eq!(state.enemies[1].hp, Hp(17));
    }

    #[test]
    fn mercury_hourglass_fires_every_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::MercuryHourglass);
        let mut events = vec![];
        state.turn = 5;
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(17));
    }

    #[test]
    fn mercury_hourglass_kills_enemy_emits_enemy_died() {
        let mut state = combat_with_hand(vec![]);
        state.enemies[0].hp = Hp(2);
        state.player.relics.push(Relic::MercuryHourglass);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(0));
        assert!(events.contains(&Event::EnemyDied));
    }

    #[test]
    fn mercury_hourglass_killing_last_enemy_sets_victory() {
        let mut state = combat_with_hand(vec![]);
        state.enemies[0].hp = Hp(2);
        state.player.relics.push(Relic::MercuryHourglass);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    // --- Tier 3: CaptainsWheel ---

    #[test]
    fn captains_wheel_gains_18_block_on_turn_3() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 3;
        state.player.relics.push(Relic::CaptainsWheel);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.block, Block(18));
    }

    #[test]
    fn captains_wheel_does_not_fire_on_turn_2() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 2;
        state.player.relics.push(Relic::CaptainsWheel);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn captains_wheel_does_not_fire_on_turn_4() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 4;
        state.player.relics.push(Relic::CaptainsWheel);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn captains_wheel_emits_player_blocked_event() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 3;
        state.player.relics.push(Relic::CaptainsWheel);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert!(events.contains(&Event::PlayerBlocked { amount: 18 }));
    }

    // --- Tier 3: Chandelier ---

    #[test]
    fn chandelier_gains_3_energy_on_turn_3() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 3;
        state.player.relics.push(Relic::Chandelier);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(6));
    }

    #[test]
    fn chandelier_does_not_fire_on_other_turns() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 4;
        state.player.relics.push(Relic::Chandelier);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(3));
    }

    // --- Tier 3: Candelabra ---

    #[test]
    fn candelabra_gains_2_energy_on_turn_2() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 2;
        state.player.relics.push(Relic::Candelabra);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(5));
    }

    #[test]
    fn candelabra_does_not_fire_on_turn_3() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 3;
        state.player.relics.push(Relic::Candelabra);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(3));
    }

    // --- Tier 3: HornCleat ---

    #[test]
    fn horn_cleat_gains_14_block_on_turn_2() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 2;
        state.player.relics.push(Relic::HornCleat);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.block, Block(14));
    }

    #[test]
    fn horn_cleat_does_not_fire_on_turn_3() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 3;
        state.player.relics.push(Relic::HornCleat);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.block, Block(0));
    }

    // --- Tier 3: HappyFlower ---

    #[test]
    fn happy_flower_gains_1_energy_on_turn_3() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 3;
        state.player.relics.push(Relic::HappyFlower);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(4));
    }

    #[test]
    fn happy_flower_fires_again_on_turn_6() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 6;
        state.player.relics.push(Relic::HappyFlower);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(4));
    }

    #[test]
    fn happy_flower_does_not_fire_on_turn_4() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 4;
        state.player.relics.push(Relic::HappyFlower);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(3));
    }

    // --- Tier 3: Pendulum ---

    #[test]
    fn pendulum_draws_1_card_on_turn_3() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        state.turn = 3;
        state.player.relics.push(Relic::Pendulum);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn pendulum_fires_again_on_turn_6() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        state.turn = 6;
        state.player.relics.push(Relic::Pendulum);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn pendulum_does_not_fire_on_turn_4() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        state.turn = 4;
        state.player.relics.push(Relic::Pendulum);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.hand.len(), 0);
    }

    #[test]
    fn pendulum_emits_cards_drawn_event() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        state.turn = 3;
        state.player.relics.push(Relic::Pendulum);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert!(events.contains(&Event::CardsDrawn { count: 1 }));
    }

    // --- Tier 3: StoneCalendar ---

    #[test]
    fn stone_calendar_deals_52_to_all_enemies_on_turn_7() {
        let mut state = combat_with_two_enemies(vec![]);
        state.enemies[0].hp = Hp(100);
        state.enemies[1].hp = Hp(100);
        state.turn = 7;
        state.player.relics.push(Relic::StoneCalendar);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(48));
        assert_eq!(state.enemies[1].hp, Hp(48));
    }

    #[test]
    fn stone_calendar_does_not_fire_on_turn_6() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 6;
        state.player.relics.push(Relic::StoneCalendar);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    #[test]
    fn stone_calendar_kills_enemy_and_sets_victory() {
        let mut state = combat_with_hand(vec![]);
        state.turn = 7;
        state.player.relics.push(Relic::StoneCalendar);
        let mut events = vec![];
        apply_turn_start_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(0));
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    // --- Tier 3: Orichalcum ---

    #[test]
    fn orichalcum_gains_6_block_when_ending_turn_with_no_block() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Orichalcum);
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 0);
        assert_eq!(state.player.block, Block(6));
    }

    #[test]
    fn orichalcum_does_not_fire_when_player_has_block() {
        let mut state = combat_with_hand(vec![]);
        state.player.block = Block(5);
        state.player.relics.push(Relic::Orichalcum);
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 0);
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn orichalcum_emits_player_blocked_event() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Orichalcum);
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 0);
        assert!(events.contains(&Event::PlayerBlocked { amount: 6 }));
    }

    // --- Tier 3: CloakClasp ---

    #[test]
    fn cloak_clasp_gains_1_block_per_card_in_hand_at_turn_end() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::CloakClasp);
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 3);
        assert_eq!(state.player.block, Block(3));
    }

    #[test]
    fn cloak_clasp_gives_no_block_with_empty_hand() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::CloakClasp);
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 0);
        assert_eq!(state.player.block, Block(0));
        assert!(!events.iter().any(|e| matches!(e, Event::PlayerBlocked { .. })));
    }

    #[test]
    fn cloak_clasp_emits_player_blocked_event() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::CloakClasp);
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 4);
        assert!(events.contains(&Event::PlayerBlocked { amount: 4 }));
    }

    // --- Tier 3: RegalPillow ---

    #[test]
    fn regal_pillow_heals_15_hp_on_rest() {
        let mut player = test_player();
        player.hp = Hp(50);
        player.relics.push(Relic::RegalPillow);
        let mut events = vec![];
        apply_rest_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(65));
    }

    #[test]
    fn regal_pillow_cannot_overheal_on_rest() {
        let mut player = test_player();
        player.hp = Hp(70);
        player.relics.push(Relic::RegalPillow);
        let mut events = vec![];
        apply_rest_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(80));
    }

    #[test]
    fn regal_pillow_emits_healed_event() {
        let mut player = test_player();
        player.hp = Hp(50);
        player.relics.push(Relic::RegalPillow);
        let mut events = vec![];
        apply_rest_relics(&mut player, &mut events);
        assert!(events.contains(&Event::Healed { amount: 15 }));
    }

    #[test]
    fn without_regal_pillow_no_extra_rest_heal() {
        let mut player = test_player();
        player.hp = Hp(50);
        let mut events = vec![];
        apply_rest_relics(&mut player, &mut events);
        assert_eq!(player.hp, Hp(50));
        assert!(events.is_empty());
    }

    #[test]
    fn relic_id_roundtrip_for_all_relics() {
        let all = [
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
        ];
        for relic in all {
            assert_eq!(Relic::from_id(relic.id()), Some(relic));
        }
    }

    #[test]
    fn from_id_returns_none_for_unknown_id() {
        assert_eq!(Relic::from_id("not-a-relic"), None);
    }

    // --- Tier 4: Nunchaku ---

    #[test]
    fn nunchaku_grants_1_energy_on_10th_attack_total() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Nunchaku);
        state.attacks_this_combat = 10;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.energy, Energy(4));
        assert!(events.contains(&Event::EnergyGained { amount: 1 }));
    }

    #[test]
    fn nunchaku_does_not_fire_on_9th_attack() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Nunchaku);
        state.attacks_this_combat = 9;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.energy, Energy(3));
        assert!(!events.iter().any(|e| matches!(e, Event::EnergyGained { .. })));
    }

    #[test]
    fn nunchaku_fires_again_on_20th_attack() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Nunchaku);
        state.attacks_this_combat = 20;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.energy, Energy(4));
    }

    #[test]
    fn nunchaku_does_not_fire_for_skills() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Nunchaku);
        state.skills_this_combat = 10;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Skill, &mut rng());
        assert_eq!(state.player.energy, Energy(3));
    }

    // --- Tier 4: Ornamental Fan ---

    #[test]
    fn ornamental_fan_grants_4_block_on_3rd_attack_this_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::OrnamentalFan);
        state.attacks_this_turn = 3;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.block, Block(4));
        assert!(events.contains(&Event::PlayerBlocked { amount: 4 }));
    }

    #[test]
    fn ornamental_fan_does_not_fire_on_2nd_attack() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::OrnamentalFan);
        state.attacks_this_turn = 2;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn ornamental_fan_fires_again_on_6th_attack_this_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::OrnamentalFan);
        state.attacks_this_turn = 6;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.block, Block(4));
    }

    // --- Tier 4: Shuriken ---

    #[test]
    fn shuriken_grants_1_strength_on_3rd_attack_this_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Shuriken);
        state.attacks_this_turn = 3;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&1));
    }

    #[test]
    fn shuriken_does_not_fire_on_2nd_attack() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Shuriken);
        state.attacks_this_turn = 2;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert!(!state.player.statuses.contains_key(&StatusEffect::Strength));
    }

    // --- Tier 4: Kunai ---

    #[test]
    fn kunai_grants_1_dexterity_on_3rd_attack_this_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Kunai);
        state.attacks_this_turn = 3;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.statuses.get(&StatusEffect::Dexterity), Some(&1));
    }

    #[test]
    fn kunai_does_not_fire_on_2nd_attack() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Kunai);
        state.attacks_this_turn = 2;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert!(!state.player.statuses.contains_key(&StatusEffect::Dexterity));
    }

    // --- Tier 4: Kusarigama ---

    #[test]
    fn kusarigama_deals_6_to_enemy_on_3rd_attack_this_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Kusarigama);
        state.attacks_this_turn = 3;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(14));
        assert!(events.iter().any(|e| matches!(e, Event::EnemyAttacked { raw: 6, .. })));
    }

    #[test]
    fn kusarigama_does_not_fire_on_2nd_attack() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Kusarigama);
        state.attacks_this_turn = 2;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    // --- Tier 4: Letter Opener ---

    #[test]
    fn letter_opener_deals_5_to_all_on_3rd_skill_this_turn() {
        let mut state = combat_with_two_enemies(vec![]);
        state.player.relics.push(Relic::LetterOpener);
        state.skills_this_turn = 3;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Skill, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(15));
        assert_eq!(state.enemies[1].hp, Hp(15));
    }

    #[test]
    fn letter_opener_does_not_fire_on_2nd_skill() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::LetterOpener);
        state.skills_this_turn = 2;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Skill, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    #[test]
    fn letter_opener_does_not_fire_for_attacks() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::LetterOpener);
        state.attacks_this_turn = 3;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    // --- Tier 4: Tuning Fork ---

    #[test]
    fn tuning_fork_grants_7_block_on_10th_skill_total() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::TuningFork);
        state.skills_this_combat = 10;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Skill, &mut rng());
        assert_eq!(state.player.block, Block(7));
        assert!(events.contains(&Event::PlayerBlocked { amount: 7 }));
    }

    #[test]
    fn tuning_fork_does_not_fire_on_9th_skill() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::TuningFork);
        state.skills_this_combat = 9;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Skill, &mut rng());
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn tuning_fork_does_not_fire_for_attacks() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::TuningFork);
        state.attacks_this_combat = 10;
        let mut events = vec![];
        apply_card_play_relics(&mut state, &mut events, CardType::Attack, &mut rng());
        assert_eq!(state.player.block, Block(0));
    }

    // --- Tier 4: Gremlin Horn ---

    #[test]
    fn gremlin_horn_grants_1_energy_on_enemy_death() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::GremlinHorn);
        let mut events = vec![];
        apply_enemy_died_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(4));
        assert!(events.contains(&Event::EnergyGained { amount: 1 }));
    }

    #[test]
    fn gremlin_horn_draws_1_card_on_enemy_death() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        state.player.relics.push(Relic::GremlinHorn);
        let mut events = vec![];
        apply_enemy_died_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.hand.len(), 1);
        assert!(events.contains(&Event::CardsDrawn { count: 1 }));
    }

    #[test]
    fn without_gremlin_horn_no_energy_on_death() {
        let mut state = combat_with_hand(vec![]);
        let mut events = vec![];
        apply_enemy_died_relics(&mut state, &mut events, &mut rng());
        assert_eq!(state.player.energy, Energy(3));
        assert!(events.is_empty());
    }

    // --- Tier 4: Pocketwatch ---

    #[test]
    fn pocketwatch_sets_extra_draws_when_3_or_fewer_cards_played() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Pocketwatch);
        state.cards_played_this_turn = 3;
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 0);
        assert_eq!(state.extra_draws_next_turn, 3);
    }

    #[test]
    fn pocketwatch_fires_when_0_cards_played() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Pocketwatch);
        state.cards_played_this_turn = 0;
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 0);
        assert_eq!(state.extra_draws_next_turn, 3);
    }

    #[test]
    fn pocketwatch_does_not_fire_when_4_cards_played() {
        let mut state = combat_with_hand(vec![]);
        state.player.relics.push(Relic::Pocketwatch);
        state.cards_played_this_turn = 4;
        let mut events = vec![];
        apply_turn_end_relics(&mut state, &mut events, 0);
        assert_eq!(state.extra_draws_next_turn, 0);
    }
}
