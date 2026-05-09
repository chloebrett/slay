    use super::*;
    use crate::combat::{combat_with_hand, combat_with_deck, combat_with_two_enemies, apply_combat_command, CombatPhase, Event, Target};
    use crate::run::{Command, CommandError};
    use crate::status::{StatusEffect, get_stacks};
    use crate::types::{Block, Energy, Hp};
    use crate::rng::NoOpRng;

    fn rng() -> NoOpRng { NoOpRng }

    fn apply_command(
        state: crate::combat::CombatState,
        cmd: Command,
        rng: &mut impl crate::rng::Rng,
    ) -> Result<(crate::combat::CombatState, Vec<Event>), CommandError> {
        apply_combat_command(state, cmd, rng)
    }

    // --- Strike ---

    #[test]
    fn strike_deals_6_damage_to_enemy() {
        let state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(14));
    }

    #[test]
    fn strike_emits_player_attacked_event() {
        let state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerAttacked { raw: 6, damage: 6 }));
    }

    #[test]
    fn strike_killing_enemy_yields_victory() {
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    #[test]
    fn strike_killing_enemy_emits_enemy_died_event() {
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].hp = Hp(1);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyDied));
    }

    #[test]
    fn strike_moves_to_discard_after_play() {
        let state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 0);
        assert_eq!(state.player.discard_pile, vec![Card::Strike(Grade::Base)]);
    }

    #[test]
    fn strike_goes_to_discard_not_exhaust() {
        let state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.discard_pile, vec![Card::Strike(Grade::Base)]);
        assert!(state.player.exhaust_pile.is_empty());
    }

    // --- Defend ---

    #[test]
    fn defend_grants_5_block() {
        let state = combat_with_hand(vec![Card::Defend(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn defend_emits_player_blocked_event() {
        let state = combat_with_hand(vec![Card::Defend(Grade::Base)]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerBlocked { amount: 5 }));
    }

    // --- Bash ---

    #[test]
    fn bash_deals_8_damage_to_enemy() {
        let state = combat_with_hand(vec![Card::Bash(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
    }

    #[test]
    fn bash_costs_2_energy() {
        let state = combat_with_hand(vec![Card::Bash(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(1));
    }

    #[test]
    fn bash_applies_2_vulnerable_to_enemy() {
        let state = combat_with_hand(vec![Card::Bash(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&2));
    }

    #[test]
    fn bash_emits_status_applied_event() {
        let state = combat_with_hand(vec![Card::Bash(Grade::Base)]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::StatusApplied {
            target: Target::Enemy,
            status: StatusEffect::Vulnerable,
            stacks: 2,
        }));
    }

    #[test]
    fn strike_damage_boosted_against_vulnerable_enemy() {
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::Vulnerable, 2);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerAttacked { raw: 9, damage: 9 }));
    }

    // --- Clothesline ---

    #[test]
    fn clothesline_deals_12_damage_to_enemy() {
        let state = combat_with_hand(vec![Card::Clothesline(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(8));
    }

    #[test]
    fn clothesline_applies_2_weak_to_enemy() {
        let state = combat_with_hand(vec![Card::Clothesline(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&2));
    }

    // --- Deadly Poison ---

    #[test]
    fn deadly_poison_applies_5_poison_to_enemy() {
        let state = combat_with_hand(vec![Card::DeadlyPoison(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Poison), Some(&5));
    }

    #[test]
    fn deadly_poison_deals_no_direct_damage() {
        let state = combat_with_hand(vec![Card::DeadlyPoison(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    // --- CardType ---

    #[test]
    fn strike_card_type_is_attack() {
        assert_eq!(Card::Strike(Grade::Base).card_type(), CardType::Attack);
    }

    #[test]
    fn bash_card_type_is_attack() {
        assert_eq!(Card::Bash(Grade::Base).card_type(), CardType::Attack);
    }

    #[test]
    fn clothesline_card_type_is_attack() {
        assert_eq!(Card::Clothesline(Grade::Base).card_type(), CardType::Attack);
    }

    #[test]
    fn defend_card_type_is_skill() {
        assert_eq!(Card::Defend(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn deadly_poison_card_type_is_skill() {
        assert_eq!(Card::DeadlyPoison(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn disarm_card_type_is_skill() {
        assert_eq!(Card::Disarm.card_type(), CardType::Skill);
    }

    #[test]
    fn inflame_card_type_is_power() {
        assert_eq!(Card::Inflame(Grade::Base).card_type(), CardType::Power);
    }

    // --- Inflame ---

    #[test]
    fn inflame_grants_2_strength_to_player() {
        let state = combat_with_hand(vec![Card::Inflame(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&2));
    }

    #[test]
    fn inflame_is_absorbed_not_discarded() {
        let state = combat_with_hand(vec![Card::Inflame(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn inflame_is_absorbed_not_exhausted() {
        let state = combat_with_hand(vec![Card::Inflame(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.is_empty());
    }

    // --- Upgraded effects ---

    #[test]
    fn strike_plus_deals_9_damage() {
        let state = combat_with_hand(vec![Card::Strike(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(11));
    }

    #[test]
    fn defend_plus_grants_8_block() {
        let state = combat_with_hand(vec![Card::Defend(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(8));
    }

    #[test]
    fn bash_plus_deals_10_damage() {
        let state = combat_with_hand(vec![Card::Bash(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    #[test]
    fn bash_plus_applies_3_vulnerable() {
        let state = combat_with_hand(vec![Card::Bash(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&3));
    }

    #[test]
    fn clothesline_plus_deals_14_damage() {
        let state = combat_with_hand(vec![Card::Clothesline(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(6));
    }

    #[test]
    fn clothesline_plus_applies_3_weak() {
        let state = combat_with_hand(vec![Card::Clothesline(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&3));
    }

    #[test]
    fn inflame_plus_grants_3_strength() {
        let state = combat_with_hand(vec![Card::Inflame(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&3));
    }

    #[test]
    fn deadly_poison_plus_applies_7_poison() {
        let state = combat_with_hand(vec![Card::DeadlyPoison(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Poison), Some(&7));
    }

    // --- Card::upgrade() ---

    #[test]
    fn upgrading_strike_gives_strike_plus() {
        assert_eq!(Card::Strike(Grade::Base).upgrade(), Some(Card::Strike(Grade::Plus)));
    }

    #[test]
    fn upgrading_defend_gives_defend_plus() {
        assert_eq!(Card::Defend(Grade::Base).upgrade(), Some(Card::Defend(Grade::Plus)));
    }

    #[test]
    fn upgrading_bash_gives_bash_plus() {
        assert_eq!(Card::Bash(Grade::Base).upgrade(), Some(Card::Bash(Grade::Plus)));
    }

    #[test]
    fn upgrading_clothesline_gives_clothesline_plus() {
        assert_eq!(Card::Clothesline(Grade::Base).upgrade(), Some(Card::Clothesline(Grade::Plus)));
    }

    #[test]
    fn upgrading_inflame_gives_inflame_plus() {
        assert_eq!(Card::Inflame(Grade::Base).upgrade(), Some(Card::Inflame(Grade::Plus)));
    }

    #[test]
    fn upgrading_deadly_poison_gives_deadly_poison_plus() {
        assert_eq!(Card::DeadlyPoison(Grade::Base).upgrade(), Some(Card::DeadlyPoison(Grade::Plus)));
    }

    #[test]
    fn upgrading_plus_card_returns_none() {
        assert_eq!(Card::Strike(Grade::Plus).upgrade(), None);
    }

    #[test]
    fn disarm_cannot_be_upgraded() {
        assert_eq!(Card::Disarm.upgrade(), None);
    }

    // --- Disarm ---

    #[test]
    fn disarm_applies_minus_2_strength_to_enemy() {
        let state = combat_with_hand(vec![Card::Disarm]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Strength), Some(&-2));
    }

    #[test]
    fn disarm_goes_to_exhaust_pile_not_discard() {
        let state = combat_with_hand(vec![Card::Disarm]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.exhaust_pile, vec![Card::Disarm]);
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn disarm_emits_card_exhausted_event() {
        let state = combat_with_hand(vec![Card::Disarm]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::CardExhausted { card: Card::Disarm }));
    }

    // --- Iron Wave ---

    #[test]
    fn iron_wave_deals_5_damage_and_grants_5_block() {
        let state = combat_with_hand(vec![Card::IronWave(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(15));
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn iron_wave_plus_deals_7_damage_and_grants_7_block() {
        let state = combat_with_hand(vec![Card::IronWave(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(13));
        assert_eq!(state.player.block, Block(7));
    }

    // --- Spot Weakness ---

    #[test]
    fn spot_weakness_grants_3_strength_when_enemy_intends_to_attack() {
        let state = combat_with_hand(vec![Card::SpotWeakness(Grade::Base)]);
        // default enemy intent is Attack
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&3));
    }

    #[test]
    fn spot_weakness_grants_no_strength_when_enemy_defends() {
        let mut state = combat_with_hand(vec![Card::SpotWeakness(Grade::Base)]);
        use crate::enemies::Move;
        state.enemies[0].move_ = Move::LouseBlock;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), None);
    }

    #[test]
    fn spot_weakness_plus_grants_4_strength() {
        let state = combat_with_hand(vec![Card::SpotWeakness(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength), Some(&4));
    }

    // --- Twin Strike ---

    #[test]
    fn twin_strike_deals_5_damage_twice() {
        let state = combat_with_hand(vec![Card::TwinStrike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    #[test]
    fn twin_strike_plus_deals_7_damage_twice() {
        let state = combat_with_hand(vec![Card::TwinStrike(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(6));
    }

    // --- Bludgeon ---

    #[test]
    fn bludgeon_deals_32_damage() {
        let mut state = combat_with_hand(vec![Card::Bludgeon(Grade::Base)]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(18));
    }

    #[test]
    fn bludgeon_plus_deals_42_damage() {
        let mut state = combat_with_hand(vec![Card::Bludgeon(Grade::Plus)]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(8));
    }

    // --- Impervious ---

    #[test]
    fn impervious_grants_30_block() {
        let state = combat_with_hand(vec![Card::Impervious(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(30));
    }

    #[test]
    fn impervious_goes_to_exhaust_pile() {
        let state = combat_with_hand(vec![Card::Impervious(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.exhaust_pile, vec![Card::Impervious(Grade::Base)]);
        assert!(state.player.discard_pile.is_empty());
    }

    // --- Seeing Red ---

    #[test]
    fn seeing_red_gains_2_energy() {
        let mut state = combat_with_hand(vec![Card::SeeingRed(Grade::Base)]);
        state.player.energy = Energy(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(2)); // spent 1 to play, gained 2
    }

    #[test]
    fn seeing_red_goes_to_exhaust_pile() {
        let state = combat_with_hand(vec![Card::SeeingRed(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.exhaust_pile, vec![Card::SeeingRed(Grade::Base)]);
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn seeing_red_plus_costs_0_energy() {
        assert_eq!(Card::SeeingRed(Grade::Plus).energy_cost(), Energy(0));
    }

    // --- Pummel ---

    #[test]
    fn pummel_deals_2_damage_4_times() {
        let state = combat_with_hand(vec![Card::Pummel(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
    }

    #[test]
    fn pummel_plus_deals_2_damage_5_times() {
        let state = combat_with_hand(vec![Card::Pummel(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    #[test]
    fn pummel_goes_to_exhaust_pile() {
        let state = combat_with_hand(vec![Card::Pummel(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.exhaust_pile, vec![Card::Pummel(Grade::Base)]);
    }

    // --- Uppercut ---

    #[test]
    fn uppercut_deals_13_damage() {
        let state = combat_with_hand(vec![Card::Uppercut(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(7));
    }

    #[test]
    fn uppercut_applies_1_weak_and_1_vulnerable() {
        let state = combat_with_hand(vec![Card::Uppercut(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&1));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&1));
    }

    #[test]
    fn uppercut_plus_applies_2_weak_and_2_vulnerable() {
        let state = combat_with_hand(vec![Card::Uppercut(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak), Some(&2));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&2));
    }

    // --- True Grit ---

    #[test]
    fn true_grit_grants_7_block() {
        let state = combat_with_hand(vec![Card::TrueGrit(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(7));
    }

    #[test]
    fn true_grit_exhausts_a_card_from_hand() {
        let state = combat_with_hand(vec![Card::TrueGrit(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.exhaust_pile, vec![Card::Strike(Grade::Base)]);
    }

    #[test]
    fn true_grit_does_not_exhaust_from_hand_when_hand_is_empty() {
        let state = combat_with_hand(vec![Card::TrueGrit(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.is_empty());
    }

    #[test]
    fn true_grit_plus_grants_9_block() {
        let state = combat_with_hand(vec![Card::TrueGrit(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(9));
    }

    // --- Thunderclap ---

    #[test]
    fn thunderclap_deals_4_damage_and_applies_1_vulnerable_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Thunderclap(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(16));
        assert_eq!(state.enemies[1].hp, Hp(16));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&1));
        assert_eq!(state.enemies[1].statuses.get(&StatusEffect::Vulnerable), Some(&1));
    }

    #[test]
    fn thunderclap_plus_deals_7_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Thunderclap(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(13));
        assert_eq!(state.enemies[1].hp, Hp(13));
    }

    // --- Cleave ---

    #[test]
    fn cleave_deals_8_damage_to_single_enemy() {
        let state = combat_with_hand(vec![Card::Cleave(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
    }

    #[test]
    fn cleave_deals_8_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Cleave(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
        assert_eq!(state.enemies[1].hp, Hp(12));
    }

    #[test]
    fn cleave_plus_deals_11_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Cleave(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(9));
        assert_eq!(state.enemies[1].hp, Hp(9));
    }

    #[test]
    fn cleave_card_type_is_attack() {
        assert_eq!(Card::Cleave(Grade::Base).card_type(), CardType::Attack);
    }

    #[test]
    fn upgrading_cleave_gives_cleave_plus() {
        assert_eq!(Card::Cleave(Grade::Base).upgrade(), Some(Card::Cleave(Grade::Plus)));
    }

    // --- Card IDs ---

    #[test]
    fn pommel_strike_has_kebab_id() {
        assert_eq!(Card::PommelStrike(Grade::Base).id(), "pommel-strike");
    }

    #[test]
    fn shrug_it_off_has_kebab_id() {
        assert_eq!(Card::ShrugItOff(Grade::Base).id(), "shrug-it-off");
    }

    #[test]
    fn bloodletting_roundtrips_through_id() {
        assert_eq!(Card::from_id("bloodletting"), Some(Card::Bloodletting(Grade::Base)));
    }

    #[test]
    fn unknown_id_returns_none() {
        assert_eq!(Card::from_id("not-a-card"), None);
    }

    // --- Pommel Strike ---

    #[test]
    fn pommel_strike_deals_9_damage() {
        let state = combat_with_hand(vec![Card::PommelStrike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(11));
    }

    #[test]
    fn pommel_strike_draws_1_card() {
        let mut state = combat_with_hand(vec![Card::PommelStrike(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn pommel_strike_plus_deals_10_damage() {
        let state = combat_with_hand(vec![Card::PommelStrike(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    #[test]
    fn pommel_strike_plus_draws_2_cards() {
        let mut state = combat_with_hand(vec![Card::PommelStrike(Grade::Plus)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 2);
    }

    // --- Shrug It Off ---

    #[test]
    fn shrug_it_off_grants_8_block() {
        let state = combat_with_hand(vec![Card::ShrugItOff(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(8));
    }

    #[test]
    fn shrug_it_off_draws_1_card() {
        let mut state = combat_with_hand(vec![Card::ShrugItOff(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn shrug_it_off_plus_grants_11_block() {
        let state = combat_with_hand(vec![Card::ShrugItOff(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(11));
    }

    // --- Body Slam ---

    #[test]
    fn body_slam_deals_damage_equal_to_block() {
        let mut state = combat_with_hand(vec![Card::BodySlam(Grade::Base)]);
        state.player.block = Block(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    #[test]
    fn body_slam_with_no_block_deals_zero_damage() {
        let state = combat_with_hand(vec![Card::BodySlam(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    #[test]
    fn body_slam_plus_costs_0_energy() {
        assert_eq!(Card::BodySlam(Grade::Plus).energy_cost(), Energy(0));
    }

    // --- Anger ---

    #[test]
    fn anger_deals_6_damage() {
        let state = combat_with_hand(vec![Card::Anger(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(14));
    }

    #[test]
    fn anger_adds_copy_to_discard() {
        let state = combat_with_hand(vec![Card::Anger(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.discard_pile.iter().filter(|c| **c == Card::Anger(Grade::Base)).count(), 2);
    }

    #[test]
    fn anger_plus_deals_8_damage() {
        let state = combat_with_hand(vec![Card::Anger(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
    }

    #[test]
    fn anger_plus_adds_anger_plus_copy_to_discard() {
        let state = combat_with_hand(vec![Card::Anger(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.discard_pile.iter().filter(|c| **c == Card::Anger(Grade::Plus)).count(), 2);
    }

    // --- Reckless Charge ---

    #[test]
    fn reckless_charge_deals_7_damage() {
        let state = combat_with_hand(vec![Card::RecklessCharge(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(13));
    }

    #[test]
    fn reckless_charge_costs_0_energy() {
        assert_eq!(Card::RecklessCharge(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn reckless_charge_shuffles_dazed_into_draw_pile() {
        let state = combat_with_hand(vec![Card::RecklessCharge(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.draw_pile.contains(&Card::Dazed) || state.player.discard_pile.contains(&Card::Dazed));
    }

    #[test]
    fn reckless_charge_plus_deals_10_damage() {
        let state = combat_with_hand(vec![Card::RecklessCharge(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    // --- Entrench ---

    #[test]
    fn entrench_doubles_current_block() {
        let mut state = combat_with_hand(vec![Card::Entrench(Grade::Base)]);
        state.player.block = Block(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(20));
    }

    #[test]
    fn entrench_with_no_block_stays_zero() {
        let state = combat_with_hand(vec![Card::Entrench(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn entrench_plus_costs_1_energy() {
        assert_eq!(Card::Entrench(Grade::Plus).energy_cost(), Energy(1));
    }

    // --- Bloodletting ---

    #[test]
    fn bloodletting_gains_2_energy() {
        let mut state = combat_with_hand(vec![Card::Bloodletting(Grade::Base)]);
        state.player.energy = Energy(0); // drain energy so we can measure the gain
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(2));
    }

    #[test]
    fn bloodletting_costs_3_hp() {
        let state = combat_with_hand(vec![Card::Bloodletting(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(77));
    }

    #[test]
    fn bloodletting_emits_energy_gained_event() {
        let state = combat_with_hand(vec![Card::Bloodletting(Grade::Base)]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::EnergyGained { amount: 2 }));
    }

    #[test]
    fn bloodletting_plus_gains_3_energy() {
        let mut state = combat_with_hand(vec![Card::Bloodletting(Grade::Plus)]);
        state.player.energy = Energy(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(3));
    }

    // --- Hemokinesis ---

    #[test]
    fn hemokinesis_deals_15_damage() {
        let state = combat_with_hand(vec![Card::Hemokinesis(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(5));
    }

    #[test]
    fn hemokinesis_costs_2_hp() {
        let state = combat_with_hand(vec![Card::Hemokinesis(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(78));
    }

    #[test]
    fn hemokinesis_plus_deals_20_damage() {
        let mut state = combat_with_hand(vec![Card::Hemokinesis(Grade::Plus)]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(30));
    }

    // --- Self-damage defeat ---

    #[test]
    fn self_damage_killing_player_yields_defeat() {
        let mut state = combat_with_hand(vec![Card::Bloodletting(Grade::Base)]);
        state.player.hp = Hp(1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);
    }

    // --- Injury ---

    #[test]
    fn injury_card_type_is_curse() {
        assert_eq!(Card::Injury.card_type(), CardType::Curse);
    }

    #[test]
    fn injury_is_not_playable() {
        assert!(!Card::Injury.is_playable());
    }

    #[test]
    fn injury_name_is_injury() {
        assert_eq!(Card::Injury.name(), "Injury");
    }

    #[test]
    fn injury_id_is_injury_string() {
        assert_eq!(Card::Injury.id(), "injury");
    }

    #[test]
    fn injury_id_round_trips() {
        let id = Card::Injury.id();
        assert_eq!(Card::from_id(id), Some(Card::Injury));
    }

    // --- Dazed (ethereal) ---

    #[test]
    fn dazed_card_type_is_status() {
        assert_eq!(Card::Dazed.card_type(), CardType::Status);
    }

    #[test]
    fn dazed_is_ethereal() {
        assert!(Card::Dazed.is_ethereal());
    }

    #[test]
    fn dazed_in_hand_exhausts_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command, CombatPhase};
        let mut state = combat_with_hand(vec![Card::Dazed]);
        state.phase = CombatPhase::PlayerTurn;
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Dazed));
        assert!(!state.player.discard_pile.contains(&Card::Dazed));
    }

    // --- Clumsy ---

    #[test]
    fn clumsy_card_type_is_status() {
        assert_eq!(Card::Clumsy.card_type(), CardType::Status);
    }

    #[test]
    fn clumsy_is_not_playable() {
        assert!(!Card::Clumsy.is_playable());
    }

    #[test]
    fn clumsy_is_ethereal() {
        assert!(Card::Clumsy.is_ethereal());
    }

    #[test]
    fn clumsy_id_is_clumsy_string() {
        assert_eq!(Card::Clumsy.id(), "clumsy");
    }

    #[test]
    fn clumsy_in_hand_exhausts_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command, CombatPhase};
        let mut state = combat_with_hand(vec![Card::Clumsy]);
        state.phase = CombatPhase::PlayerTurn;
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Clumsy));
        assert!(!state.player.discard_pile.contains(&Card::Clumsy));
    }

    #[test]
    fn non_ethereal_card_in_hand_discards_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command, CombatPhase};
        let mut state = combat_with_hand(vec![Card::Strike(Grade::Base)]);
        state.phase = CombatPhase::PlayerTurn;
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert!(state.player.discard_pile.contains(&Card::Strike(Grade::Base)));
        assert!(!state.player.exhaust_pile.contains(&Card::Strike(Grade::Base)));
    }

    // --- Decay ---

    #[test]
    fn decay_card_type_is_curse() {
        assert_eq!(Card::Decay.card_type(), CardType::Curse);
    }

    #[test]
    fn decay_is_not_playable() {
        assert!(!Card::Decay.is_playable());
    }

    #[test]
    fn decay_id_is_decay_string() {
        assert_eq!(Card::Decay.id(), "decay");
    }

    #[test]
    fn decay_in_hand_deals_2_damage_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let state = combat_with_hand(vec![Card::Decay]);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(78));
    }

    #[test]
    fn decay_in_draw_pile_does_not_deal_damage_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile.push(Card::Decay);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn decay_damage_is_blockable() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let mut state = combat_with_hand(vec![Card::Decay]);
        state.player.block = Block(5);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
        assert_eq!(state.player.block, Block(3));
    }

    #[test]
    fn two_decays_in_hand_deal_4_damage_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let state = combat_with_hand(vec![Card::Decay, Card::Decay]);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(76));
    }

    // --- Regret ---

    #[test]
    fn regret_card_type_is_curse() {
        assert_eq!(Card::Regret.card_type(), CardType::Curse);
    }

    #[test]
    fn regret_is_not_playable() {
        assert!(!Card::Regret.is_playable());
    }

    #[test]
    fn regret_id_is_regret_string() {
        assert_eq!(Card::Regret.id(), "regret");
    }

    #[test]
    fn regret_deals_1_damage_bypassing_block_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let mut state = combat_with_hand(vec![Card::Regret]);
        state.player.block = Block(10);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(79));
        assert_eq!(state.player.block, Block(10));
    }

    #[test]
    fn regret_deals_damage_equal_to_hand_size() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let state = combat_with_hand(vec![Card::Regret, Card::Strike(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(77));
    }

    // --- Wound ---

    #[test]
    fn wound_card_type_is_status() {
        assert_eq!(Card::Wound.card_type(), CardType::Status);
    }

    #[test]
    fn wound_is_not_playable() {
        assert!(!Card::Wound.is_playable());
    }

    #[test]
    fn wound_id_is_wound_string() {
        assert_eq!(Card::Wound.id(), "wound");
    }

    // --- Burn ---

    #[test]
    fn burn_card_type_is_status() {
        assert_eq!(Card::Burn.card_type(), CardType::Status);
    }

    #[test]
    fn burn_is_not_playable() {
        assert!(!Card::Burn.is_playable());
    }

    #[test]
    fn burn_id_is_burn_string() {
        assert_eq!(Card::Burn.id(), "burn");
    }

    #[test]
    fn burn_in_hand_deals_2_blockable_damage_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let state = combat_with_hand(vec![Card::Burn]);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(78));
    }

    #[test]
    fn burn_in_draw_pile_does_not_deal_damage_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile.push(Card::Burn);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    // --- BurnPlus ---

    #[test]
    fn burn_plus_card_type_is_status() {
        assert_eq!(Card::BurnPlus.card_type(), CardType::Status);
    }

    #[test]
    fn burn_plus_is_not_playable() {
        assert!(!Card::BurnPlus.is_playable());
    }

    #[test]
    fn burn_plus_id_is_burn_plus_string() {
        assert_eq!(Card::BurnPlus.id(), "burn+");
    }

    #[test]
    fn burn_plus_in_hand_deals_4_blockable_damage_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let state = combat_with_hand(vec![Card::BurnPlus]);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(76));
    }

    #[test]
    fn burn_plus_in_draw_pile_does_not_deal_damage_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile.push(Card::BurnPlus);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    // --- Doubt ---

    #[test]
    fn doubt_card_type_is_curse() {
        assert_eq!(Card::Doubt.card_type(), CardType::Curse);
    }

    #[test]
    fn doubt_is_not_playable() {
        assert!(!Card::Doubt.is_playable());
    }

    #[test]
    fn doubt_id_is_doubt_string() {
        assert_eq!(Card::Doubt.id(), "doubt");
    }

    #[test]
    fn doubt_in_hand_applies_1_weak_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let state = combat_with_hand(vec![Card::Doubt]);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Weak), 1);
    }

    // --- Shame ---

    #[test]
    fn shame_card_type_is_curse() {
        assert_eq!(Card::Shame.card_type(), CardType::Curse);
    }

    #[test]
    fn shame_is_not_playable() {
        assert!(!Card::Shame.is_playable());
    }

    #[test]
    fn shame_id_is_shame_string() {
        assert_eq!(Card::Shame.id(), "shame");
    }

    #[test]
    fn shame_in_hand_applies_1_frail_at_end_of_turn() {
        use crate::combat::{combat_with_hand, apply_combat_command};
        let state = combat_with_hand(vec![Card::Shame]);
        let (state, _) = apply_combat_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Frail), 1);
    }

    // --- Parasite ---

    #[test]
    fn parasite_card_type_is_curse() {
        assert_eq!(Card::Parasite.card_type(), CardType::Curse);
    }

    #[test]
    fn parasite_is_not_playable() {
        assert!(!Card::Parasite.is_playable());
    }

    #[test]
    fn parasite_id_is_parasite_string() {
        assert_eq!(Card::Parasite.id(), "parasite");
    }

    // --- Curse of the Bell ---

    #[test]
    fn curse_of_the_bell_card_type_is_curse() {
        assert_eq!(Card::CurseOfTheBell.card_type(), CardType::Curse);
    }

    #[test]
    fn curse_of_the_bell_is_not_playable() {
        assert!(!Card::CurseOfTheBell.is_playable());
    }

    #[test]
    fn curse_of_the_bell_id_is_curse_of_the_bell_string() {
        assert_eq!(Card::CurseOfTheBell.id(), "curse_of_the_bell");
    }

    // --- Ascender's Bane ---

    #[test]
    fn ascenders_bane_card_type_is_curse() {
        assert_eq!(Card::AscendersBane.card_type(), CardType::Curse);
    }

    #[test]
    fn ascenders_bane_is_not_playable() {
        assert!(!Card::AscendersBane.is_playable());
    }

    #[test]
    fn ascenders_bane_is_ethereal() {
        assert!(Card::AscendersBane.is_ethereal());
    }

    #[test]
    fn ascenders_bane_id_is_ascenders_bane_string() {
        assert_eq!(Card::AscendersBane.id(), "ascenders_bane");
    }

    fn full_turn(state: crate::combat::CombatState) -> crate::combat::CombatState {
        let (s, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (s, _) = apply_command(s, Command::EndEnemyTurn, &mut rng()).unwrap();
        let (s, _) = apply_command(s, Command::StartPlayerTurn, &mut rng()).unwrap();
        s
    }

    // --- Barricade ---

    #[test]
    fn barricade_sets_barricade_status() {
        let state = combat_with_hand(vec![Card::Barricade(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.statuses.contains_key(&StatusEffect::Barricade));
    }

    #[test]
    fn barricade_preserves_block_across_turn() {
        use crate::enemies::Move;
        let mut state = combat_with_hand(vec![Card::Defend(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::Barricade, 1);
        state.enemies[0].move_ = Move::LouseBlock; // enemy defends, no damage to player
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(5));
        let state = full_turn(state);
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn without_barricade_block_clears_each_turn() {
        let state = combat_with_hand(vec![Card::Defend(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let state = full_turn(state);
        assert_eq!(state.player.block, Block(0));
    }

    // --- Feel No Pain ---

    #[test]
    fn feel_no_pain_sets_status_with_3_stacks() {
        let state = combat_with_hand(vec![Card::FeelNoPain(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::FeelNoPain).copied(), Some(3));
    }

    #[test]
    fn feel_no_pain_grants_block_when_card_exhausted() {
        let mut state = combat_with_hand(vec![Card::TrueGrit(Grade::Base), Card::Strike(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::FeelNoPain, 3);
        state.player.draw_pile = vec![];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(3 + 7)); // juggernaut 0 + true grit block + feel no pain
    }

    // --- Dark Embrace ---

    #[test]
    fn dark_embrace_sets_status() {
        let state = combat_with_hand(vec![Card::DarkEmbrace(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.statuses.contains_key(&StatusEffect::DarkEmbrace));
    }

    #[test]
    fn dark_embrace_draws_card_when_card_exhausted() {
        let mut state = combat_with_hand(vec![Card::SeeingRed(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::DarkEmbrace, 1);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        let hand_before = state.player.hand.len();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // SeeingRed exhausts itself; DarkEmbrace draws 1 card; also gains energy
        assert_eq!(state.player.hand.len(), hand_before); // removed SeeingRed, drew 1 via DarkEmbrace
    }

    // --- Juggernaut ---

    #[test]
    fn juggernaut_sets_status_with_5_damage() {
        let state = combat_with_hand(vec![Card::Juggernaut(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Juggernaut).copied(), Some(5));
    }

    #[test]
    fn juggernaut_damages_enemy_when_block_gained() {
        let mut state = combat_with_hand(vec![Card::Defend(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::Juggernaut, 5);
        let enemy_hp_before = state.enemies[0].hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp.0, enemy_hp_before.0 - 5);
    }

    #[test]
    fn juggernaut_plus_deals_7_damage() {
        let state = combat_with_hand(vec![Card::Juggernaut(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Juggernaut).copied(), Some(7));
    }

    // --- Rupture ---

    #[test]
    fn rupture_sets_status_with_1_stack() {
        let state = combat_with_hand(vec![Card::Rupture(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Rupture).copied(), Some(1));
    }

    #[test]
    fn rupture_grants_strength_when_losing_hp_on_turn() {
        let mut state = combat_with_hand(vec![Card::Bloodletting(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::Rupture, 1);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength).copied(), Some(1));
    }

    #[test]
    fn rupture_plus_grants_2_strength_per_hp_loss() {
        let state = combat_with_hand(vec![Card::Rupture(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Rupture).copied(), Some(2));
    }

    // --- Demon Form ---

    #[test]
    fn demon_form_sets_status_with_2_stacks() {
        let state = combat_with_hand(vec![Card::DemonForm(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::DemonForm).copied(), Some(2));
    }

    #[test]
    fn demon_form_grants_strength_at_start_of_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::DemonForm, 2);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let state = full_turn(state);
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength).copied(), Some(2));
    }

    // --- Innate ---

    #[test]
    fn brutality_plus_is_innate() {
        assert!(Card::Brutality(Grade::Plus).is_innate());
    }

    #[test]
    fn brutality_base_is_not_innate() {
        assert!(!Card::Brutality(Grade::Base).is_innate());
    }

    #[test]
    fn innate_card_starts_in_opening_hand() {
        let state = combat_with_deck(
            vec![Card::Brutality(Grade::Plus), Card::Strike(Grade::Base)],
            &mut rng(),
        );
        assert!(state.player.hand.contains(&Card::Brutality(Grade::Plus)));
    }

    #[test]
    fn innate_card_counts_toward_opening_hand_size() {
        let state = combat_with_deck(
            vec![
                Card::Brutality(Grade::Plus),
                Card::Strike(Grade::Base), Card::Strike(Grade::Base),
                Card::Strike(Grade::Base), Card::Strike(Grade::Base),
                Card::Strike(Grade::Base),
            ],
            &mut rng(),
        );
        assert_eq!(state.player.hand.len(), 5);
    }

    // --- Limit Break ---

    #[test]
    fn limit_break_doubles_strength() {
        let mut state = combat_with_hand(vec![Card::LimitBreak(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::Strength, 5);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength).copied(), Some(10));
    }

    #[test]
    fn limit_break_is_noop_with_no_strength() {
        let state = combat_with_hand(vec![Card::LimitBreak(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Strength), 0);
    }

    #[test]
    fn limit_break_base_exhausts() {
        let state = combat_with_hand(vec![Card::LimitBreak(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::LimitBreak(Grade::Base)));
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn limit_break_plus_does_not_exhaust() {
        let state = combat_with_hand(vec![Card::LimitBreak(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.is_empty());
        assert!(state.player.discard_pile.contains(&Card::LimitBreak(Grade::Plus)));
    }

    // --- Berserk ---

    #[test]
    fn berserk_base_applies_2_vulnerable_and_berserk_status() {
        let state = combat_with_hand(vec![Card::Berserk(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Vulnerable).copied(), Some(2));
        assert_eq!(state.player.statuses.get(&StatusEffect::Berserk).copied(), Some(1));
    }

    #[test]
    fn berserk_plus_applies_1_vulnerable_and_berserk_status() {
        let state = combat_with_hand(vec![Card::Berserk(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Vulnerable).copied(), Some(1));
        assert_eq!(state.player.statuses.get(&StatusEffect::Berserk).copied(), Some(1));
    }

    #[test]
    fn berserk_grants_1_energy_at_start_of_turn() {
        use crate::types::Energy;
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Berserk, 1);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let state = full_turn(state);
        assert_eq!(state.player.energy, Energy(4)); // 3 base + 1 berserk
    }

    // --- Brutality ---

    #[test]
    fn brutality_sets_brutality_status() {
        let state = combat_with_hand(vec![Card::Brutality(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Brutality).copied(), Some(1));
    }

    #[test]
    fn brutality_causes_1_hp_loss_at_start_of_turn() {
        use crate::enemies::Move;
        use crate::types::Hp;
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Brutality, 1);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        state.enemies[0].move_ = Move::LouseBlock;
        let initial_hp = state.player.hp;
        let state = full_turn(state);
        assert_eq!(state.player.hp, Hp(initial_hp.0 - 1));
    }

    #[test]
    fn brutality_draws_1_extra_card_at_start_of_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Brutality, 1);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 10];
        let state = full_turn(state);
        assert_eq!(state.player.hand.len(), 6); // 5 normal + 1 from brutality
    }

    // --- from_id round-trips ---

    #[test]
    fn berserk_id_round_trips() {
        assert_eq!(Card::from_id("berserk"),      Some(Card::Berserk(Grade::Base)));
        assert_eq!(Card::from_id("berserk-plus"), Some(Card::Berserk(Grade::Plus)));
    }

    #[test]
    fn brutality_id_round_trips() {
        assert_eq!(Card::from_id("brutality"),      Some(Card::Brutality(Grade::Base)));
        assert_eq!(Card::from_id("brutality-plus"), Some(Card::Brutality(Grade::Plus)));
    }

    #[test]
    fn limit_break_id_round_trips() {
        assert_eq!(Card::from_id("limit-break"),      Some(Card::LimitBreak(Grade::Base)));
        assert_eq!(Card::from_id("limit-break-plus"), Some(Card::LimitBreak(Grade::Plus)));
    }

    // --- Combust ---

    #[test]
    fn combust_base_adds_5_to_combust_status() {
        let state = combat_with_hand(vec![Card::Combust(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Combust).copied(), Some(5));
    }

    #[test]
    fn combust_plus_adds_7_to_combust_status() {
        let state = combat_with_hand(vec![Card::Combust(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Combust).copied(), Some(7));
    }

    #[test]
    fn combust_deals_5_damage_to_enemy_at_end_of_turn() {
        let mut state = combat_with_hand(vec![Card::Combust(Grade::Base)]);
        state.player.statuses.insert(StatusEffect::Combust, 5);
        state.player.hand.clear(); // discard manually to just test EndTurn effect
        // can't call PlayCard and EndTurn; set up statuses directly
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Combust, 5);
        let enemy_hp_before = state.enemies[0].hp;
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp.0, enemy_hp_before.0 - 5);
    }

    #[test]
    fn combust_causes_1_hp_loss_at_end_of_turn() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Combust, 5);
        let hp_before = state.player.hp;
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.player.hp.0, hp_before.0 - 1);
    }

    #[test]
    fn combust_kills_enemy_producing_victory() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Combust, 5);
        state.enemies[0].hp = Hp(1);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Victory);
    }

    #[test]
    fn combust_kills_player_producing_defeat() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Combust, 5);
        state.player.hp = Hp(1);
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::Defeat);
    }

    #[test]
    fn combust_emits_enemy_died_when_overkilling() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Combust, 5);
        state.enemies[0].hp = Hp(1);
        let (_, events) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert!(events.contains(&Event::EnemyDied));
    }

    #[test]
    fn combust_id_round_trips() {
        assert_eq!(Card::from_id("combust"),      Some(Card::Combust(Grade::Base)));
        assert_eq!(Card::from_id("combust-plus"), Some(Card::Combust(Grade::Plus)));
    }

    // --- Evolve ---

    #[test]
    fn evolve_base_sets_status_to_1() {
        let state = combat_with_hand(vec![Card::Evolve(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Evolve).copied(), Some(1));
    }

    #[test]
    fn evolve_plus_sets_status_to_2() {
        let state = combat_with_hand(vec![Card::Evolve(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Evolve).copied(), Some(2));
    }

    #[test]
    fn evolve_draws_extra_card_when_status_drawn_at_turn_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::Evolve, 1);
        // draw pile: 4 normal + 1 status (Wound drawn first via pop)
        state.player.draw_pile = vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Wound,
        ];
        state.player.discard_pile = vec![Card::Strike(Grade::Base)]; // for Evolve to draw
        let state = full_turn(state);
        assert_eq!(state.player.hand.len(), 6); // 5 normal draw + 1 from Evolve
    }

    #[test]
    fn evolve_id_round_trips() {
        assert_eq!(Card::from_id("evolve"),      Some(Card::Evolve(Grade::Base)));
        assert_eq!(Card::from_id("evolve-plus"), Some(Card::Evolve(Grade::Plus)));
    }

    // --- Fire Breathing ---

    #[test]
    fn fire_breathing_base_sets_status_to_6() {
        let state = combat_with_hand(vec![Card::FireBreathing(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::FireBreathing).copied(), Some(6));
    }

    #[test]
    fn fire_breathing_plus_sets_status_to_10() {
        let state = combat_with_hand(vec![Card::FireBreathing(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::FireBreathing).copied(), Some(10));
    }

    #[test]
    fn fire_breathing_deals_damage_when_status_drawn_at_turn_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::FireBreathing, 6);
        state.player.draw_pile = vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Wound, // status card, triggers Fire Breathing
        ];
        let enemy_hp_before = state.enemies[0].hp;
        let state = full_turn(state);
        assert_eq!(state.enemies[0].hp.0, enemy_hp_before.0 - 6);
    }

    #[test]
    fn fire_breathing_deals_damage_when_curse_drawn_at_turn_start() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::FireBreathing, 6);
        state.player.draw_pile = vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Injury, // curse card, triggers Fire Breathing
        ];
        let enemy_hp_before = state.enemies[0].hp;
        let state = full_turn(state);
        assert_eq!(state.enemies[0].hp.0, enemy_hp_before.0 - 6);
    }

    #[test]
    fn fire_breathing_deals_12_damage_when_two_triggers_drawn() {
        let mut state = combat_with_hand(vec![]);
        state.player.statuses.insert(StatusEffect::FireBreathing, 6);
        state.player.draw_pile = vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Injury, // curse (drawn second via pop)
            Card::Wound,  // status (drawn first via pop)
        ];
        let enemy_hp_before = state.enemies[0].hp;
        let state = full_turn(state);
        assert_eq!(state.enemies[0].hp.0, enemy_hp_before.0 - 12); // 6 * 2 triggers
    }

    #[test]
    fn fire_breathing_id_round_trips() {
        assert_eq!(Card::from_id("fire-breathing"),      Some(Card::FireBreathing(Grade::Base)));
        assert_eq!(Card::from_id("fire-breathing-plus"), Some(Card::FireBreathing(Grade::Plus)));
    }

    // --- Flex ---

    #[test]
    fn flex_base_applies_2_strength() {
        let state = combat_with_hand(vec![Card::Flex(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength).copied(), Some(2));
    }

    #[test]
    fn flex_base_applies_2_strength_down() {
        let state = combat_with_hand(vec![Card::Flex(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::StrengthDown).copied(), Some(2));
    }

    #[test]
    fn flex_plus_applies_4_strength() {
        let state = combat_with_hand(vec![Card::Flex(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::Strength).copied(), Some(4));
    }

    #[test]
    fn flex_plus_applies_4_strength_down() {
        let state = combat_with_hand(vec![Card::Flex(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.statuses.get(&StatusEffect::StrengthDown).copied(), Some(4));
    }

    #[test]
    fn flex_strength_expires_at_end_of_turn() {
        let state = combat_with_hand(vec![Card::Flex(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Strength), 0);
    }

    #[test]
    fn flex_id_round_trips() {
        assert_eq!(Card::from_id("flex"),      Some(Card::Flex(Grade::Base)));
        assert_eq!(Card::from_id("flex-plus"), Some(Card::Flex(Grade::Plus)));
    }

    // --- Intimidate ---

    #[test]
    fn intimidate_applies_1_weak_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Intimidate(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak).copied(), Some(1));
        assert_eq!(state.enemies[1].statuses.get(&StatusEffect::Weak).copied(), Some(1));
    }

    #[test]
    fn intimidate_plus_applies_2_weak_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Intimidate(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak).copied(), Some(2));
        assert_eq!(state.enemies[1].statuses.get(&StatusEffect::Weak).copied(), Some(2));
    }

    #[test]
    fn intimidate_exhausts() {
        let state = combat_with_hand(vec![Card::Intimidate(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Intimidate(Grade::Base)));
    }

    #[test]
    fn intimidate_id_round_trips() {
        assert_eq!(Card::from_id("intimidate"),      Some(Card::Intimidate(Grade::Base)));
        assert_eq!(Card::from_id("intimidate-plus"), Some(Card::Intimidate(Grade::Plus)));
    }

    // --- Shockwave ---

    #[test]
    fn shockwave_applies_3_weak_and_3_vulnerable_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Shockwave(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak).copied(), Some(3));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable).copied(), Some(3));
        assert_eq!(state.enemies[1].statuses.get(&StatusEffect::Weak).copied(), Some(3));
        assert_eq!(state.enemies[1].statuses.get(&StatusEffect::Vulnerable).copied(), Some(3));
    }

    #[test]
    fn shockwave_plus_applies_5_weak_and_5_vulnerable_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Shockwave(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Weak).copied(), Some(5));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable).copied(), Some(5));
    }

    #[test]
    fn shockwave_exhausts() {
        let state = combat_with_hand(vec![Card::Shockwave(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Shockwave(Grade::Base)));
    }

    #[test]
    fn shockwave_id_round_trips() {
        assert_eq!(Card::from_id("shockwave"),      Some(Card::Shockwave(Grade::Base)));
        assert_eq!(Card::from_id("shockwave-plus"), Some(Card::Shockwave(Grade::Plus)));
    }

    // --- Immolate ---

    #[test]
    fn immolate_deals_21_damage_to_all_enemies() {
        let mut state = combat_with_two_enemies(vec![Card::Immolate(Grade::Base)]);
        state.enemies[0].hp = Hp(50); state.enemies[1].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(29));
        assert_eq!(state.enemies[1].hp, Hp(29));
    }

    #[test]
    fn immolate_plus_deals_28_damage_to_all_enemies() {
        let mut state = combat_with_two_enemies(vec![Card::Immolate(Grade::Plus)]);
        state.enemies[0].hp = Hp(50); state.enemies[1].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(22));
        assert_eq!(state.enemies[1].hp, Hp(22));
    }

    #[test]
    fn immolate_adds_5_burn_to_discard() {
        let state = combat_with_hand(vec![Card::Immolate(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let burn_count = state.player.discard_pile.iter().filter(|c| **c == Card::Burn).count();
        assert_eq!(burn_count, 1);
    }

    #[test]
    fn immolate_respects_strength() {
        let mut state = combat_with_two_enemies(vec![Card::Immolate(Grade::Base)]);
        state.enemies[0].hp = Hp(50);
        state.player.statuses.insert(StatusEffect::Strength, 3);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(26)); // 50 - (21 + 3)
    }

    #[test]
    fn immolate_id_round_trips() {
        assert_eq!(Card::from_id("immolate"),      Some(Card::Immolate(Grade::Base)));
        assert_eq!(Card::from_id("immolate-plus"), Some(Card::Immolate(Grade::Plus)));
    }

    // --- Fiend Fire ---

    #[test]
    fn fiend_fire_exhausts_hand_and_deals_7_damage_per_card() {
        let state = combat_with_hand(vec![
            Card::FiendFire(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // 2 cards in hand when played → 2 × 7 = 14 damage
        assert_eq!(state.enemies[0].hp, Hp(6));
        assert!(state.player.hand.is_empty());
        assert_eq!(state.player.exhaust_pile.len(), 3); // 2 strikes + fiend fire itself
    }

    #[test]
    fn fiend_fire_plus_deals_10_damage_per_card() {
        let state = combat_with_hand(vec![
            Card::FiendFire(Grade::Plus),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // 2 × 10 = 20 damage
        assert_eq!(state.enemies[0].hp, Hp(0));
    }

    #[test]
    fn fiend_fire_with_empty_hand_deals_no_damage() {
        let state = combat_with_hand(vec![Card::FiendFire(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // hand was empty (only Fiend Fire, which was removed before apply), no damage
        assert_eq!(state.enemies[0].hp, Hp(20));
        assert_eq!(state.player.exhaust_pile, vec![Card::FiendFire(Grade::Base)]);
    }

    #[test]
    fn fiend_fire_exhausts() {
        assert!(Card::FiendFire(Grade::Base).exhausts());
        assert!(Card::FiendFire(Grade::Plus).exhausts());
    }

    #[test]
    fn fiend_fire_id_round_trips() {
        assert_eq!(Card::from_id("fiend-fire"),      Some(Card::FiendFire(Grade::Base)));
        assert_eq!(Card::from_id("fiend-fire-plus"), Some(Card::FiendFire(Grade::Plus)));
    }

    // --- Perfected Strike ---

    #[test]
    fn perfected_strike_counts_itself() {
        // Only Perfected Strike in hand (no other Strikes) → 1 Strike card (itself)
        // damage = 6 + 2*1 = 8
        let state = combat_with_hand(vec![Card::PerfectedStrike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
    }

    #[test]
    fn perfected_strike_counts_strikes_in_hand() {
        // Perfected Strike + 2 Strikes in hand → 3 Strike cards
        // damage = 6 + 2*3 = 12
        let state = combat_with_hand(vec![
            Card::PerfectedStrike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(8));
    }

    #[test]
    fn perfected_strike_counts_strikes_in_draw_and_discard_piles() {
        // 1 Strike in draw pile, 1 Strike in discard pile, Perfected Strike itself → 3
        // damage = 6 + 2*3 = 12
        let mut state = combat_with_hand(vec![Card::PerfectedStrike(Grade::Base)]);
        state.player.draw_pile.push(Card::Strike(Grade::Base));
        state.player.discard_pile.push(Card::Strike(Grade::Base));
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(8));
    }

    #[test]
    fn perfected_strike_counts_twin_and_pommel_strike() {
        // Twin Strike + Pommel Strike in hand + Perfected Strike itself → 3
        // damage = 6 + 2*3 = 12
        let state = combat_with_hand(vec![
            Card::PerfectedStrike(Grade::Base),
            Card::TwinStrike(Grade::Base),
            Card::PommelStrike(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(8));
    }

    #[test]
    fn perfected_strike_plus_gives_3_damage_per_strike() {
        // Only Perfected Strike+ itself → 1 Strike card
        // damage = 6 + 3*1 = 9
        let state = combat_with_hand(vec![Card::PerfectedStrike(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(11));
    }

    #[test]
    fn perfected_strike_id_round_trips() {
        assert_eq!(Card::from_id("perfected-strike"),      Some(Card::PerfectedStrike(Grade::Base)));
        assert_eq!(Card::from_id("perfected-strike-plus"), Some(Card::PerfectedStrike(Grade::Plus)));
    }

    // --- Reaper ---

    #[test]
    fn reaper_deals_4_damage_to_all_enemies_and_heals() {
        let mut state = combat_with_two_enemies(vec![Card::Reaper(Grade::Base)]);
        state.player.hp = Hp(60);
        state.enemies[0].hp = Hp(50);
        state.enemies[1].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(46));
        assert_eq!(state.enemies[1].hp, Hp(46));
        assert_eq!(state.player.hp, Hp(68)); // heals 4 + 4 = 8
    }

    #[test]
    fn reaper_plus_deals_5_damage_per_enemy() {
        let mut state = combat_with_two_enemies(vec![Card::Reaper(Grade::Plus)]);
        state.player.hp = Hp(60);
        state.enemies[0].hp = Hp(50);
        state.enemies[1].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(45));
        assert_eq!(state.enemies[1].hp, Hp(45));
        assert_eq!(state.player.hp, Hp(70)); // heals 5 + 5 = 10
    }

    #[test]
    fn reaper_heal_is_capped_at_max_hp() {
        let mut state = combat_with_two_enemies(vec![Card::Reaper(Grade::Base)]);
        state.player.hp = Hp(79);
        state.player.max_hp = Hp(80);
        state.enemies[0].hp = Hp(50);
        state.enemies[1].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn reaper_blocked_damage_does_not_heal() {
        let mut state = combat_with_hand(vec![Card::Reaper(Grade::Base)]);
        state.player.hp = Hp(60);
        state.enemies[0].block.0 = 10; // 4 damage fully absorbed by block
        let (state, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(60)); // no heal
        assert!(!events.iter().any(|e| matches!(e, Event::Healed { .. }))); // no Healed event
    }

    #[test]
    fn reaper_exhausts() {
        assert!(Card::Reaper(Grade::Base).exhausts());
        assert!(Card::Reaper(Grade::Plus).exhausts());
    }

    #[test]
    fn reaper_id_round_trips() {
        assert_eq!(Card::from_id("reaper"),      Some(Card::Reaper(Grade::Base)));
        assert_eq!(Card::from_id("reaper-plus"), Some(Card::Reaper(Grade::Plus)));
    }

    // --- Whirlwind ---

    #[test]
    fn whirlwind_deals_5_damage_per_hit_per_energy_spent() {
        // 3 energy → X=3 → 3 hits × 5 damage = 15 total
        let mut state = combat_with_hand(vec![Card::Whirlwind(Grade::Base)]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(35));
        assert_eq!(state.player.energy, Energy(0));
    }

    #[test]
    fn whirlwind_with_1_energy_deals_5_damage() {
        let mut state = combat_with_hand(vec![Card::Whirlwind(Grade::Base)]);
        state.player.energy = Energy(1);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(45));
    }

    #[test]
    fn whirlwind_with_0_energy_deals_no_damage() {
        let mut state = combat_with_hand(vec![Card::Whirlwind(Grade::Base)]);
        state.player.energy = Energy(0);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(50));
    }

    #[test]
    fn whirlwind_hits_all_enemies() {
        let mut state = combat_with_two_enemies(vec![Card::Whirlwind(Grade::Base)]);
        state.player.energy = Energy(1);
        state.enemies[0].hp = Hp(50);
        state.enemies[1].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(45));
        assert_eq!(state.enemies[1].hp, Hp(45));
    }

    #[test]
    fn whirlwind_plus_deals_8_damage_per_hit() {
        // 3 energy → X=3 → 3 hits × 8 = 24 total
        let mut state = combat_with_hand(vec![Card::Whirlwind(Grade::Plus)]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(26));
    }

    #[test]
    fn whirlwind_is_affordable_with_0_energy() {
        assert!(Card::Whirlwind(Grade::Base).card_cost().is_affordable(Energy(0)));
    }

    #[test]
    fn whirlwind_cost_displays_as_x() {
        assert_eq!(Card::Whirlwind(Grade::Base).card_cost().display(), "X");
    }

    #[test]
    fn whirlwind_id_round_trips() {
        assert_eq!(Card::from_id("whirlwind"),      Some(Card::Whirlwind(Grade::Base)));
        assert_eq!(Card::from_id("whirlwind-plus"), Some(Card::Whirlwind(Grade::Plus)));
    }

    // --- Feed ---

    #[test]
    fn feed_deals_10_damage() {
        let state = combat_with_hand(vec![Card::Feed(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    #[test]
    fn feed_raises_max_hp_and_current_hp_by_3_on_kill() {
        let mut state = combat_with_hand(vec![Card::Feed(Grade::Base)]);
        state.enemies[0].hp = Hp(5);
        let before_max = state.player.max_hp;
        let before_hp = state.player.hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.max_hp.0, before_max.0 + 3);
        assert_eq!(state.player.hp.0, before_hp.0 + 3);
    }

    #[test]
    fn feed_plus_raises_max_hp_and_current_hp_by_4_on_kill() {
        let mut state = combat_with_hand(vec![Card::Feed(Grade::Plus)]);
        state.enemies[0].hp = Hp(5);
        let before_max = state.player.max_hp;
        let before_hp = state.player.hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.max_hp.0, before_max.0 + 4);
        assert_eq!(state.player.hp.0, before_hp.0 + 4);
    }

    #[test]
    fn feed_does_not_raise_max_hp_if_enemy_survives() {
        let state = combat_with_hand(vec![Card::Feed(Grade::Base)]);
        let before = state.player.max_hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.max_hp, before);
    }

    #[test]
    fn feed_emits_max_hp_increased_event_on_kill() {
        let mut state = combat_with_hand(vec![Card::Feed(Grade::Base)]);
        state.enemies[0].hp = Hp(5);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::MaxHpIncreased { amount: 3 }));
    }

    #[test]
    fn feed_does_not_emit_max_hp_increased_if_no_kill() {
        let state = combat_with_hand(vec![Card::Feed(Grade::Base)]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(!events.iter().any(|e| matches!(e, Event::MaxHpIncreased { .. })));
    }

    #[test]
    fn feed_exhausts() {
        assert!(Card::Feed(Grade::Base).exhausts());
        assert!(Card::Feed(Grade::Plus).exhausts());
    }

    #[test]
    fn feed_id_round_trips() {
        assert_eq!(Card::from_id("feed"),      Some(Card::Feed(Grade::Base)));
        assert_eq!(Card::from_id("feed-plus"), Some(Card::Feed(Grade::Plus)));
    }

    // --- Power Through ---

    #[test]
    fn power_through_adds_2_wounds_to_hand() {
        let state = combat_with_hand(vec![Card::PowerThrough(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.iter().filter(|c| **c == Card::Wound).count(), 2);
    }

    #[test]
    fn power_through_gains_15_block() {
        let state = combat_with_hand(vec![Card::PowerThrough(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(15));
    }

    #[test]
    fn power_through_plus_gains_20_block() {
        let state = combat_with_hand(vec![Card::PowerThrough(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(20));
    }

    #[test]
    fn power_through_emits_status_card_added_to_hand_events() {
        let state = combat_with_hand(vec![Card::PowerThrough(Grade::Base)]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let count = events.iter().filter(|e| matches!(e, Event::StatusCardAddedToHand { card: Card::Wound })).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn power_through_id_round_trips() {
        assert_eq!(Card::from_id("power-through"),      Some(Card::PowerThrough(Grade::Base)));
        assert_eq!(Card::from_id("power-through-plus"), Some(Card::PowerThrough(Grade::Plus)));
    }

    // --- Burning Pact ---

    #[test]
    fn burning_pact_enters_choose_card_phase() {
        let state = combat_with_hand(vec![Card::BurningPact(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state.phase, CombatPhase::ChooseCard(_)));
    }

    #[test]
    fn burning_pact_choose_hand_card_exhausts_it() {
        let mut state = combat_with_hand(vec![Card::BurningPact(Grade::Base), Card::Strike(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Bash(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 1);
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Strike(Grade::Base)));
        assert_eq!(state.player.hand.len(), 2); // drew 2
    }

    #[test]
    fn burning_pact_choose_hand_card_draws_two() {
        let mut state = combat_with_hand(vec![Card::BurningPact(Grade::Base), Card::Strike(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Bash(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 2);
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn burning_pact_plus_draws_three() {
        let mut state = combat_with_hand(vec![Card::BurningPact(Grade::Plus), Card::Strike(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Bash(Grade::Base), Card::IronWave(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 3);
    }

    #[test]
    fn burning_pact_invalid_index_returns_error() {
        let state = combat_with_hand(vec![Card::BurningPact(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let result = apply_command(state, Command::ChooseHandCard(5), &mut rng());
        assert!(matches!(result, Err(CommandError::InvalidCard)));
    }

    #[test]
    fn burning_pact_id_round_trips() {
        assert_eq!(Card::from_id("burning-pact"),      Some(Card::BurningPact(Grade::Base)));
        assert_eq!(Card::from_id("burning-pact-plus"), Some(Card::BurningPact(Grade::Plus)));
    }

    // --- Warcry ---

    #[test]
    fn warcry_draws_one_card_and_enters_choose_card_phase() {
        let mut state = combat_with_hand(vec![Card::Warcry(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 1); // drew 1
        assert!(matches!(state.phase, CombatPhase::ChooseCard(_)));
    }

    #[test]
    fn warcry_plus_draws_two_cards() {
        let mut state = combat_with_hand(vec![Card::Warcry(Grade::Plus)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 2);
        assert!(matches!(state.phase, CombatPhase::ChooseCard(_)));
    }

    #[test]
    fn warcry_choose_hand_card_topdecks_it() {
        let mut state = combat_with_hand(vec![Card::Warcry(Grade::Base), Card::Bash(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // hand now has Bash + Strike (drawn). Choose Bash (index 0) to topdeck.
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.draw_pile.last(), Some(&Card::Bash(Grade::Base)));
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn warcry_exhausts() {
        let mut state = combat_with_hand(vec![Card::Warcry(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Warcry(Grade::Base)));
    }

    #[test]
    fn warcry_id_round_trips() {
        assert_eq!(Card::from_id("warcry"),      Some(Card::Warcry(Grade::Base)));
        assert_eq!(Card::from_id("warcry-plus"), Some(Card::Warcry(Grade::Plus)));
    }

    // --- Armaments ---

    #[test]
    fn armaments_gains_five_block_and_enters_choose_card_phase() {
        let state = combat_with_hand(vec![Card::Armaments(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(5));
        assert!(matches!(state.phase, CombatPhase::ChooseCard(_)));
    }

    #[test]
    fn armaments_choose_hand_card_upgrades_it() {
        let state = combat_with_hand(vec![Card::Armaments(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert!(state.player.hand.contains(&Card::Strike(Grade::Plus)));
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn armaments_plus_upgrades_all_cards_in_hand() {
        let state = combat_with_hand(vec![Card::Armaments(Grade::Plus), Card::Strike(Grade::Base), Card::Defend(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.contains(&Card::Strike(Grade::Plus)));
        assert!(state.player.hand.contains(&Card::Defend(Grade::Plus)));
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn armaments_emits_card_upgraded_event() {
        let state = combat_with_hand(vec![Card::Armaments(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (_, events) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert!(events.contains(&Event::CardUpgraded { from: Card::Strike(Grade::Base), to: Card::Strike(Grade::Plus) }));
    }

    #[test]
    fn armaments_plus_preserves_unupgradeable_cards() {
        let state = combat_with_hand(vec![
            Card::Armaments(Grade::Plus),
            Card::Strike(Grade::Base),
            Card::Wound,
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.contains(&Card::Strike(Grade::Plus)));
        assert!(state.player.hand.contains(&Card::Wound));
        assert!(!state.player.hand.contains(&Card::Strike(Grade::Base)));
    }

    #[test]
    fn armaments_id_round_trips() {
        assert_eq!(Card::from_id("armaments"),      Some(Card::Armaments(Grade::Base)));
        assert_eq!(Card::from_id("armaments-plus"), Some(Card::Armaments(Grade::Plus)));
    }

    // --- Ghostly Armor ---

    #[test]
    fn ghostly_armor_gains_13_block() {
        let state = combat_with_hand(vec![Card::GhostlyArmor(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(13));
    }

    #[test]
    fn ghostly_armor_plus_gains_16_block() {
        let state = combat_with_hand(vec![Card::GhostlyArmor(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(16));
    }

    #[test]
    fn ghostly_armor_is_ethereal() {
        assert!(Card::GhostlyArmor(Grade::Base).is_ethereal());
    }

    #[test]
    fn ghostly_armor_exhausts_at_end_of_turn_when_unplayed() {
        let mut state = combat_with_hand(vec![Card::GhostlyArmor(Grade::Base)]);
        state.player.draw_pile = vec![];
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::GhostlyArmor(Grade::Base)));
        assert!(!state.player.discard_pile.contains(&Card::GhostlyArmor(Grade::Base)));
    }

    #[test]
    fn ghostly_armor_id_round_trips() {
        assert_eq!(Card::from_id("ghostly-armor"),      Some(Card::GhostlyArmor(Grade::Base)));
        assert_eq!(Card::from_id("ghostly-armor-plus"), Some(Card::GhostlyArmor(Grade::Plus)));
    }

    // --- Second Wind ---

    #[test]
    fn second_wind_exhausts_non_attack_cards() {
        let state = combat_with_hand(vec![
            Card::SecondWind(Grade::Base),
            Card::Defend(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Wound,
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Defend(Grade::Base)));
        assert!(state.player.exhaust_pile.contains(&Card::Wound));
        assert!(!state.player.exhaust_pile.contains(&Card::Strike(Grade::Base)));
    }

    #[test]
    fn second_wind_gains_5_block_per_exhausted_card() {
        let state = combat_with_hand(vec![
            Card::SecondWind(Grade::Base),
            Card::Defend(Grade::Base),
            Card::ShrugItOff(Grade::Base),
            Card::Strike(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(10)); // 2 non-attack cards × 5
    }

    #[test]
    fn second_wind_leaves_attack_cards_in_hand() {
        let state = combat_with_hand(vec![
            Card::SecondWind(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Defend(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.contains(&Card::Strike(Grade::Base)));
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn second_wind_no_block_when_no_non_attacks() {
        let state = combat_with_hand(vec![
            Card::SecondWind(Grade::Base),
            Card::Strike(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(0));
    }

    #[test]
    fn second_wind_id_round_trips() {
        assert_eq!(Card::from_id("second-wind"),      Some(Card::SecondWind(Grade::Base)));
        assert_eq!(Card::from_id("second-wind-plus"), Some(Card::SecondWind(Grade::Plus)));
    }

    // --- All-Out Attack ---

    #[test]
    fn all_out_attack_deals_10_to_all_enemies() {
        let mut state = combat_with_two_enemies(vec![Card::AllOutAttack(Grade::Base)]);
        state.enemies[0].hp = crate::types::Hp(20);
        state.enemies[1].hp = crate::types::Hp(20);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, crate::types::Hp(10));
        assert_eq!(state.enemies[1].hp, crate::types::Hp(10));
    }

    #[test]
    fn all_out_attack_plus_deals_14_to_all_enemies() {
        let mut state = combat_with_two_enemies(vec![Card::AllOutAttack(Grade::Plus)]);
        state.enemies[0].hp = crate::types::Hp(20);
        state.enemies[1].hp = crate::types::Hp(20);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, crate::types::Hp(6));
        assert_eq!(state.enemies[1].hp, crate::types::Hp(6));
    }

    #[test]
    fn all_out_attack_discards_one_card_from_hand() {
        let state = combat_with_hand(vec![
            Card::AllOutAttack(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Defend(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Started with 2 cards after AllOutAttack removed; 1 should be discarded
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn all_out_attack_no_discard_when_hand_empty() {
        let state = combat_with_hand(vec![Card::AllOutAttack(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 0);
    }

    #[test]
    fn all_out_attack_id_round_trips() {
        assert_eq!(Card::from_id("all-out-attack"),      Some(Card::AllOutAttack(Grade::Base)));
        assert_eq!(Card::from_id("all-out-attack-plus"), Some(Card::AllOutAttack(Grade::Plus)));
    }

    // --- All for One ---

    #[test]
    fn all_for_one_deals_10_damage() {
        let mut state = combat_with_hand(vec![Card::AllForOne(Grade::Base)]);
        state.enemies[0].hp = crate::types::Hp(20);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, crate::types::Hp(10));
    }

    #[test]
    fn all_for_one_retrieves_zero_cost_cards_from_discard() {
        let mut state = combat_with_hand(vec![Card::AllForOne(Grade::Base)]);
        state.player.discard_pile = vec![
            Card::Wound,         // cost 0 (unplayable status, but cost is 0)
            Card::Strike(Grade::Base), // cost 1
            Card::SeeingRed(Grade::Base), // cost 1
        ];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.contains(&Card::Wound));
        assert!(!state.player.hand.contains(&Card::Strike(Grade::Base)));
    }

    #[test]
    fn all_for_one_plus_deals_14_damage() {
        let mut state = combat_with_hand(vec![Card::AllForOne(Grade::Plus)]);
        state.enemies[0].hp = crate::types::Hp(20);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, crate::types::Hp(6));
    }

    #[test]
    fn all_for_one_id_round_trips() {
        assert_eq!(Card::from_id("all-for-one"),      Some(Card::AllForOne(Grade::Base)));
        assert_eq!(Card::from_id("all-for-one-plus"), Some(Card::AllForOne(Grade::Plus)));
    }

    // --- Sentinel ---

    #[test]
    fn sentinel_gains_5_block_when_played() {
        let state = combat_with_hand(vec![Card::Sentinel(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(5));
    }

    #[test]
    fn sentinel_plus_gains_8_block_when_played() {
        let state = combat_with_hand(vec![Card::Sentinel(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(8));
    }

    #[test]
    fn sentinel_does_not_exhaust_when_played() {
        let state = combat_with_hand(vec![Card::Sentinel(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.is_empty());
        assert!(state.player.discard_pile.contains(&Card::Sentinel(Grade::Base)));
    }

    #[test]
    fn sentinel_grants_2_energy_when_exhausted_by_true_grit() {
        let mut state = combat_with_hand(vec![
            Card::TrueGrit(Grade::Base),
            Card::Sentinel(Grade::Base),
        ]);
        state.player.energy = Energy(3);
        let energy_before = state.player.energy;
        // TrueGrit exhausts a random card; NoOpRng picks index 0 after shuffle = first card
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Sentinel (index 0 after TrueGrit removed) exhausted → +2 energy
        assert!(state.player.exhaust_pile.contains(&Card::Sentinel(Grade::Base)));
        assert_eq!(state.player.energy.0, energy_before.0 - 1 + 2); // spent 1 for TrueGrit, gained 2
    }

    #[test]
    fn sentinel_plus_grants_3_energy_when_exhausted() {
        let mut state = combat_with_hand(vec![
            Card::Sentinel(Grade::Plus),
        ]);
        state.player.energy = Energy(3);
        // Exhaust Sentinel directly via Second Wind (non-attack cards get exhausted)
        let state2 = {
            let mut s = combat_with_hand(vec![
                Card::SecondWind(Grade::Base),
                Card::Sentinel(Grade::Plus),
            ]);
            s.player.energy = Energy(3);
            let (s, _) = apply_command(s, Command::PlayCard(0, 0), &mut rng()).unwrap();
            s
        };
        assert!(state2.player.exhaust_pile.contains(&Card::Sentinel(Grade::Plus)));
        assert_eq!(state2.player.energy.0, 2 + 3); // 3 - 1 for SecondWind + 3 for Sentinel exhaust
    }

    #[test]
    fn sentinel_emits_energy_gained_event_when_exhausted() {
        let mut state = combat_with_hand(vec![
            Card::SecondWind(Grade::Base),
            Card::Sentinel(Grade::Base),
        ]);
        state.player.energy = Energy(3);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::EnergyGained { amount: 2 })));
    }

    #[test]
    fn sentinel_id_round_trips() {
        assert_eq!(Card::from_id("sentinel"),      Some(Card::Sentinel(Grade::Base)));
        assert_eq!(Card::from_id("sentinel-plus"), Some(Card::Sentinel(Grade::Plus)));
    }

    // --- Searing Blow ---

    #[test]
    fn searing_blow_base_deals_12_damage() {
        let state = combat_with_hand(vec![Card::SearingBlow(0)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, crate::types::Hp(8)); // 20 - 12
    }

    #[test]
    fn searing_blow_upgraded_once_deals_16_damage() {
        let state = combat_with_hand(vec![Card::SearingBlow(1)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, crate::types::Hp(4)); // 20 - 16
    }

    #[test]
    fn searing_blow_upgraded_twice_deals_21_damage() {
        let mut state = combat_with_hand(vec![Card::SearingBlow(2)]);
        state.enemies[0].hp = crate::types::Hp(30);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, crate::types::Hp(9)); // 30 - 21
    }

    #[test]
    fn searing_blow_upgraded_three_times_deals_27_damage() {
        let mut state = combat_with_hand(vec![Card::SearingBlow(3)]);
        state.enemies[0].hp = crate::types::Hp(40);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, crate::types::Hp(13)); // 40 - 27
    }

    #[test]
    fn searing_blow_upgrade_always_returns_some() {
        assert_eq!(Card::SearingBlow(0).upgrade(), Some(Card::SearingBlow(1)));
        assert_eq!(Card::SearingBlow(1).upgrade(), Some(Card::SearingBlow(2)));
        assert_eq!(Card::SearingBlow(5).upgrade(), Some(Card::SearingBlow(6)));
    }

    #[test]
    fn searing_blow_base_name_is_searing_blow() {
        assert_eq!(Card::SearingBlow(0).name(), "Searing Blow");
    }

    #[test]
    fn searing_blow_upgraded_name_is_searing_blow_plus() {
        assert_eq!(Card::SearingBlow(1).name(), "Searing Blow+");
        assert_eq!(Card::SearingBlow(3).name(), "Searing Blow+");
    }

    #[test]
    fn searing_blow_description_shows_correct_damage() {
        assert!(Card::SearingBlow(0).description().contains("12"));
        assert!(Card::SearingBlow(1).description().contains("16"));
        assert!(Card::SearingBlow(2).description().contains("21"));
        assert!(Card::SearingBlow(3).description().contains("27"));
    }

    #[test]
    fn searing_blow_id_round_trips() {
        assert_eq!(Card::from_id("searing-blow"), Some(Card::SearingBlow(0)));
    }

    // --- Slimed ---

    #[test]
    fn slimed_card_type_is_status() {
        assert_eq!(Card::Slimed.card_type(), CardType::Status);
    }

    #[test]
    fn slimed_costs_1() {
        assert_eq!(Card::Slimed.card_cost(), CardCost::Fixed(Energy(1)));
    }

    #[test]
    fn slimed_is_playable() {
        assert!(Card::Slimed.is_playable());
    }

    #[test]
    fn slimed_exhausts_when_played() {
        let mut state = combat_with_hand(vec![Card::Slimed]);
        state.player.energy = crate::types::Energy(3);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Slimed));
        assert!(!state.player.discard_pile.contains(&Card::Slimed));
    }

    // --- Blind ---

    #[test]
    fn blind_applies_2_weak_to_enemy() {
        let state = combat_with_hand(vec![Card::Blind(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Weak), 2);
    }

    #[test]
    fn blind_plus_applies_2_weak_to_all_enemies() {
        let state = combat_with_hand(vec![Card::Blind(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Weak), 2);
        assert_eq!(state.enemies.len(), 1); // single-enemy combat still works
    }

    #[test]
    fn blind_costs_0_energy() {
        let mut state = combat_with_hand(vec![Card::Blind(Grade::Base)]);
        state.player.energy = Energy(0);
        apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
    }

    #[test]
    fn blind_plus_applies_weak_to_all_in_two_enemy_combat() {
        let _state = combat_with_hand(vec![Card::Blind(Grade::Plus)]);
        let (state, events) = apply_command(
            crate::combat::combat_with_two_enemies(vec![Card::Blind(Grade::Plus)]),
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Weak), 2);
        assert_eq!(get_stacks(&state.enemies[1].statuses, StatusEffect::Weak), 2);
        let _ = events;
    }

    // --- Trip ---

    #[test]
    fn trip_applies_2_vulnerable_to_all_enemies() {
        let (state, _) = apply_command(
            combat_with_two_enemies(vec![Card::Trip(Grade::Base)]),
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Vulnerable), 2);
        assert_eq!(get_stacks(&state.enemies[1].statuses, StatusEffect::Vulnerable), 2);
    }

    #[test]
    fn trip_plus_applies_2_vulnerable_to_all_enemies() {
        let (state, _) = apply_command(
            combat_with_two_enemies(vec![Card::Trip(Grade::Plus)]),
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Vulnerable), 2);
        assert_eq!(get_stacks(&state.enemies[1].statuses, StatusEffect::Vulnerable), 2);
    }

    #[test]
    fn trip_costs_0_energy() {
        let mut state = combat_with_hand(vec![Card::Trip(Grade::Base)]);
        state.player.energy = Energy(0);
        apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
    }

    // --- Dramatic Entrance ---

    #[test]
    fn dramatic_entrance_deals_8_damage_to_all_enemies() {
        let (state, _) = apply_command(
            combat_with_two_enemies(vec![Card::DramaticEntrance(Grade::Base)]),
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(12));
        assert_eq!(state.enemies[1].hp, Hp(12));
    }

    #[test]
    fn dramatic_entrance_plus_deals_12_damage_to_all_enemies() {
        let (state, _) = apply_command(
            combat_with_two_enemies(vec![Card::DramaticEntrance(Grade::Plus)]),
            Command::PlayCard(0, 0),
            &mut rng(),
        ).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(8));
        assert_eq!(state.enemies[1].hp, Hp(8));
    }

    #[test]
    fn dramatic_entrance_costs_0_energy() {
        let mut state = combat_with_hand(vec![Card::DramaticEntrance(Grade::Base)]);
        state.player.energy = Energy(0);
        apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
    }

    #[test]
    fn dramatic_entrance_exhausts() {
        let state = combat_with_hand(vec![Card::DramaticEntrance(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::DramaticEntrance(Grade::Base)));
        assert!(!state.player.discard_pile.contains(&Card::DramaticEntrance(Grade::Base)));
    }

    #[test]
    fn dramatic_entrance_is_innate() {
        assert!(Card::DramaticEntrance(Grade::Base).is_innate());
        assert!(Card::DramaticEntrance(Grade::Plus).is_innate());
    }

    // --- Pain ---

    #[test]
    fn pain_is_not_playable() {
        assert!(!Card::Pain.is_playable());
    }

    #[test]
    fn pain_is_a_curse() {
        assert_eq!(Card::Pain.card_type(), CardType::Curse);
    }

    #[test]
    fn pain_id_round_trips() {
        assert_eq!(Card::from_id("pain"), Some(Card::Pain));
    }

    #[test]
    fn pain_deals_1_hp_when_card_played() {
        let state = combat_with_hand(vec![Card::Strike(Grade::Base), Card::Pain]);
        let hp_before = state.player.hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(hp_before.0 - 1));
    }

    #[test]
    fn pain_triggers_once_per_card_played() {
        let mut state = combat_with_hand(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Pain,
        ]);
        state.player.energy = Energy(9);
        let hp_before = state.player.hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(hp_before.0 - 2));
    }

    // --- Normality ---

    #[test]
    fn normality_is_not_playable() {
        assert!(!Card::Normality.is_playable());
    }

    #[test]
    fn normality_is_a_curse() {
        assert_eq!(Card::Normality.card_type(), CardType::Curse);
    }

    #[test]
    fn normality_id_round_trips() {
        assert_eq!(Card::from_id("normality"), Some(Card::Normality));
    }

    #[test]
    fn normality_allows_playing_3_cards() {
        let mut state = combat_with_hand(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Normality,
        ]);
        state.player.energy = Energy(9);
        apply_command(state.clone(), Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
    }

    #[test]
    fn normality_blocks_4th_card() {
        let mut state = combat_with_hand(vec![
            Card::Strike(Grade::Base), Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Normality,
        ]);
        state.player.energy = Energy(9);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let result = apply_command(state, Command::PlayCard(0, 0), &mut rng());
        assert_eq!(result, Err(CommandError::Normality));
    }

    // --- Void ---

    #[test]
    fn void_is_not_playable() {
        assert!(!Card::Void.is_playable());
    }

    #[test]
    fn void_is_ethereal() {
        assert!(Card::Void.is_ethereal());
    }

    #[test]
    fn void_is_a_status() {
        assert_eq!(Card::Void.card_type(), CardType::Status);
    }

    #[test]
    fn void_id_round_trips() {
        assert_eq!(Card::from_id("void"), Some(Card::Void));
    }

    #[test]
    fn drawing_void_costs_1_energy() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Void];
        let state = full_turn(state);
        assert_eq!(state.player.energy, Energy(state.player.max_energy.0 - 1));
    }

    #[test]
    fn drawing_void_does_not_go_below_zero_energy() {
        let mut state = combat_with_hand(vec![]);
        state.player.draw_pile = vec![Card::Void];
        state.player.max_energy = Energy(0);
        let state = full_turn(state);
        assert_eq!(state.player.energy, Energy(0));
    }

    // --- Writhe ---

    #[test]
    fn writhe_is_not_playable() {
        assert!(!Card::Writhe.is_playable());
    }

    #[test]
    fn writhe_is_innate() {
        assert!(Card::Writhe.is_innate());
    }

    #[test]
    fn writhe_is_a_curse() {
        assert_eq!(Card::Writhe.card_type(), CardType::Curse);
    }

    // --- Neutral card id round-trips ---

    #[test]
    fn flash_of_steel_id_round_trips() {
        assert_eq!(Card::from_id("flash-of-steel"),      Some(Card::FlashOfSteel(Grade::Base)));
        assert_eq!(Card::from_id("flash-of-steel-plus"), Some(Card::FlashOfSteel(Grade::Plus)));
    }

    #[test]
    fn finesse_id_round_trips() {
        assert_eq!(Card::from_id("finesse"),      Some(Card::Finesse(Grade::Base)));
        assert_eq!(Card::from_id("finesse-plus"), Some(Card::Finesse(Grade::Plus)));
    }

    #[test]
    fn good_instincts_id_round_trips() {
        assert_eq!(Card::from_id("good-instincts"),      Some(Card::GoodInstincts(Grade::Base)));
        assert_eq!(Card::from_id("good-instincts-plus"), Some(Card::GoodInstincts(Grade::Plus)));
    }

    #[test]
    fn swift_strike_id_round_trips() {
        assert_eq!(Card::from_id("swift-strike"),      Some(Card::SwiftStrike(Grade::Base)));
        assert_eq!(Card::from_id("swift-strike-plus"), Some(Card::SwiftStrike(Grade::Plus)));
    }

    #[test]
    fn blind_id_round_trips() {
        assert_eq!(Card::from_id("blind"),      Some(Card::Blind(Grade::Base)));
        assert_eq!(Card::from_id("blind-plus"), Some(Card::Blind(Grade::Plus)));
    }

    #[test]
    fn dramatic_entrance_id_round_trips() {
        assert_eq!(Card::from_id("dramatic-entrance"),      Some(Card::DramaticEntrance(Grade::Base)));
        assert_eq!(Card::from_id("dramatic-entrance-plus"), Some(Card::DramaticEntrance(Grade::Plus)));
    }

    #[test]
    fn trip_id_round_trips() {
        assert_eq!(Card::from_id("trip"),      Some(Card::Trip(Grade::Base)));
        assert_eq!(Card::from_id("trip-plus"), Some(Card::Trip(Grade::Plus)));
    }

    #[test]
    fn writhe_id_round_trips() {
        assert_eq!(Card::from_id("writhe"), Some(Card::Writhe));
    }

    // --- Flash of Steel ---

    #[test]
    fn flash_of_steel_deals_3_damage() {
        let state = combat_with_hand(vec![Card::FlashOfSteel(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(17));
    }

    #[test]
    fn flash_of_steel_draws_1_card() {
        let mut state = combat_with_hand(vec![Card::FlashOfSteel(Grade::Base)]);
        state.player.draw_pile.push(Card::Strike(Grade::Base));
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn flash_of_steel_costs_0_energy() {
        let mut state = combat_with_hand(vec![Card::FlashOfSteel(Grade::Base)]);
        state.player.energy = Energy(0);
        apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
    }

    #[test]
    fn flash_of_steel_plus_deals_6_damage() {
        let state = combat_with_hand(vec![Card::FlashOfSteel(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(14));
    }

    // --- Finesse ---

    #[test]
    fn finesse_gains_2_block() {
        let state = combat_with_hand(vec![Card::Finesse(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(2));
    }

    #[test]
    fn finesse_draws_1_card() {
        let mut state = combat_with_hand(vec![Card::Finesse(Grade::Base)]);
        state.player.draw_pile.push(Card::Strike(Grade::Base));
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn finesse_costs_0_energy() {
        let mut state = combat_with_hand(vec![Card::Finesse(Grade::Base)]);
        state.player.energy = Energy(0);
        apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
    }

    #[test]
    fn finesse_plus_gains_4_block() {
        let state = combat_with_hand(vec![Card::Finesse(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(4));
    }

    // --- Good Instincts ---

    #[test]
    fn good_instincts_gains_6_block() {
        let state = combat_with_hand(vec![Card::GoodInstincts(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(6));
    }

    #[test]
    fn good_instincts_costs_0_energy() {
        let mut state = combat_with_hand(vec![Card::GoodInstincts(Grade::Base)]);
        state.player.energy = Energy(0);
        apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
    }

    #[test]
    fn good_instincts_plus_gains_9_block() {
        let state = combat_with_hand(vec![Card::GoodInstincts(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(9));
    }

    // --- Swift Strike ---

    #[test]
    fn swift_strike_deals_7_damage() {
        let state = combat_with_hand(vec![Card::SwiftStrike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(13));
    }

    #[test]
    fn swift_strike_costs_0_energy() {
        let mut state = combat_with_hand(vec![Card::SwiftStrike(Grade::Base)]);
        state.player.energy = Energy(0);
        apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
    }

    #[test]
    fn swift_strike_plus_deals_10_damage() {
        let state = combat_with_hand(vec![Card::SwiftStrike(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(10));
    }

    // --- Bandage Up ---

    #[test]
    fn bandage_up_heals_4_hp() {
        let mut state = combat_with_hand(vec![Card::BandageUp(Grade::Base)]);
        state.player.hp = Hp(70);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(74));
    }

    #[test]
    fn bandage_up_does_not_exceed_max_hp() {
        let mut state = combat_with_hand(vec![Card::BandageUp(Grade::Base)]);
        state.player.hp = Hp(78);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn bandage_up_plus_heals_6_hp() {
        let mut state = combat_with_hand(vec![Card::BandageUp(Grade::Plus)]);
        state.player.hp = Hp(70);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(76));
    }

    #[test]
    fn bandage_up_exhausts() {
        let state = combat_with_hand(vec![Card::BandageUp(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::BandageUp(Grade::Base)));
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn bandage_up_costs_0_energy() {
        assert_eq!(Card::BandageUp(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn bandage_up_emits_healed_event() {
        let mut state = combat_with_hand(vec![Card::BandageUp(Grade::Base)]);
        state.player.hp = Hp(70);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::Healed { amount: 4 }));
    }

    #[test]
    fn bandage_up_id_round_trips() {
        assert_eq!(Card::from_id("bandage-up"),      Some(Card::BandageUp(Grade::Base)));
        assert_eq!(Card::from_id("bandage-up-plus"), Some(Card::BandageUp(Grade::Plus)));
    }

    // --- Dark Shackles ---

    #[test]
    fn dark_shackles_reduces_enemy_strength_by_9() {
        let mut state = combat_with_hand(vec![Card::DarkShackles(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::Strength, 10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Strength), 1);
    }

    #[test]
    fn dark_shackles_strength_restored_at_end_of_enemy_turn() {
        let mut state = combat_with_hand(vec![Card::DarkShackles(Grade::Base)]);
        state.enemies[0].statuses.insert(StatusEffect::Strength, 10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndTurn, &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::EndEnemyTurn, &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Strength), 10);
    }

    #[test]
    fn dark_shackles_plus_reduces_enemy_strength_by_15() {
        let mut state = combat_with_hand(vec![Card::DarkShackles(Grade::Plus)]);
        state.enemies[0].statuses.insert(StatusEffect::Strength, 20);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.enemies[0].statuses, StatusEffect::Strength), 5);
    }

    #[test]
    fn dark_shackles_exhausts() {
        let state = combat_with_hand(vec![Card::DarkShackles(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::DarkShackles(Grade::Base)));
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn dark_shackles_costs_0_energy() {
        assert_eq!(Card::DarkShackles(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn dark_shackles_id_round_trips() {
        assert_eq!(Card::from_id("dark-shackles"),      Some(Card::DarkShackles(Grade::Base)));
        assert_eq!(Card::from_id("dark-shackles-plus"), Some(Card::DarkShackles(Grade::Plus)));
    }

    // --- Violence ---

    #[test]
    fn violence_moves_up_to_3_attacks_from_draw_to_hand() {
        let mut state = combat_with_hand(vec![Card::Violence(Grade::Base)]);
        state.player.draw_pile = vec![
            Card::Defend(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
        ];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let strikes_in_hand = state.player.hand.iter().filter(|c| **c == Card::Strike(Grade::Base)).count();
        assert_eq!(strikes_in_hand, 3);
    }

    #[test]
    fn violence_does_not_move_non_attacks() {
        let mut state = combat_with_hand(vec![Card::Violence(Grade::Base)]);
        state.player.draw_pile = vec![
            Card::Defend(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
        ];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.draw_pile.contains(&Card::Defend(Grade::Base)));
    }

    #[test]
    fn violence_moves_fewer_if_not_enough_attacks() {
        let mut state = combat_with_hand(vec![Card::Violence(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let strikes_in_hand = state.player.hand.iter().filter(|c| **c == Card::Strike(Grade::Base)).count();
        assert_eq!(strikes_in_hand, 2);
        assert!(state.player.draw_pile.is_empty());
    }

    #[test]
    fn violence_plus_moves_up_to_5_attacks() {
        let mut state = combat_with_hand(vec![Card::Violence(Grade::Plus)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 6];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let strikes_in_hand = state.player.hand.iter().filter(|c| **c == Card::Strike(Grade::Base)).count();
        assert_eq!(strikes_in_hand, 5);
    }

    #[test]
    fn violence_exhausts() {
        let mut state = combat_with_hand(vec![Card::Violence(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::Violence(Grade::Base)));
        assert!(state.player.discard_pile.is_empty());
    }

    #[test]
    fn violence_costs_0_energy() {
        assert_eq!(Card::Violence(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn violence_id_round_trips() {
        assert_eq!(Card::from_id("violence"),      Some(Card::Violence(Grade::Base)));
        assert_eq!(Card::from_id("violence-plus"), Some(Card::Violence(Grade::Plus)));
    }

    // --- Secret Weapon ---

    #[test]
    fn secret_weapon_moves_attack_from_draw_to_hand() {
        let mut state = combat_with_hand(vec![Card::SecretWeapon(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.contains(&Card::Strike(Grade::Base)));
    }

    #[test]
    fn secret_weapon_does_not_move_skills() {
        let mut state = combat_with_hand(vec![Card::SecretWeapon(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.draw_pile.contains(&Card::Defend(Grade::Base)));
    }

    #[test]
    fn secret_weapon_does_nothing_if_no_attacks_in_draw() {
        let mut state = combat_with_hand(vec![Card::SecretWeapon(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.is_empty());
        assert_eq!(state.player.draw_pile.len(), 1);
    }

    #[test]
    fn secret_weapon_exhausts() {
        let mut state = combat_with_hand(vec![Card::SecretWeapon(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::SecretWeapon(Grade::Base)));
    }

    #[test]
    fn secret_weapon_plus_does_not_exhaust() {
        let mut state = combat_with_hand(vec![Card::SecretWeapon(Grade::Plus)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.iter().all(|c| !matches!(c, Card::SecretWeapon(_))));
        assert!(state.player.discard_pile.contains(&Card::SecretWeapon(Grade::Plus)));
    }

    #[test]
    fn secret_weapon_costs_0_energy() {
        assert_eq!(Card::SecretWeapon(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn secret_weapon_id_round_trips() {
        assert_eq!(Card::from_id("secret-weapon"),      Some(Card::SecretWeapon(Grade::Base)));
        assert_eq!(Card::from_id("secret-weapon-plus"), Some(Card::SecretWeapon(Grade::Plus)));
    }

    // --- Secret Technique ---

    #[test]
    fn secret_technique_moves_skill_from_draw_to_hand() {
        let mut state = combat_with_hand(vec![Card::SecretTechnique(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.contains(&Card::Defend(Grade::Base)));
    }

    #[test]
    fn secret_technique_does_not_move_attacks() {
        let mut state = combat_with_hand(vec![Card::SecretTechnique(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.draw_pile.contains(&Card::Strike(Grade::Base)));
    }

    #[test]
    fn secret_technique_does_nothing_if_no_skills_in_draw() {
        let mut state = combat_with_hand(vec![Card::SecretTechnique(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.is_empty());
        assert_eq!(state.player.draw_pile.len(), 1);
    }

    #[test]
    fn secret_technique_exhausts() {
        let mut state = combat_with_hand(vec![Card::SecretTechnique(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::SecretTechnique(Grade::Base)));
    }

    #[test]
    fn secret_technique_plus_does_not_exhaust() {
        let mut state = combat_with_hand(vec![Card::SecretTechnique(Grade::Plus)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.iter().all(|c| !matches!(c, Card::SecretTechnique(_))));
        assert!(state.player.discard_pile.contains(&Card::SecretTechnique(Grade::Plus)));
    }

    #[test]
    fn secret_technique_costs_0_energy() {
        assert_eq!(Card::SecretTechnique(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn secret_technique_id_round_trips() {
        assert_eq!(Card::from_id("secret-technique"),      Some(Card::SecretTechnique(Grade::Base)));
        assert_eq!(Card::from_id("secret-technique-plus"), Some(Card::SecretTechnique(Grade::Plus)));
    }

    // --- Forethought ---

    #[test]
    fn forethought_enters_choose_card_phase() {
        let state = combat_with_hand(vec![Card::Forethought(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state.phase, CombatPhase::ChooseCard(_)));
    }

    #[test]
    fn forethought_puts_chosen_card_at_bottom_of_draw_pile() {
        let mut state = combat_with_hand(vec![Card::Forethought(Grade::Base), Card::Strike(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.draw_pile[0], Card::Strike(Grade::Base));
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn forethought_removes_card_from_hand() {
        let state = combat_with_hand(vec![Card::Forethought(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert!(state.player.hand.is_empty());
    }

    #[test]
    fn forethought_plus_is_innate() {
        assert!(Card::Forethought(Grade::Plus).is_innate());
    }

    #[test]
    fn forethought_base_is_not_innate() {
        assert!(!Card::Forethought(Grade::Base).is_innate());
    }

    #[test]
    fn forethought_costs_0() {
        assert_eq!(Card::Forethought(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn forethought_id_round_trips() {
        assert_eq!(Card::from_id("forethought"),      Some(Card::Forethought(Grade::Base)));
        assert_eq!(Card::from_id("forethought-plus"), Some(Card::Forethought(Grade::Plus)));
    }

    // --- Thinking Ahead ---

    #[test]
    fn thinking_ahead_draws_2_and_enters_choose_phase() {
        let mut state = combat_with_hand(vec![Card::ThinkingAhead(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 2);
        assert!(matches!(state.phase, CombatPhase::ChooseCard(_)));
    }

    #[test]
    fn thinking_ahead_puts_chosen_card_on_top_of_draw_pile() {
        let mut state = combat_with_hand(vec![Card::ThinkingAhead(Grade::Base), Card::Bash(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // hand now: Bash + 2 drawn Defends. Choose Bash (index 0).
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert_eq!(state.player.draw_pile.last(), Some(&Card::Bash(Grade::Base)));
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn thinking_ahead_exhausts() {
        let mut state = combat_with_hand(vec![Card::ThinkingAhead(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.contains(&Card::ThinkingAhead(Grade::Base)));
    }

    #[test]
    fn thinking_ahead_plus_does_not_exhaust() {
        let mut state = combat_with_hand(vec![Card::ThinkingAhead(Grade::Plus)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base), Card::Strike(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert!(state.player.exhaust_pile.iter().all(|c| !matches!(c, Card::ThinkingAhead(_))));
        assert!(state.player.discard_pile.contains(&Card::ThinkingAhead(Grade::Plus)));
    }

    #[test]
    fn thinking_ahead_costs_0() {
        assert_eq!(Card::ThinkingAhead(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn thinking_ahead_id_round_trips() {
        assert_eq!(Card::from_id("thinking-ahead"),      Some(Card::ThinkingAhead(Grade::Base)));
        assert_eq!(Card::from_id("thinking-ahead-plus"), Some(Card::ThinkingAhead(Grade::Plus)));
    }

    // --- Mind Blast ---

    #[test]
    fn mind_blast_deals_damage_equal_to_draw_pile_size() {
        let mut state = combat_with_hand(vec![Card::MindBlast(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(15));
    }

    #[test]
    fn mind_blast_with_empty_draw_pile_deals_no_damage() {
        let state = combat_with_hand(vec![Card::MindBlast(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(20));
    }

    #[test]
    fn mind_blast_respects_strength() {
        let mut state = combat_with_hand(vec![Card::MindBlast(Grade::Base)]);
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
        state.player.statuses.insert(StatusEffect::Strength, 2);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(13)); // 20 - (5 + 2)
    }

    #[test]
    fn mind_blast_is_innate() {
        assert!(Card::MindBlast(Grade::Base).is_innate());
        assert!(Card::MindBlast(Grade::Plus).is_innate());
    }

    #[test]
    fn mind_blast_plus_costs_1() {
        assert_eq!(Card::MindBlast(Grade::Plus).energy_cost(), Energy(1));
    }

    #[test]
    fn mind_blast_costs_2() {
        assert_eq!(Card::MindBlast(Grade::Base).energy_cost(), Energy(2));
    }

    #[test]
    fn mind_blast_id_round_trips() {
        assert_eq!(Card::from_id("mind-blast"),      Some(Card::MindBlast(Grade::Base)));
        assert_eq!(Card::from_id("mind-blast-plus"), Some(Card::MindBlast(Grade::Plus)));
    }

    // --- Impatience ---

    #[test]
    fn impatience_draws_2_if_no_attacks_in_hand() {
        let mut state = combat_with_hand(vec![Card::Impatience(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 2);
    }

    #[test]
    fn impatience_does_not_draw_if_attack_in_hand() {
        let mut state = combat_with_hand(vec![Card::Impatience(Grade::Base), Card::Strike(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Only Strike remains in hand (Impatience discarded, no draw)
        assert_eq!(state.player.hand.len(), 1);
        assert!(!state.player.hand.contains(&Card::Defend(Grade::Base)));
    }

    #[test]
    fn impatience_plus_draws_3_if_no_attacks_in_hand() {
        let mut state = combat_with_hand(vec![Card::Impatience(Grade::Plus)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base); 3];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 3);
    }

    #[test]
    fn impatience_costs_0() {
        assert_eq!(Card::Impatience(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn impatience_id_round_trips() {
        assert_eq!(Card::from_id("impatience"),      Some(Card::Impatience(Grade::Base)));
        assert_eq!(Card::from_id("impatience-plus"), Some(Card::Impatience(Grade::Plus)));
    }

    // --- Master of Strategy ---

    #[test]
    fn master_of_strategy_draws_3_cards() {
        let mut state = combat_with_hand(vec![Card::MasterOfStrategy(Grade::Base)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base); 3];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 3);
    }

    #[test]
    fn master_of_strategy_plus_draws_4_cards() {
        let mut state = combat_with_hand(vec![Card::MasterOfStrategy(Grade::Plus)]);
        state.player.draw_pile = vec![Card::Defend(Grade::Base); 4];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 4);
    }

    #[test]
    fn master_of_strategy_exhausts() {
        assert!(Card::MasterOfStrategy(Grade::Base).exhausts());
        assert!(Card::MasterOfStrategy(Grade::Plus).exhausts());
    }

    #[test]
    fn master_of_strategy_costs_0() {
        assert_eq!(Card::MasterOfStrategy(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn master_of_strategy_id_round_trips() {
        assert_eq!(Card::from_id("master-of-strategy"),      Some(Card::MasterOfStrategy(Grade::Base)));
        assert_eq!(Card::from_id("master-of-strategy-plus"), Some(Card::MasterOfStrategy(Grade::Plus)));
    }

    // --- Deep Breath ---

    #[test]
    fn deep_breath_shuffles_discard_into_draw_pile() {
        let mut state = combat_with_hand(vec![Card::DeepBreath(Grade::Base)]);
        state.player.draw_pile = vec![];
        state.player.discard_pile = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Deep Breath itself goes to discard after play; the 2 original discard cards move to draw+hand
        assert!(!state.player.discard_pile.contains(&Card::Strike(Grade::Base)));
        assert!(!state.player.discard_pile.contains(&Card::Defend(Grade::Base)));
        assert_eq!(state.player.draw_pile.len() + state.player.hand.len(), 2);
    }

    #[test]
    fn deep_breath_draws_1_after_shuffle() {
        let mut state = combat_with_hand(vec![Card::DeepBreath(Grade::Base)]);
        state.player.draw_pile = vec![];
        state.player.discard_pile = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn deep_breath_plus_draws_2_after_shuffle() {
        let mut state = combat_with_hand(vec![Card::DeepBreath(Grade::Plus)]);
        state.player.draw_pile = vec![];
        state.player.discard_pile = vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 2);
    }

    #[test]
    fn deep_breath_costs_0() {
        assert_eq!(Card::DeepBreath(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn deep_breath_id_round_trips() {
        assert_eq!(Card::from_id("deep-breath"),      Some(Card::DeepBreath(Grade::Base)));
        assert_eq!(Card::from_id("deep-breath-plus"), Some(Card::DeepBreath(Grade::Plus)));
    }

    // --- Apotheosis ---

    #[test]
    fn apotheosis_upgrades_all_cards_in_hand() {
        let state = combat_with_hand(vec![Card::Apotheosis(Grade::Base), Card::Strike(Grade::Base), Card::Defend(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.hand.iter().all(|c| matches!(c, Card::Strike(Grade::Plus) | Card::Defend(Grade::Plus))));
    }

    #[test]
    fn apotheosis_upgrades_all_cards_in_discard_pile() {
        let mut state = combat_with_hand(vec![Card::Apotheosis(Grade::Base)]);
        state.player.discard_pile = vec![Card::Strike(Grade::Base), Card::Bash(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.discard_pile.iter().all(|c| matches!(c, Card::Strike(Grade::Plus) | Card::Bash(Grade::Plus))));
    }

    #[test]
    fn apotheosis_exhausts() {
        assert!(Card::Apotheosis(Grade::Base).exhausts());
        assert!(Card::Apotheosis(Grade::Plus).exhausts());
    }

    #[test]
    fn apotheosis_costs_2() {
        assert_eq!(Card::Apotheosis(Grade::Base).energy_cost(), Energy(2));
    }

    #[test]
    fn apotheosis_id_round_trips() {
        assert_eq!(Card::from_id("apotheosis"),      Some(Card::Apotheosis(Grade::Base)));
        assert_eq!(Card::from_id("apotheosis-plus"), Some(Card::Apotheosis(Grade::Plus)));
    }

    // --- Enlightenment ---

    #[test]
    fn enlightenment_reduces_hand_card_costs_to_1_this_turn() {
        let state = combat_with_hand(vec![Card::Enlightenment(Grade::Base), Card::Bash(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Bash normally costs 2; after Enlightenment it should cost 1
        // With 3 energy and Enlightenment (0 cost) already played, playing Bash for 1 should leave 2 energy
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(2));
    }

    #[test]
    fn enlightenment_cost_cap_expires_at_start_of_next_turn() {
        let mut state = combat_with_hand(vec![Card::Enlightenment(Grade::Base)]);
        state.player.discard_pile = vec![Card::Bash(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Full turn cycle
        let state = full_turn(state);
        // Bash is now in hand (drawn at start of turn); its cost should be back to 2
        // With 3 energy, playing Bash (cost 2) should leave 1 energy
        let bash_idx = state.player.hand.iter().position(|c| matches!(c, Card::Bash(_))).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(bash_idx, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(1));
    }

    #[test]
    fn enlightenment_plus_cost_cap_persists_across_turns() {
        let mut state = combat_with_hand(vec![Card::Enlightenment(Grade::Plus)]);
        state.player.discard_pile = vec![Card::Bash(Grade::Base)];
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Full turn cycle
        let state = full_turn(state);
        // Bash cost is still capped at 1; playing it leaves 2 energy
        let bash_idx = state.player.hand.iter().position(|c| matches!(c, Card::Bash(_))).unwrap();
        let (state, _) = apply_command(state, Command::PlayCard(bash_idx, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, Energy(2));
    }

    #[test]
    fn enlightenment_costs_0() {
        assert_eq!(Card::Enlightenment(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn enlightenment_id_round_trips() {
        assert_eq!(Card::from_id("enlightenment"),      Some(Card::Enlightenment(Grade::Base)));
        assert_eq!(Card::from_id("enlightenment-plus"), Some(Card::Enlightenment(Grade::Plus)));
    }

    // --- Purity ---

    #[test]
    fn purity_prompts_to_choose_card_from_hand() {
        let mut state = combat_with_hand(vec![Card::Purity(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state.phase, CombatPhase::ChooseCard(crate::combat::ChooseCardContext::Purity { remaining: 3 })));
    }

    #[test]
    fn purity_exhaust_chosen_card_and_returns_to_player_turn_when_done() {
        let mut state = combat_with_hand(vec![Card::Purity(Grade::Base), Card::Strike(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Choose the one remaining card (Strike)
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
        assert!(state.player.exhaust_pile.contains(&Card::Strike(Grade::Base)));
        assert!(state.player.hand.is_empty());
    }

    #[test]
    fn purity_allows_up_to_3_exhaust_choices() {
        let mut state = combat_with_hand(vec![
            Card::Purity(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // remaining: 3 → choose
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert!(matches!(state.phase, CombatPhase::ChooseCard(crate::combat::ChooseCardContext::Purity { remaining: 2 })));
        // remaining: 2 → choose
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert!(matches!(state.phase, CombatPhase::ChooseCard(crate::combat::ChooseCardContext::Purity { remaining: 1 })));
        // remaining: 1 → last choice returns to PlayerTurn
        let (state, _) = apply_command(state, Command::ChooseHandCard(0), &mut rng()).unwrap();
        assert_eq!(state.phase, CombatPhase::PlayerTurn);
        // 3 chosen cards + Purity itself = 4 exhausted
        assert_eq!(state.player.exhaust_pile.len(), 4);
        assert_eq!(state.player.hand.len(), 1); // 4 remained, exhausted 3
    }

    #[test]
    fn purity_plus_allows_up_to_5_exhaust_choices() {
        let mut state = combat_with_hand(vec![
            Card::Purity(Grade::Plus),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base), Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
        ]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(matches!(state.phase, CombatPhase::ChooseCard(crate::combat::ChooseCardContext::Purity { remaining: 5 })));
    }

    #[test]
    fn purity_exhausts() {
        assert!(Card::Purity(Grade::Base).exhausts());
        assert!(Card::Purity(Grade::Plus).exhausts());
    }

    #[test]
    fn purity_costs_0() {
        assert_eq!(Card::Purity(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn purity_id_round_trips() {
        assert_eq!(Card::from_id("purity"),      Some(Card::Purity(Grade::Base)));
        assert_eq!(Card::from_id("purity-plus"), Some(Card::Purity(Grade::Plus)));
    }

    // --- Hand of Greed ---

    #[test]
    fn hand_of_greed_deals_20_damage() {
        let state = combat_with_hand(vec![Card::HandOfGreed(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(0));
    }

    #[test]
    fn hand_of_greed_gains_20_gold_on_kill() {
        let state = combat_with_hand(vec![Card::HandOfGreed(Grade::Base)]);
        let initial_gold = state.player.gold;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.gold, initial_gold + 20);
    }

    #[test]
    fn hand_of_greed_no_gold_if_enemy_survives() {
        let mut state = combat_with_hand(vec![Card::HandOfGreed(Grade::Base)]);
        state.enemies[0].hp = Hp(30); // 20 damage won't kill
        let initial_gold = state.player.gold;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.gold, initial_gold);
    }

    #[test]
    fn hand_of_greed_plus_deals_25_damage() {
        let mut state = combat_with_hand(vec![Card::HandOfGreed(Grade::Plus)]);
        state.enemies[0].hp = Hp(30);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(5));
    }

    #[test]
    fn hand_of_greed_costs_2() {
        assert_eq!(Card::HandOfGreed(Grade::Base).energy_cost(), Energy(2));
    }

    #[test]
    fn hand_of_greed_id_round_trips() {
        assert_eq!(Card::from_id("hand-of-greed"),      Some(Card::HandOfGreed(Grade::Base)));
        assert_eq!(Card::from_id("hand-of-greed-plus"), Some(Card::HandOfGreed(Grade::Plus)));
    }

    // --- Jack of All Trades ---

    #[test]
    fn jack_of_all_trades_adds_one_colorless_card_to_hand() {
        let state = combat_with_hand(vec![Card::JackOfAllTrades(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 1);
    }

    #[test]
    fn jack_of_all_trades_card_is_from_colorless_pool() {
        let state = combat_with_hand(vec![Card::JackOfAllTrades(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let card = &state.player.hand[0];
        assert!(crate::cards::colorless_reward_pool().contains(card));
    }

    #[test]
    fn jack_of_all_trades_exhausts() {
        assert!(Card::JackOfAllTrades(Grade::Base).exhausts());
        assert!(Card::JackOfAllTrades(Grade::Plus).exhausts());
    }

    #[test]
    fn jack_of_all_trades_costs_0() {
        assert_eq!(Card::JackOfAllTrades(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn jack_of_all_trades_id_round_trips() {
        assert_eq!(Card::from_id("jack-of-all-trades"),      Some(Card::JackOfAllTrades(Grade::Base)));
        assert_eq!(Card::from_id("jack-of-all-trades-plus"), Some(Card::JackOfAllTrades(Grade::Plus)));
    }

    // --- Panache ---

    #[test]
    fn panache_deals_10_aoe_damage_on_5th_card_played() {
        let mut state = combat_with_hand(vec![
            Card::Panache(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
            Card::Strike(Grade::Base),
        ]);
        state.player.energy = crate::types::Energy(10);
        state.enemies[0].hp = crate::types::Hp(100);
        state.enemies[0].max_hp = crate::types::Hp(100);
        // Play Panache (1st card), then 4 Strikes (total 5)
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Panache
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Strike 1
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Strike 2
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Strike 3
        // 5th card: enemy HP should drop by 10 from Panache after the strike damage
        let hp_before = state.enemies[0].hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Strike 4 = 5th card
        assert!(state.enemies[0].hp.0 < hp_before.0 - 6); // at least strike (6) + panache (10)
    }

    #[test]
    fn panache_plus_deals_14_aoe_on_5th_card() {
        use crate::status::get_stacks;
        let mut state = combat_with_hand(vec![Card::Panache(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Panache), 14);
    }

    #[test]
    fn panache_does_not_fire_before_5_cards() {
        let enemy_start_hp = 20;
        let mut state = combat_with_hand(vec![
            Card::Panache(Grade::Base),
            Card::Defend(Grade::Base),
            Card::Defend(Grade::Base),
            Card::Defend(Grade::Base),
        ]);
        state.player.energy = crate::types::Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Panache
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Defend 1
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Defend 2
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Defend 3 = 4th card
        assert_eq!(state.enemies[0].hp.0, enemy_start_hp); // no damage yet
    }

    #[test]
    fn panache_is_a_power() {
        assert_eq!(Card::Panache(Grade::Base).card_type(), CardType::Power);
    }

    #[test]
    fn panache_costs_0() {
        assert_eq!(Card::Panache(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn panache_id_round_trips() {
        assert_eq!(Card::from_id("panache"),      Some(Card::Panache(Grade::Base)));
        assert_eq!(Card::from_id("panache-plus"), Some(Card::Panache(Grade::Plus)));
    }

    // --- Panacea ---

    #[test]
    fn panacea_costs_0() {
        assert_eq!(Card::Panacea(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn panacea_is_a_skill() {
        assert_eq!(Card::Panacea(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn panacea_exhausts() {
        assert!(Card::Panacea(Grade::Base).exhausts());
    }

    #[test]
    fn panacea_grants_2_artifact_to_player() {
        let state = combat_with_hand(vec![Card::Panacea(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Artifact), 2);
    }

    #[test]
    fn panacea_plus_grants_3_artifact() {
        let state = combat_with_hand(vec![Card::Panacea(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Artifact), 3);
    }

    #[test]
    fn artifact_blocks_debuff_applied_to_player() {
        use crate::combat::{apply_status, Target};
        let mut state = combat_with_hand(vec![Card::Panacea(Grade::Base)]);
        let (mut state, mut events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Artifact = 2; apply a debuff to the player — it should be blocked
        let applied = apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Vulnerable, 2, &mut events);
        assert!(!applied);
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Vulnerable), 0);
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Artifact), 1);
    }

    #[test]
    fn artifact_does_not_block_buffs() {
        use crate::combat::{apply_status, Target};
        let mut state = combat_with_hand(vec![Card::Panacea(Grade::Base)]);
        let (mut state, mut events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let applied = apply_status(&mut state.player.statuses, Target::Player, StatusEffect::Strength, 2, &mut events);
        assert!(applied);
        assert_eq!(get_stacks(&state.player.statuses, StatusEffect::Strength), 2);
    }

    #[test]
    fn panacea_id_round_trips() {
        assert_eq!(Card::from_id("panacea"),      Some(Card::Panacea(Grade::Base)));
        assert_eq!(Card::from_id("panacea-plus"), Some(Card::Panacea(Grade::Plus)));
    }

    // --- Sadistic Nature ---

    #[test]
    fn sadistic_nature_costs_0() {
        assert_eq!(Card::SadisticNature(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn sadistic_nature_is_a_power() {
        assert_eq!(Card::SadisticNature(Grade::Base).card_type(), CardType::Power);
    }

    #[test]
    fn sadistic_nature_deals_5_damage_when_enemy_gets_debuff() {
        let mut state = combat_with_hand(vec![Card::SadisticNature(Grade::Base), Card::Bash(Grade::Base)]);
        state.player.energy = Energy(10);
        state.enemies[0].hp = Hp(100);
        state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // SadisticNature
        let hp_before = state.enemies[0].hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // Bash: 8 dmg + 2 Vulnerable
        // Bash deals 8 damage, then Sadistic Nature fires for Vulnerable debuff: +5
        assert_eq!(state.enemies[0].hp.0, hp_before.0 - 8 - 5);
    }

    #[test]
    fn sadistic_nature_plus_deals_7_damage_per_debuff() {
        let mut state = combat_with_hand(vec![Card::SadisticNature(Grade::Plus), Card::Bash(Grade::Base)]);
        state.player.energy = Energy(10);
        state.enemies[0].hp = Hp(100);
        state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let hp_before = state.enemies[0].hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp.0, hp_before.0 - 8 - 7);
    }

    #[test]
    fn sadistic_nature_fires_per_debuff_not_per_card() {
        // Uppercut applies Weak + Vulnerable = 2 debuffs = 10 Sadistic Nature damage
        let mut state = combat_with_hand(vec![Card::SadisticNature(Grade::Base), Card::Uppercut(Grade::Base)]);
        state.player.energy = Energy(10);
        state.enemies[0].hp = Hp(100);
        state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let hp_before = state.enemies[0].hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Uppercut: 13 damage + 2 Sadistic Nature procs (Weak + Vulnerable) = 13 + 5 + 5 = 23
        assert_eq!(state.enemies[0].hp.0, hp_before.0 - 13 - 10);
    }

    #[test]
    fn sadistic_nature_does_not_fire_for_strength_reduction() {
        // Disarm applies -2 Strength which is not a debuff in our classification
        let mut state = combat_with_hand(vec![Card::SadisticNature(Grade::Base), Card::Disarm]);
        state.player.energy = Energy(10);
        state.enemies[0].hp = Hp(100);
        state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let hp_before = state.enemies[0].hp;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Disarm: no damage, Strength -2, no Sadistic Nature proc
        assert_eq!(state.enemies[0].hp.0, hp_before.0);
    }

    #[test]
    fn sadistic_nature_id_round_trips() {
        assert_eq!(Card::from_id("sadistic-nature"),      Some(Card::SadisticNature(Grade::Base)));
        assert_eq!(Card::from_id("sadistic-nature-plus"), Some(Card::SadisticNature(Grade::Plus)));
    }

    // --- Panic Button ---

    #[test]
    fn panic_button_costs_0() {
        assert_eq!(Card::PanicButton(Grade::Base).energy_cost(), Energy(0));
    }

    #[test]
    fn panic_button_is_a_skill() {
        assert_eq!(Card::PanicButton(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn panic_button_exhausts() {
        assert!(Card::PanicButton(Grade::Base).exhausts());
    }

    #[test]
    fn panic_button_grants_30_block() {
        let state = combat_with_hand(vec![Card::PanicButton(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(30));
    }

    #[test]
    fn panic_button_prevents_block_gain_next_turn() {
        let mut state = combat_with_hand(vec![Card::PanicButton(Grade::Base), Card::Defend(Grade::Base)]);
        state.player.energy = Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // PanicButton
        let state = full_turn(state); // new turn starts, lock active
        // playing Defend should gain 0 block (locked)
        let block_before = state.player.block;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, block_before); // no change
    }

    #[test]
    fn panic_button_lock_expires_after_2_turns() {
        let mut state = combat_with_hand(vec![Card::PanicButton(Grade::Base)]);
        state.player.energy = Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let state = full_turn(state); // turn 2: locked
        let mut state = full_turn(state); // turn 3: lock expires
        state.player.hand.push(Card::Defend(Grade::Base));
        let block_before = state.player.block;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(state.player.block > block_before); // block gained again
    }

    #[test]
    fn panic_button_plus_grants_30_block() {
        let state = combat_with_hand(vec![Card::PanicButton(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(30));
    }

    #[test]
    fn panic_button_id_round_trips() {
        assert_eq!(Card::from_id("panic-button"),      Some(Card::PanicButton(Grade::Base)));
        assert_eq!(Card::from_id("panic-button-plus"), Some(Card::PanicButton(Grade::Plus)));
    }

    // --- The Bomb ---

    #[test]
    fn the_bomb_costs_2() {
        assert_eq!(Card::TheBomb(Grade::Base).energy_cost(), Energy(2));
    }

    #[test]
    fn the_bomb_is_a_skill() {
        assert_eq!(Card::TheBomb(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn the_bomb_deals_40_damage_after_3_turns() {
        let mut state = combat_with_hand(vec![Card::TheBomb(Grade::Base)]);
        state.player.energy = Energy(10);
        state.enemies[0].hp = Hp(100);
        state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let hp_after_play = state.enemies[0].hp;
        let state = full_turn(state); // turn 2
        assert_eq!(state.enemies[0].hp, hp_after_play); // no damage yet
        let state = full_turn(state); // turn 3
        assert_eq!(state.enemies[0].hp, hp_after_play); // still no damage
        let state = full_turn(state); // turn 4: bomb fires at end of turn 3
        assert_eq!(state.enemies[0].hp.0, hp_after_play.0 - 40);
    }

    #[test]
    fn the_bomb_plus_deals_50_damage_after_3_turns() {
        let mut state = combat_with_hand(vec![Card::TheBomb(Grade::Plus)]);
        state.player.energy = Energy(10);
        state.enemies[0].hp = Hp(100);
        state.enemies[0].max_hp = Hp(100);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        let hp_after_play = state.enemies[0].hp;
        let state = full_turn(state);
        let state = full_turn(state);
        let state = full_turn(state);
        assert_eq!(state.enemies[0].hp.0, hp_after_play.0 - 50);
    }

    #[test]
    fn the_bomb_id_round_trips() {
        assert_eq!(Card::from_id("the-bomb"),      Some(Card::TheBomb(Grade::Base)));
        assert_eq!(Card::from_id("the-bomb-plus"), Some(Card::TheBomb(Grade::Plus)));
    }

    // --- Madness ---

    #[test]
    fn madness_costs_1() {
        assert_eq!(Card::Madness(Grade::Base).energy_cost(), Energy(1));
    }

    #[test]
    fn madness_is_a_skill() {
        assert_eq!(Card::Madness(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn madness_exhausts() {
        assert!(Card::Madness(Grade::Base).exhausts());
    }

    #[test]
    fn madness_makes_card_cost_0_this_combat() {
        // Hand: [Madness, Strike]. NoOpRng picks index 0 (Strike after Madness is removed).
        // After playing Madness the hand has [Strike], which should now cost 0.
        let mut state = combat_with_hand(vec![Card::Madness(Grade::Base), Card::Strike(Grade::Base)]);
        state.player.energy = Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        // Strike is now in zero_cost_cards — playing it should cost 0, so energy only drops by strike's natural cost (0 because zero cost)
        let energy_before = state.player.energy;
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, energy_before); // no energy spent on Strike
    }

    #[test]
    fn madness_zero_cost_persists_after_reshuffle() {
        // The zero-cost effect should survive into the next turn
        let mut state = combat_with_hand(vec![Card::Madness(Grade::Base), Card::Strike(Grade::Base)]);
        state.player.energy = Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap(); // play Madness
        // End turn and start new turn
        let state = full_turn(state);
        // Strike should be back in hand (draw pile has only Strike after Madness exhausted)
        let strike_idx = state.player.hand.iter().position(|c| matches!(c, Card::Strike(_))).expect("Strike in hand");
        let energy_before = state.player.energy;
        let (state, _) = apply_command(state, Command::PlayCard(strike_idx, 0), &mut rng()).unwrap();
        assert_eq!(state.player.energy, energy_before); // still free
    }

    #[test]
    fn madness_id_round_trips() {
        assert_eq!(Card::from_id("madness"),      Some(Card::Madness(Grade::Base)));
        assert_eq!(Card::from_id("madness-plus"), Some(Card::Madness(Grade::Plus)));
    }

    // --- Transmutation ---

    #[test]
    fn transmutation_costs_x() {
        assert_eq!(Card::Transmutation(Grade::Base).card_cost(), CardCost::X);
    }

    #[test]
    fn transmutation_is_a_skill() {
        assert_eq!(Card::Transmutation(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn transmutation_exhausts() {
        assert!(Card::Transmutation(Grade::Base).exhausts());
    }

    #[test]
    fn transmutation_base_creates_x_colorless_cards() {
        // With energy=3, X=3, hand should gain 3 colorless cards
        let mut state = combat_with_hand(vec![Card::Transmutation(Grade::Base)]);
        state.player.energy = Energy(3);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 3);
        for card in &state.player.hand {
            assert!(crate::cards::colorless_reward_pool().contains(card), "card {:?} not in colorless pool", card);
        }
    }

    #[test]
    fn transmutation_plus_creates_x_zero_cost_colorless_cards() {
        let mut state = combat_with_hand(vec![Card::Transmutation(Grade::Plus)]);
        state.player.energy = Energy(2);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 2);
        // All created cards should cost 0 (be in zero_cost_cards)
        for card in &state.player.hand {
            assert!(state.zero_cost_cards.contains(card), "card {:?} not in zero_cost_cards", card);
        }
    }

    #[test]
    fn transmutation_x0_creates_no_cards() {
        // With energy=0, X=0, no cards should be created
        let mut state = combat_with_hand(vec![Card::Transmutation(Grade::Base)]);
        state.player.energy = Energy(0);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 0);
    }

    #[test]
    fn transmutation_id_round_trips() {
        assert_eq!(Card::from_id("transmutation"),      Some(Card::Transmutation(Grade::Base)));
        assert_eq!(Card::from_id("transmutation-plus"), Some(Card::Transmutation(Grade::Plus)));
    }

    // --- Chrysalis ---

    #[test]
    fn chrysalis_costs_2() {
        assert_eq!(Card::Chrysalis(Grade::Base).energy_cost(), Energy(2));
    }

    #[test]
    fn chrysalis_is_a_skill() {
        assert_eq!(Card::Chrysalis(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn chrysalis_exhausts() {
        assert!(Card::Chrysalis(Grade::Base).exhausts());
    }

    #[test]
    fn chrysalis_adds_3_skills_to_hand() {
        let mut state = combat_with_hand(vec![Card::Chrysalis(Grade::Base)]);
        state.player.energy = Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 3);
        for card in &state.player.hand {
            assert_eq!(card.card_type(), CardType::Skill, "expected Skill, got {:?}", card);
        }
    }

    #[test]
    fn chrysalis_cards_cost_0() {
        let mut state = combat_with_hand(vec![Card::Chrysalis(Grade::Base)]);
        state.player.energy = Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        for card in &state.player.hand {
            assert!(state.zero_cost_cards.contains(card), "card {:?} not in zero_cost_cards", card);
        }
    }

    #[test]
    fn chrysalis_id_round_trips() {
        assert_eq!(Card::from_id("chrysalis"),      Some(Card::Chrysalis(Grade::Base)));
        assert_eq!(Card::from_id("chrysalis-plus"), Some(Card::Chrysalis(Grade::Plus)));
    }

    // --- Metamorphosis ---

    #[test]
    fn metamorphosis_costs_2() {
        assert_eq!(Card::Metamorphosis(Grade::Base).energy_cost(), Energy(2));
    }

    #[test]
    fn metamorphosis_is_a_skill() {
        assert_eq!(Card::Metamorphosis(Grade::Base).card_type(), CardType::Skill);
    }

    #[test]
    fn metamorphosis_exhausts() {
        assert!(Card::Metamorphosis(Grade::Base).exhausts());
    }

    #[test]
    fn metamorphosis_adds_3_attacks_to_hand() {
        let mut state = combat_with_hand(vec![Card::Metamorphosis(Grade::Base)]);
        state.player.energy = Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hand.len(), 3);
        for card in &state.player.hand {
            assert_eq!(card.card_type(), CardType::Attack, "expected Attack, got {:?}", card);
        }
    }

    #[test]
    fn metamorphosis_cards_cost_0() {
        let mut state = combat_with_hand(vec![Card::Metamorphosis(Grade::Base)]);
        state.player.energy = Energy(10);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        for card in &state.player.hand {
            assert!(state.zero_cost_cards.contains(card), "card {:?} not in zero_cost_cards", card);
        }
    }

    #[test]
    fn metamorphosis_id_round_trips() {
        assert_eq!(Card::from_id("metamorphosis"),      Some(Card::Metamorphosis(Grade::Base)));
        assert_eq!(Card::from_id("metamorphosis-plus"), Some(Card::Metamorphosis(Grade::Plus)));
    }
