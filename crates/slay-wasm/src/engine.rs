use slay_core::{
    AnyRng, CardType, Command, CommandError, Enemy, EnemyKind, Event, GameState, Intent,
    CombatPhase, MapNode, Relic, StatusEffect, StatusMap, Target,
};

pub fn apply_and_drain(
    mut state: GameState,
    command: Command,
    rng: &mut AnyRng,
) -> Result<(GameState, Vec<Event>), CommandError> {
    let (new_state, mut all_events) = slay_core::apply_command(state, command, rng)?;
    state = new_state;
    loop {
        let cmd = match &state {
            GameState::Combat { state: cs, .. } if cs.phase == CombatPhase::EnemyTurn => Command::EndEnemyTurn,
            GameState::Combat { state: cs, .. } if cs.phase == CombatPhase::StartOfPlayerTurn => Command::StartPlayerTurn,
            _ => break,
        };
        let (ns, evts) = slay_core::apply_command(state, cmd, rng)?;
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
        Event::EnemySplit => "🔀 Slime splits!".into(),
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
        Event::MaxHpIncreased { amount } => format!("❤️  Your Max HP increases by {amount}."),
        Event::PlayerSelfDamaged { amount } => format!("🩸 You lose {amount} HP."),
        Event::EnergyGained { amount } => format!("⚡ You gain {amount} energy."),
        Event::CardsDrawn { count } => format!("🃏 You draw {count} card{}.", if *count == 1 { "" } else { "s" }),
        Event::CardAdded { card } => format!("✨ {} added to your deck.", card.name()),
        Event::CardExhausted { card } => format!("🔥 {} was exhausted.", card.name()),
        Event::CardUpgraded { from, to } => format!("⬆️  {} upgraded to {}.", from.name(), to.name()),
        Event::StatusCardAddedToDiscard { card } => format!("🃏 {} added to your discard.", card.name()),
        Event::StatusCardAddedToHand { card } => format!("🃏 {} added to your hand.", card.name()),
        Event::PotionUsed { potion } => format!("🧪 You use {}.", potion.name()),
        Event::PotionAwarded { potion } => format!("🧪 {} added to your belt.", potion.name()),
        Event::PotionDiscarded { potion } => format!("🧪 {} discarded.", potion.name()),
        Event::EnemyFled => "🏃 Enemy fled!".into(),
        Event::GoldStolen { amount } => format!("💰 Enemy stole {amount} gold!"),
        Event::GoldReturned { amount } => format!("💰 Returned {amount} gold."),
        Event::RelicObtained { relic } => format!("✨ You obtain {}.", relic.name()),
    }
}

pub fn describe_intent(intent: &Intent) -> String {
    match intent {
        Intent::Attack(n) => format!("⚔️  Attack {n}"),
        Intent::AttackDebuff(n) => format!("⚔️💀 Attack {n} + Debuff"),
        Intent::Defend(n) => format!("🛡️  Defend {n}"),
        Intent::AttackDefend(d, b) => format!("⚔️🛡️  Attack {d} + Defend {b}"),
        Intent::Buff => "✨ Buff".into(),
        Intent::Debuff => "💀 Debuff".into(),
        Intent::Split => "🔀 Split".into(),
        Intent::EscapeBlock(n) => format!("🏃 Flee + Defend {n}"),
        Intent::Escape => "🏃 Flee".into(),
        Intent::AttackUnknown => "⚔️  ?".into(),
        Intent::BlockAndGainStrength(n) => format!("🛡️{n} + 💪"),
    }
}

