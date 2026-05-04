use slay_core::{
    generate_map, new_run, starter_deck, AnyRng, Block, Card, CombatPhase, CombatState, Enemy,
    EnemyKind, Energy, GameState, Grade, Hp, Intent, Move, NoOpRng, Player, Relic, RestSiteState, StatusMap,
};

struct TestHarness {
    state: GameState,
    rng: AnyRng,
    debug: bool,
}

impl TestHarness {
    fn with_state(state: GameState) -> Self {
        Self { state, rng: AnyRng::NoOp(NoOpRng), debug: false }
    }

    fn debug(mut self) -> Self {
        self.debug = true;
        self
    }

    fn with_hand(hand: Vec<Card>) -> Self {
        let state = GameState::Combat {
            state: CombatState {
                player: Player {
                    hp: Hp(80),
                    max_hp: Hp(80),
                    block: Block(0),
                    energy: Energy(3),
                    max_energy: Energy(3),
                    hand,
                    draw_pile: Vec::new(),
                    discard_pile: Vec::new(),
                    exhaust_pile: Vec::new(),
                    statuses: StatusMap::new(),
                    deck: starter_deck(),
                    gold: 0,
                    relics: vec![],
                    potions: vec![],
                },
                enemies: vec![Enemy {
                    kind: EnemyKind::Louse,
                    hp: Hp(20),
                    max_hp: Hp(20),
                    block: Block(0),
                    move_: Move::LouseBite,
                    last_move: None,
                    statuses: StatusMap::new(),
                }],
                turn: 1,
                phase: CombatPhase::PlayerTurn,
                attacks_this_turn: 0,
                skills_this_turn: 0,
                attacks_this_combat: 0,
                skills_this_combat: 0,
                cards_played_this_turn: 0,
                extra_draws_next_turn: 0,
            },
            floor: 0,
            is_boss: false,
            graph: slay_core::generate_map(&mut AnyRng::NoOp(NoOpRng)),
            next_floor_cols: vec![0, 1],
            scenario: slay_core::Scenario::Main,
        };
        Self { state, rng: AnyRng::NoOp(NoOpRng), debug: false }
    }

    fn send(&mut self, input: &str) -> Result<(), String> {
        let command = slay_tui::command::parse(input, &self.state, self.debug)
            .ok_or_else(|| format!("unknown command: '{input}'"))?;
        let (new_state, _) = slay_tui::engine::apply_and_drain(self.state.clone(), command, &mut self.rng)
            .map_err(|e| format!("{e:?}"))?;
        self.state = new_state;
        Ok(())
    }

    fn player_hp(&self) -> i32 {
        match &self.state {
            GameState::Combat { state, .. } => state.player.hp.0,
            _ => panic!("not in combat: {:?}", std::mem::discriminant(&self.state)),
        }
    }

    fn enemy_hp(&self) -> i32 {
        match &self.state {
            GameState::Combat { state, .. } => state.enemies[0].hp.0,
            _ => panic!("not in combat"),
        }
    }

    fn is_victory(&self) -> bool {
        // Victory means we transitioned out of combat — either CardReward or GameOver
        matches!(
            &self.state,
            GameState::CardReward(_) | GameState::GameOver { victory: true }
        )
    }
}

#[test]
fn play_strike_reduces_enemy_hp() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base)]);
    game.send("play 1").unwrap();
    assert_eq!(game.enemy_hp(), 14);
}

#[test]
fn play_defend_then_end_reduces_damage_taken() {
    let mut game = TestHarness::with_hand(vec![Card::Defend(Grade::Base)]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Defend(Grade::Base); 5];
    }
    game.send("play 1").unwrap();
    game.send("end").unwrap(); // turn 1 end → enemy attacks 8, block 5 absorbs → 3 dmg
    assert_eq!(game.player_hp(), 77); // 80 - (8 - 5)
}

#[test]
fn unknown_command_returns_error_without_crashing() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base)]);
    let result = game.send("fireball");
    assert!(result.is_err());
    assert_eq!(game.player_hp(), 80);
}

