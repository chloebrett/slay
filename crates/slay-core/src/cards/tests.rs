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
        let mut state = combat_with_hand(vec![Card::Defend(Grade::Base)]);
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
        assert_eq!(burn_count, 5);
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