pub fn status_display(status: StatusEffect) -> (&'static str, &'static str) {
    match status {
        StatusEffect::Vulnerable       => ("🎯", "Vulnerable"),
        StatusEffect::Weak             => ("🪫", "Weak"),
        StatusEffect::Poison           => ("🟢", "Poison"),
        StatusEffect::Strength         => ("💪", "Strength"),
        StatusEffect::Ritual           => ("🔮", "Ritual"),
        StatusEffect::Dexterity        => ("🛡️", "Dexterity"),
        StatusEffect::Entangle         => ("🕸️", "Entangle"),
        StatusEffect::Frail            => ("🫧", "Frail"),
        StatusEffect::SharpHide        => ("🦔", "Sharp Hide"),
        StatusEffect::ModeShiftProgress => ("⚡", "Mode Shift"),
        StatusEffect::ModeShiftCount   => ("🔄", "Mode Shifts"),
        StatusEffect::GuardianMode     => ("🛡️", "Guardian Mode"),
        StatusEffect::DemonForm        => ("😈", "Demon Form"),
        StatusEffect::Barricade        => ("🏰", "Barricade"),
        StatusEffect::FeelNoPain       => ("🩹", "Feel No Pain"),
        StatusEffect::DarkEmbrace      => ("🌑", "Dark Embrace"),
        StatusEffect::Juggernaut       => ("🏋️", "Juggernaut"),
        StatusEffect::Unmovable        => ("🪨", "Unmovable"),
        StatusEffect::Rupture          => ("💔", "Rupture"),
        StatusEffect::Berserk          => ("😡", "Berserk"),
        StatusEffect::Brutality        => ("🩹", "Brutality"),
        StatusEffect::Combust          => ("🔥", "Combust"),
        StatusEffect::Evolve           => ("🧬", "Evolve"),
        StatusEffect::FireBreathing    => ("🐉", "Fire Breathing"),
        StatusEffect::StrengthDown     => ("📉", "Strength Down"),
        StatusEffect::DexterityDown    => ("📉", "Dexterity Down"),
        StatusEffect::Shackled         => ("⛓️", "Shackled"),
        StatusEffect::StonePlating     => ("🪨", "Stone Plating"),
        StatusEffect::Enrage           => ("😤", "Enrage"),
        StatusEffect::Metallicize      => ("🔩", "Metallicize"),
        StatusEffect::Stunned          => ("💫", "Stunned"),
        StatusEffect::Sleep            => ("💤", "Sleep"),
        StatusEffect::CurlUp          => ("🛡️", "Curl Up"),
        StatusEffect::Panache          => ("✨", "Panache"),
        StatusEffect::Regen            => ("💚", "Regen"),
        StatusEffect::Thorns           => ("🌵", "Thorns"),
        StatusEffect::Artifact         => ("🏺", "Artifact"),
        StatusEffect::SadisticNature   => ("😈", "Sadistic Nature"),
        StatusEffect::Mayhem           => ("💥", "Mayhem"),
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
        CardType::Attack => "⚔️ ",
        CardType::Skill  => "🪄 ",
        CardType::Power  => "🔮 ",
        CardType::Curse  => "😈 ",
        CardType::Status => "🩹 ",
    }
}

pub fn enemy_icon(enemy: &Enemy) -> &'static str {
    match enemy.kind {
        EnemyKind::Fungibeast      => "🍄",
        EnemyKind::Cultist         => "🦆",
        EnemyKind::JawWorm         => "🦈",
        EnemyKind::SmallSpikeSlime => "🟢",
        EnemyKind::RedLouse        => "🦟",
        EnemyKind::GreenLouse      => "🦟",
        EnemyKind::SmallAcidSlime  => "🟢",
        EnemyKind::BlueSlaver      => "⛓️",
        EnemyKind::RedSlaver       => "⛓️",
        EnemyKind::TheGuardian     => "🦾",
        EnemyKind::GremlinNob      => "👺",
        EnemyKind::Lagavulin       => "🐚",
        EnemyKind::MadGremlin      => "😤",
        EnemyKind::SneakyGremlin   => "🗡️",
        EnemyKind::FatGremlin      => "🐷",
        EnemyKind::GremlinWizard   => "🧙",
        EnemyKind::ShieldGremlin   => "🛡️",
        EnemyKind::Sentry          => "🤖",
        EnemyKind::SlimeBoss       => "🟣",
        EnemyKind::LargeSpike      => "🔵",
        EnemyKind::MediumSpike     => "🔵",
        EnemyKind::LargeAcid       => "🟢",
        EnemyKind::MediumAcid      => "🟢",
        EnemyKind::Looter          => "🦹",
        EnemyKind::Mugger          => "🦹",
        EnemyKind::Hexaghost       => "👻",
    }
}

