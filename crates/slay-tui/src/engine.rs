use slay_core::{
    AnyRng, Card, CardType, CombatPhase, Command, CommandError, Enemy, EnemyKind, Event, GameState,
    Intent, StatusEffect, StatusMap, Target,
};

/// Applies one player command, then auto-drains all EnemyTurn ticks.
/// Returns the final state plus all events (command events + every enemy-turn tick) combined.
pub fn apply_and_drain(
    mut state: GameState,
    command: Command,
    rng: &mut AnyRng,
) -> Result<(GameState, Vec<Event>), CommandError> {
    let (new_state, mut all_events) = slay_core::apply_command(state, command, rng)?;
    state = new_state;
    loop {
        let is_enemy_turn = matches!(
            &state,
            GameState::Combat { state: cs, .. } if cs.phase == CombatPhase::EnemyTurn
        );
        if !is_enemy_turn {
            break;
        }
        let (ns, evts) = slay_core::apply_command(state, Command::EndEnemyTurn, rng)?;
        state = ns;
        all_events.extend(evts);
    }
    Ok((state, all_events))
}

pub fn describe_event(event: &Event) -> String {
    match event {
        Event::CardPlayed { card } => format!("▶ You play {}.", card.name()),
        Event::PlayerAttacked { raw, damage } => {
            if *damage == 0 {
                format!("⚔️  You attack {raw}. (fully blocked)")
            } else if *damage < *raw {
                format!("⚔️  You deal {damage} damage. ({} blocked by enemy)", raw - damage)
            } else {
                format!("⚔️  You deal {damage} damage.")
            }
        }
        Event::PlayerBlocked { amount } => format!("🛡️  You gain {amount} block."),
        Event::EnemyAttacked { raw, damage } => {
            if *damage == 0 {
                format!("⚔️  Enemy attacks {raw}. (fully blocked)")
            } else if *damage < *raw {
                format!("⚔️  Enemy attacks {raw}. ({} blocked, {damage} damage)", raw - damage)
            } else {
                format!("⚔️  Enemy attacks {damage}.")
            }
        }
        Event::EnemyDefended { amount } => format!("🛡️  Enemy gains {amount} block."),
        Event::IntentRevealed { intent } => format!("👁  Enemy prepares: {}.", describe_intent(intent)),
        Event::PlayerBlockExpired { amount } => format!("🛡️  Your {amount} block expired."),
        Event::EnemyDied => "💀 Enemy slain!".into(),
        Event::PlayerDied => "💀 You have been slain.".into(),
        Event::EnemyPoisoned { damage } => format!("{} Poison deals {damage} to enemy.", status_display(StatusEffect::Poison).0),
        Event::TurnEnded => String::new(),
        Event::TurnStarted { turn } => format!("─── Turn {turn} ───"),
        Event::StatusApplied { target, status, stacks } => {
            let (icon, name) = status_display(*status);
            match target {
                Target::Player => format!("{icon} You gain {stacks} {name}."),
                Target::Enemy => format!("{icon} Enemy gains {stacks} {name}."),
            }
        }
        Event::GoldEarned { amount } => format!("🪙 You earn {amount} gold."),
        Event::Healed { amount } => format!("❤️‍🩹 You heal for {amount} HP."),
        Event::PlayerSelfDamaged { amount } => format!("🩸 You lose {amount} HP."),
        Event::EnergyGained { amount } => format!("⚡ You gain {amount} energy."),
        Event::CardsDrawn { count } => format!("🃏 You draw {count} card{}.", if *count == 1 { "" } else { "s" }),
        Event::CardAdded { card } => format!("✨ {} added to your deck.", card.name()),
        Event::CardExhausted { card } => format!("🔥 {} was exhausted.", card.name()),
        Event::CardUpgraded { from, to } => format!("⬆️  {} upgraded to {}.", from.name(), to.name()),
        Event::StatusCardAddedToDiscard { card } => format!("🃏 {} added to your discard.", card.name()),
        Event::PotionUsed { potion } => format!("🧪 You use {}.", potion.name()),
        Event::PotionAwarded { potion } => format!("🧪 {} added to your belt.", potion.name()),
        Event::PotionDiscarded { potion } => format!("🧪 {} discarded.", potion.name()),
    }
}

