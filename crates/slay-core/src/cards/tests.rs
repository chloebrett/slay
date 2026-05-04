    use super::*;
    use crate::combat::{combat_with_hand, combat_with_two_enemies, apply_combat_command, CombatPhase, Event, Target};
    use crate::run::{Command, CommandError};
    use crate::status::StatusEffect;
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

    // --- Tremble ---

    #[test]
    fn tremble_applies_3_vulnerable_to_enemy() {
        let state = combat_with_hand(vec![Card::Tremble(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&3));
    }

    #[test]
    fn tremble_plus_applies_4_vulnerable() {
        let state = combat_with_hand(vec![Card::Tremble(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&4));
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

    // --- Not Yet ---

    #[test]
    fn not_yet_heals_10_hp() {
        let mut state = combat_with_hand(vec![Card::NotYet(Grade::Base)]);
        state.player.hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(60));
    }

    #[test]
    fn not_yet_cannot_overheal() {
        let state = combat_with_hand(vec![Card::NotYet(Grade::Base)]); // already at 80/80
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(80));
    }

    #[test]
    fn not_yet_plus_heals_13_hp() {
        let mut state = combat_with_hand(vec![Card::NotYet(Grade::Plus)]);
        state.player.hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(63));
    }

    // --- Mangle ---

    #[test]
    fn mangle_deals_15_damage() {
        let state = combat_with_hand(vec![Card::Mangle(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(5));
    }

    #[test]
    fn mangle_reduces_enemy_strength_by_10() {
        let state = combat_with_hand(vec![Card::Mangle(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Strength), Some(&-10));
    }

    #[test]
    fn mangle_plus_deals_20_damage() {
        let mut state = combat_with_hand(vec![Card::Mangle(Grade::Plus)]);
        state.enemies[0].hp = Hp(50);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(30));
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

    // --- Taunt ---

    #[test]
    fn taunt_grants_7_block_and_applies_1_vulnerable() {
        let state = combat_with_hand(vec![Card::Taunt(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(7));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&1));
    }

    #[test]
    fn taunt_plus_grants_8_block_and_2_vulnerable() {
        let state = combat_with_hand(vec![Card::Taunt(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(8));
        assert_eq!(state.enemies[0].statuses.get(&StatusEffect::Vulnerable), Some(&2));
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

    // --- Breakthrough ---

    #[test]
    fn breakthrough_deals_9_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Breakthrough(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(11));
        assert_eq!(state.enemies[1].hp, Hp(11));
    }

    #[test]
    fn breakthrough_costs_1_hp() {
        let state = combat_with_hand(vec![Card::Breakthrough(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(79));
    }

    #[test]
    fn breakthrough_emits_player_self_damaged_event() {
        let state = combat_with_hand(vec![Card::Breakthrough(Grade::Base)]);
        let (_, events) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert!(events.contains(&Event::PlayerSelfDamaged { amount: 1 }));
    }

    #[test]
    fn breakthrough_plus_deals_13_damage_to_all_enemies() {
        let state = combat_with_two_enemies(vec![Card::Breakthrough(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.enemies[0].hp, Hp(7));
        assert_eq!(state.enemies[1].hp, Hp(7));
    }

    // --- Blood Wall ---

    #[test]
    fn blood_wall_grants_16_block() {
        let state = combat_with_hand(vec![Card::BloodWall(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(16));
    }

    #[test]
    fn blood_wall_costs_2_hp() {
        let state = combat_with_hand(vec![Card::BloodWall(Grade::Base)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.hp, Hp(78));
    }

    #[test]
    fn blood_wall_plus_grants_20_block() {
        let state = combat_with_hand(vec![Card::BloodWall(Grade::Plus)]);
        let (state, _) = apply_command(state, Command::PlayCard(0, 0), &mut rng()).unwrap();
        assert_eq!(state.player.block, Block(20));
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