#[test]
fn player_wins_by_playing_strikes_until_enemy_dead() {
    // Enemy 20 HP, Strike 6 dmg, player 3 energy/turn → 3 strikes per turn.
    // Turn 1: 3 strikes → enemy 2 HP. Player takes 8 from Attack intent → 72 HP.
    // Turn 2: 1 strike kills.
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base); 5]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 10];
    }

    game.send("play 1").unwrap(); // enemy hp 14
    game.send("play 1").unwrap(); // enemy hp 8
    game.send("play 1").unwrap(); // enemy hp 2
    game.send("end").unwrap();    // enemy attacks 8 → player 72
    assert_eq!(game.player_hp(), 72);

    game.send("play 1").unwrap(); // enemy hp 0 → leaves combat
    assert!(game.is_victory());
}

#[test]
fn play_zero_is_invalid() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base)]);
    let result = game.send("play 0");
    assert!(result.is_err());
}

#[test]
fn enemy_alternates_attack_and_defend_intents() {
    let mut game = TestHarness::with_hand(Vec::new());
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 10];
    }
    let intent_attack = matches!(
        &game.state,
        GameState::Combat { state: cs, .. } if cs.enemies[0].move_.intent() == Intent::Attack(8)
    );
    assert!(intent_attack);

    game.send("end").unwrap();
    let intent_defend = matches!(
        &game.state,
        GameState::Combat { state: cs, .. } if cs.enemies[0].move_.intent() == Intent::Defend(5)
    );
    assert!(intent_defend);

    game.send("end").unwrap();
    let intent_attack2 = matches!(
        &game.state,
        GameState::Combat { state: cs, .. } if cs.enemies[0].move_.intent() == Intent::Attack(8)
    );
    assert!(intent_attack2);
}

#[test]
fn enemy_block_from_defend_absorbs_player_attack() {
    let mut game = TestHarness::with_hand(Vec::new());
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 10];
    }
    game.send("end").unwrap(); // turn 2: intent Defend
    game.send("end").unwrap(); // enemy defends; now turn 3, enemy has 5 block

    let enemy_block = match &game.state {
        GameState::Combat { state, .. } => state.enemies[0].block,
        _ => panic!("not in combat"),
    };
    assert_eq!(enemy_block, slay_core::Block(5));

    game.send("play 1").unwrap(); // Strike: 5 absorbed, 1 to HP
    assert_eq!(game.enemy_hp(), 19);

    let enemy_block2 = match &game.state {
        GameState::Combat { state, .. } => state.enemies[0].block,
        _ => panic!("not in combat"),
    };
    assert_eq!(enemy_block2, slay_core::Block(0));
}

// --- Card rewards ---

#[test]
fn choosing_card_reward_adds_card_to_deck() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base)]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.enemies[0].hp = Hp(1);
        state.player.deck = vec![Card::Strike(Grade::Base); 5];
    }
    game.send("play 1").unwrap(); // kills enemy → CardReward
    assert!(matches!(game.state, GameState::CardReward(_)));
    game.send("1").unwrap(); // pick first option → Map
    let GameState::Map(map) = &game.state else { panic!("expected Map") };
    assert_eq!(map.player.deck.len(), 6);
}

#[test]
fn skipping_reward_leaves_deck_unchanged() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base)]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.enemies[0].hp = Hp(1);
        state.player.deck = vec![Card::Strike(Grade::Base); 5];
    }
    game.send("play 1").unwrap(); // kills enemy → CardReward
    game.send("skip").unwrap(); // skip → Map
    let GameState::Map(map) = &game.state else { panic!("expected Map") };
    assert_eq!(map.player.deck.len(), 5);
}

// --- Rest site ---