pub fn describe_intent(intent: &Intent) -> String {
    match intent {
        Intent::Attack(n) => format!("⚔️  Attack {n}"),
        Intent::Defend(n) => format!("🛡️  Defend {n}"),
        Intent::AttackDefend(d, b) => format!("⚔️🛡️  Attack {d} + Defend {b}"),
        Intent::Buff => "✨ Buff".into(),
        Intent::Debuff => "💀 Debuff".into(),
    }
}

pub fn status_display(status: StatusEffect) -> (&'static str, &'static str) {
    match status {
        StatusEffect::Vulnerable => ("🎯", "Vulnerable"),
        StatusEffect::Weak       => ("🪫", "Weak"),
        StatusEffect::Poison     => ("🟢", "Poison"),
        StatusEffect::Strength   => ("💪", "Strength"),
        StatusEffect::Ritual     => ("🔮", "Ritual"),
        StatusEffect::Dexterity  => ("🛡️", "Dexterity"),
        StatusEffect::Entangle   => ("🕸️", "Entangle"),
    }
}

pub fn statuses_inline(statuses: &StatusMap) -> String {
    if statuses.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = statuses
        .iter()
        .map(|(s, n)| {
            let (icon, _) = status_display(*s);
            format!("{icon}{n}")
        })
        .collect();
    format!("  [{}]", parts.join(" "))
}

pub fn card_type_icon(card_type: CardType) -> &'static str {
    match card_type {
        CardType::Attack  => "⚔️ ",
        CardType::Skill   => "🪄 ",
        CardType::Power   => "🔮 ",
        CardType::Curse   => "😈 ",
        CardType::Status  => "🩹 ",
    }
}

pub fn enemy_icon(enemy: &Enemy) -> &'static str {
    match enemy.kind {
        EnemyKind::Fungibeast      => "🍄",
        EnemyKind::Cultist         => "🐦",
        EnemyKind::JawWorm         => "🪱",
        EnemyKind::SmallSpikeSlime => "🫧",
        EnemyKind::RedLouse        => "🦟",
        EnemyKind::GreenLouse      => "🦟",
        EnemyKind::SmallAcidSlime  => "🫧",
        EnemyKind::BlueSlaver      => "⛓️",
        EnemyKind::RedSlaver       => "⛓️",
    }
}

pub fn pile_names(pile: &[Card]) -> Vec<String> {
    pile.iter().map(|c| c.name().to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use slay_core::{AnyRng, Command, EnemyKind, GameState, NoOpRng, new_simple_run};

    fn rng() -> AnyRng { AnyRng::NoOp(NoOpRng) }

    fn in_combat() -> GameState {
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap().0;
        apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0
    }

    #[test]
    fn apply_and_drain_returns_player_turn_after_end_turn() {
        use slay_core::CombatPhase;
        let state = in_combat();
        let mut r = rng();
        let (new_state, _) = apply_and_drain(state, Command::EndTurn, &mut r).unwrap();
        let GameState::Combat { state: cs, .. } = new_state else { panic!("expected Combat") };
        assert_eq!(cs.phase, CombatPhase::PlayerTurn);
    }

    #[test]
    fn apply_and_drain_flattens_events_from_enemy_turn() {
        let state = in_combat();
        let mut r = rng();
        let (_, events) = apply_and_drain(state, Command::EndTurn, &mut r).unwrap();
        // Should contain TurnEnded (from EndTurn) + TurnStarted (from enemy turn processing)
        assert!(events.iter().any(|e| matches!(e, Event::TurnEnded)));
        assert!(events.iter().any(|e| matches!(e, Event::TurnStarted { .. })));
    }

    #[test]
    fn apply_and_drain_propagates_error() {
        let state = in_combat();
        let mut r = rng();
        // PlayCard on empty hand (Simple run has no cards) → InvalidCard
        let result = apply_and_drain(state, Command::PlayCard(0, 0), &mut r);
        assert!(result.is_err());
    }
}