pub fn map_node_icon(node: &MapNode) -> &'static str {
    match node {
        MapNode::Combat(_) => "⚔️",
        MapNode::Elite(_)  => "⚡",
        MapNode::RestSite  => "🔥",
        MapNode::Boss(_)   => "💀",
        MapNode::Merchant  => "🛒",
        MapNode::Treasure  => "📦",
        MapNode::Event     => "❓",
    }
}

pub fn map_node_name(node: &MapNode) -> &'static str {
    match node {
        MapNode::Combat(_) => "Combat",
        MapNode::Elite(_)  => "Elite",
        MapNode::RestSite  => "Rest Site",
        MapNode::Boss(_)   => "Boss",
        MapNode::Merchant  => "Shop",
        MapNode::Treasure  => "Treasure",
        MapNode::Event     => "Event",
    }
}

pub fn connector_rows(floor_edges: &[Vec<usize>], num_cols: usize) -> (String, String) {
    const STRIDE: usize = 6;
    let width = num_cols * STRIDE;
    let mut r0: Vec<char> = vec![' '; width];
    let mut r1: Vec<char> = vec![' '; width];

    for (src, dsts) in floor_edges.iter().enumerate() {
        for &dst in dsts {
            let sc = src * STRIDE + 1;
            let dc = dst * STRIDE + 1;
            match src.cmp(&dst) {
                std::cmp::Ordering::Equal => {
                    r0[sc] = '│';
                    r1[sc] = '│';
                }
                std::cmp::Ordering::Less => {
                    let mid = (sc + dc) / 2;
                    r1[mid - 1] = '╱';
                    r0[mid + 1] = '╱';
                }
                std::cmp::Ordering::Greater => {
                    let mid = (sc + dc) / 2;
                    r1[mid + 1] = '╲';
                    r0[mid - 1] = '╲';
                }
            }
        }
    }

    (r0.iter().collect(), r1.iter().collect())
}

pub fn relic_emoji(relic: &Relic) -> &'static str {
    match relic {
        Relic::Strawberry       => "🍓",
        Relic::Pear             => "🍐",
        Relic::Mango            => "🥭",
        Relic::OldCoin          => "🪙",
        Relic::Whetstone        => "🪨",
        Relic::WarPaint         => "🎨",
        Relic::BurningBlood     => "🔥",
        Relic::BlackBlood       => "🖤",
        Relic::Anchor           => "⚓",
        Relic::Vajra            => "🔱",
        Relic::Lantern          => "🏮",
        Relic::BloodVial        => "🩸",
        Relic::BagOfMarbles     => "🎱",
        Relic::RedMask          => "😶",
        Relic::FestivePopper    => "🎉",
        Relic::Pantograph       => "📐",
        Relic::BagOfPreparation => "🎒",
        Relic::MercuryHourglass => "⏳",
        Relic::CaptainsWheel    => "⚙️",
        Relic::Chandelier       => "💡",
        Relic::Candelabra       => "🕯️",
        Relic::HornCleat        => "🪝",
        Relic::HappyFlower      => "🌸",
        Relic::Pendulum         => "🕰️",
        Relic::StoneCalendar    => "📅",
        Relic::Orichalcum       => "🟠",
        Relic::CloakClasp       => "🪆",
        Relic::RegalPillow      => "🛏️",
        Relic::Nunchaku         => "🥋",
        Relic::OrnamentalFan    => "🪭",
        Relic::Kunai            => "🗡️",
        Relic::Shuriken         => "⭐",
        Relic::Kusarigama       => "⛓️",
        Relic::LetterOpener     => "✉️",
        Relic::TuningFork       => "🎵",
        Relic::GremlinHorn      => "📯",
        Relic::Pocketwatch      => "⌚",
    }
}

pub fn relics_bar(relics: &[Relic]) -> String {
    if relics.is_empty() {
        return String::new();
    }
    relics.iter().map(relic_emoji).collect::<Vec<_>>().join(" ")
}