#[test]
fn rest_heals_player_hp() {
    let state = GameState::RestSite(RestSiteState {
        player: Player {
            hp: Hp(50),
            max_hp: Hp(80),
            block: Block(0),
            energy: Energy(3),
            max_energy: Energy(3),
            hand: Vec::new(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            exhaust_pile: Vec::new(),
            statuses: StatusMap::new(),
            deck: Vec::new(),
            gold: 0,
            relics: vec![],
            potions: vec![],
        },
        floor: 3,
        graph: generate_map(&mut AnyRng::NoOp(NoOpRng)),
        available_cols: vec![0, 1],
    });
    let mut game = TestHarness::with_state(state);
    game.send("rest").unwrap();
    let GameState::Map(map) = &game.state else { panic!("expected Map") };
    assert_eq!(map.player.hp.0, 74); // 50 + 30% of 80 (24)
}

// --- Full run ---

fn set_instant_win(state: &mut GameState) {
    if let GameState::Combat { state: cs, .. } = state {
        for enemy in &mut cs.enemies {
            enemy.hp = Hp(1);
        }
        let count = cs.enemies.len();
        cs.player.hand = vec![Card::Strike(Grade::Base); count];
        cs.player.energy = Energy(count as i32);
    }
}

#[test]
fn full_run_reaches_victory() {
    let mut game = TestHarness::with_state(new_run(&mut NoOpRng));

    // segment 1: floors 0-2 (3 combats)
    for _ in 0..3 {
        game.send("").unwrap(); // enter combat
        set_instant_win(&mut game.state);
        game.send("play 1").unwrap(); // kill → CardReward
        game.send("skip").unwrap(); // skip → Map
    }

    game.send("").unwrap(); // floor 3: enter shop
    game.send("leave").unwrap(); // leave → Map floor 4

    // segment 2: floors 4-5 (2 combats)
    for _ in 0..2 {
        game.send("").unwrap(); // enter combat
        set_instant_win(&mut game.state);
        game.send("play 1").unwrap(); // kill → CardReward
        game.send("skip").unwrap(); // skip → Map
    }

    game.send("").unwrap(); // floor 6: enter rest site
    game.send("rest").unwrap(); // rest → Map floor 7

    // segment 3: floors 7-8 (2 combats)
    for _ in 0..2 {
        game.send("").unwrap(); // enter combat
        set_instant_win(&mut game.state);
        game.send("play 1").unwrap(); // kill → CardReward
        game.send("skip").unwrap(); // skip → Map
    }

    game.send("").unwrap(); // floor 9: enter boss (2 enemies)
    set_instant_win(&mut game.state);
    game.send("play 1").unwrap(); // kill enemy 1
    game.send("play 1").unwrap(); // auto-target enemy 2 → GameOver

    assert!(matches!(game.state, GameState::GameOver { victory: true }));
}

// --- Status effect combos ---

#[test]
fn bash_then_strike_benefits_from_vulnerable() {
    // Bash: 8 dmg + 2 Vuln → enemy 12 HP. Strike: 6 * 3/2 = 9 dmg → enemy 3 HP.
    let mut game = TestHarness::with_hand(vec![Card::Bash(Grade::Base), Card::Strike(Grade::Base)]);
    game.send("play 1").unwrap(); // Bash — Strike shifts to slot 1
    game.send("play 1").unwrap(); // Strike with Vulnerable
    assert_eq!(game.enemy_hp(), 3);
}

#[test]
fn clothesline_reduces_enemy_attack_damage() {
    // Clothesline: 12 dmg + 2 Weak → enemy 8 HP. End turn: enemy attacks 8 * 3/4 = 6.
    let mut game = TestHarness::with_hand(vec![Card::Clothesline(Grade::Base)]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
    }
    game.send("play 1").unwrap();
    game.send("end").unwrap();
    assert_eq!(game.player_hp(), 74); // 80 - 6
}

#[test]
fn deadly_poison_drains_enemy_hp_over_multiple_turns() {
    let mut game = TestHarness::with_hand(vec![Card::DeadlyPoison(Grade::Base)]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 10];
    }
    game.send("play 1").unwrap(); // apply 5 poison — no immediate damage
    assert_eq!(game.enemy_hp(), 20);
    game.send("end").unwrap(); // poison tick: 5 dmg → enemy 15
    assert_eq!(game.enemy_hp(), 15);
    game.send("end").unwrap(); // poison tick: 4 dmg → enemy 11
    assert_eq!(game.enemy_hp(), 11);
}

#[test]
fn disarm_reduces_enemy_attack_damage() {
    // Disarm: enemy loses 2 Strength. Enemy base attack 8, so 8 + (-2) = 6.
    let mut game = TestHarness::with_hand(vec![Card::Disarm]);
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
    }
    game.send("play 1").unwrap();
    game.send("end").unwrap();
    assert_eq!(game.player_hp(), 74); // 80 - 6
}

// --- Energy management ---

#[test]
fn playing_card_without_energy_returns_error() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base); 4]);
    game.send("play 1").unwrap(); // energy 2
    game.send("play 1").unwrap(); // energy 1
    game.send("play 1").unwrap(); // energy 0
    let result = game.send("play 1"); // no energy left
    assert!(result.is_err());
    assert_eq!(game.player_hp(), 80); // state unchanged
}

// --- Exhaust pile ---

#[test]
fn disarm_goes_to_exhaust_pile_after_play() {
    let mut game = TestHarness::with_hand(vec![Card::Disarm]);
    game.send("play 1").unwrap();
    let GameState::Combat { state, .. } = &game.state else { panic!("not in combat") };
    assert!(state.player.discard_pile.is_empty());
    assert_eq!(state.player.exhaust_pile, vec![Card::Disarm]);
}

// --- Rest site: upgrade ---

#[test]
fn upgrade_at_rest_site_replaces_card_in_deck() {
    use slay_core::RestSiteState;
    let player = Player {
        hp: Hp(80), max_hp: Hp(80), block: Block(0),
        energy: Energy(3), max_energy: Energy(3),
        hand: Vec::new(), draw_pile: Vec::new(),
        discard_pile: Vec::new(), exhaust_pile: Vec::new(),
        statuses: StatusMap::new(),
        deck: vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base)],
        gold: 0,
        relics: vec![],
        potions: vec![],
    };
    let state = GameState::RestSite(RestSiteState { player, floor: 3, graph: generate_map(&mut AnyRng::NoOp(NoOpRng)), available_cols: vec![0, 1] });
    let mut game = TestHarness::with_state(state);
    game.send("upgrade 1").unwrap(); // upgrade deck[0] = Strike → StrikePlus
    let GameState::Map(map) = &game.state else { panic!("expected Map") };
    assert_eq!(map.player.deck[0], Card::Strike(Grade::Plus));
}

// --- Debug mode ---

#[test]
fn win_command_rejected_without_debug_flag() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base)]);
    assert!(game.send("win").is_err());
}

#[test]
fn skip_command_rejected_without_debug_flag() {
    let mut game = TestHarness::with_state(new_run(&mut NoOpRng));
    assert!(game.send("skip").is_err());
}

#[test]
fn win_command_kills_enemy_in_debug_mode() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base)]).debug();
    game.send("win").unwrap();
    assert!(game.is_victory());
}

#[test]
fn skip_floor_advances_map_in_debug_mode() {
    let mut game = TestHarness::with_state(new_run(&mut NoOpRng)).debug();
    game.send("skip").unwrap();
    let GameState::Map(map) = &game.state else { panic!("expected Map") };
    assert_eq!(map.floor, 1);
}

// --- Defeat ---

#[test]
fn player_dies_when_hp_reaches_zero() {
    let mut game = TestHarness::with_hand(Vec::new());
    if let GameState::Combat { state, .. } = &mut game.state {
        state.player.hp = Hp(1); // one enemy attack (8 dmg) will kill
        state.player.draw_pile = vec![Card::Strike(Grade::Base); 5];
    }
    game.send("end").unwrap(); // enemy attacks → player dead → GameOver
    assert!(matches!(game.state, GameState::GameOver { victory: false }));
}

// --- Debug relic command ---

#[test]
fn relic_command_rejected_without_debug_flag() {
    let mut game = TestHarness::with_state(new_run(&mut NoOpRng));
    assert!(game.send("relic strawberry").is_err());
}

#[test]
fn relic_command_adds_relic_on_map_in_debug_mode() {
    let mut game = TestHarness::with_state(new_run(&mut NoOpRng)).debug();
    game.send("relic strawberry").unwrap();
    let GameState::Map(map) = &game.state else { panic!("expected Map") };
    assert!(map.player.relics.contains(&Relic::Strawberry));
}

#[test]
fn relic_command_raises_max_hp_via_strawberry() {
    let mut game = TestHarness::with_state(new_run(&mut NoOpRng)).debug();
    game.send("relic strawberry").unwrap();
    let GameState::Map(map) = &game.state else { panic!("expected Map") };
    assert_eq!(map.player.max_hp, Hp(87));
}

#[test]
fn relic_command_adds_relic_during_combat_in_debug_mode() {
    let mut game = TestHarness::with_hand(vec![Card::Strike(Grade::Base)]).debug();
    game.send("relic anchor").unwrap();
    let GameState::Combat { state, .. } = &game.state else { panic!("expected Combat") };
    assert!(state.player.relics.contains(&Relic::Anchor));
}

#[test]
fn relic_unknown_id_returns_error() {
    let mut game = TestHarness::with_state(new_run(&mut NoOpRng)).debug();
    assert!(game.send("relic not-a-relic").is_err());
}
