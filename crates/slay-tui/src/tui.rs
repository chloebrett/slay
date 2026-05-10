use crate::engine::{
    apply_and_drain, card_type_icon, connector_rows, describe_event, describe_intent, enemy_icon,
    map_node_icon, map_node_name, relic_emoji, relics_bar, statuses_inline, MAP_COL_STRIDE,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
#[cfg(any(feature = "terminal", test))]
use ratatui::Terminal;
use slay_core::{
    AnyRng, CardRewardState, CombatPhase, CombatState, EventKind, EventRoomState, GameState,
    MapNode, MapState, Relic, RestSiteState, ShopState, TreasureRoomState, StatusMap,
    CARD_PRICE, RELIC_PRICE, POTION_PRICE,
};
use std::collections::VecDeque;

// Instant::now() is unavailable on wasm32-unknown-unknown; animations are no-ops there.
#[cfg(not(target_arch = "wasm32"))]
fn now() -> Option<std::time::Instant> { Some(std::time::Instant::now()) }
#[cfg(target_arch = "wasm32")]
fn now() -> Option<std::time::Instant> { None }

const LOG_CAPACITY: usize = 200;

#[allow(dead_code)] // used in the event loop, not in test compilation
const WIPE_DURATION: std::time::Duration = std::time::Duration::from_millis(500);
const FLASH_DURATION: std::time::Duration = std::time::Duration::from_millis(200);

const PILE_KEYS: &[(&str, PileView, &str)] = &[
    ("z", PileView::Draw,    "view draw pile"),
    ("x", PileView::Discard, "view discard pile"),
    ("c", PileView::Exhaust, "view exhaust pile"),
];

pub struct TuiState {
    pub game: GameState,
    pub input_buf: String,
    pub event_log: VecDeque<String>,
    pub last_error: Option<String>,
    pub show_pile: Option<PileView>,
    pub show_help: bool,
    pub show_relics: bool,
    pub wipe_start: Option<std::time::Instant>,
    pub player_flash: Option<std::time::Instant>,
    pub enemy_flashes: Vec<Option<std::time::Instant>>,
    pub hand_scroll: usize,
    pub map_scroll: usize,
    pub debug: bool,
    pub should_quit: bool,
    #[cfg(feature = "terminal")]
    save_tx: Option<std::sync::mpsc::SyncSender<Option<(GameState, u64)>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PileView {
    Deck,
    Draw,
    Discard,
    Exhaust,
}

impl TuiState {
    pub fn new(game: GameState, debug: bool) -> Self {
        #[cfg(feature = "terminal")]
        return Self::new_with_save(game, debug, None);
        #[cfg(not(feature = "terminal"))]
        return Self::new_inner(game, debug);
    }

    #[cfg(feature = "terminal")]
    pub fn new_with_save(
        game: GameState,
        debug: bool,
        save_tx: Option<std::sync::mpsc::SyncSender<Option<(GameState, u64)>>>,
    ) -> Self {
        let mut s = Self::new_inner(game, debug);
        s.save_tx = save_tx;
        s
    }

    fn new_inner(game: GameState, debug: bool) -> Self {
        let enemy_flashes = match &game {
            GameState::Combat { state: cs, .. } => vec![None; cs.enemies.len()],
            _ => vec![],
        };
        let mut s = Self {
            game,
            input_buf: String::new(),
            event_log: VecDeque::new(),
            last_error: None,
            show_pile: None,
            show_help: false,
            show_relics: false,
            wipe_start: None,
            player_flash: None,
            enemy_flashes,
            hand_scroll: 0,
            map_scroll: 0,
            debug,
            should_quit: false,
            #[cfg(feature = "terminal")]
            save_tx: None,
        };
        s.push_log(slay_core::welcome().to_string());
        if debug {
            s.push_log("⚙️  debug mode".to_string());
        }
        s
    }

    pub fn push_log(&mut self, line: String) {
        if line.is_empty() {
            return;
        }
        if self.event_log.len() >= LOG_CAPACITY {
            self.event_log.pop_front();
        }
        self.event_log.push_back(line);
    }

    /// Process the current `input_buf` as a command. Mutates `game`, `event_log`, `last_error`,
    /// and clears `input_buf`. Sets `should_quit` on game over.
    pub fn handle_enter(&mut self, rng: &mut AnyRng) {
        let input = std::mem::take(&mut self.input_buf);
        let trimmed = input.trim();

        // Global relic overlay toggle
        if trimmed == "relics" {
            self.show_relics = !self.show_relics;
            self.show_pile = None;
            return;
        }

        // d = full deck, available in all states with a player
        if trimmed == "d" && !matches!(self.game, GameState::GameOver { .. } | GameState::Neow(_)) {
            self.show_pile = Some(PileView::Deck);
            self.show_relics = false;
            return;
        }

        // Pile view shortcuts in combat
        if matches!(self.game, GameState::Combat { .. }) {
            if let Some((_, view, _)) = PILE_KEYS.iter().find(|(k, _, _)| *k == trimmed) {
                self.show_pile = Some(*view);
                self.show_relics = false;
                return;
            }
        }

        // Dismiss pile/relic overlays on any other command
        self.show_pile = None;
        self.show_relics = false;

        let Some(command) = crate::command::parse(trimmed, &self.game, self.debug) else {
            self.last_error = Some("Unknown command.".to_string());
            return;
        };

        match apply_and_drain(self.game.clone(), command, rng) {
            Ok((new_state, events)) => {
                let player_hp_before = combat_player_hp(&self.game);
                let enemy_hps_before = combat_enemy_hps(&self.game);

                if matches!(&self.game, GameState::Map(_)) && !matches!(&new_state, GameState::Map(_)) {
                    self.wipe_start = now();
                }
                if let Some(banner) = phase_banner(&self.game, &new_state) {
                    self.push_log(banner);
                }
                self.game = new_state;

                // Re-initialise flash slots when enemy count changes (new combat, etc.)
                let enemy_count = match &self.game {
                    GameState::Combat { state: cs, .. } => cs.enemies.len(),
                    _ => 0,
                };
                if self.enemy_flashes.len() != enemy_count {
                    self.enemy_flashes = vec![None; enemy_count];
                }

                // Set flashes for any HP that decreased
                if let (Some(before), Some(after)) = (player_hp_before, combat_player_hp(&self.game)) {
                    if after < before {
                        self.player_flash = now();
                    }
                }
                for (i, (&before, after)) in enemy_hps_before.iter().zip(combat_enemy_hps(&self.game)).enumerate() {
                    if after < before {
                        if let Some(slot) = self.enemy_flashes.get_mut(i) {
                            *slot = now();
                        }
                    }
                }

                self.hand_scroll = 0;
                if matches!(&self.game, GameState::Map(_)) {
                    self.map_scroll = 0;
                }
                for ev in &events {
                    let msg = describe_event(ev);
                    self.push_log(msg);
                }
                self.last_error = None;
                if let GameState::GameOver { .. } = &self.game {
                    #[cfg(feature = "terminal")]
                    {
                        let victory = matches!(&self.game, GameState::GameOver { victory } if *victory);
                        crate::game::on_run_end_tui(victory, &self.save_tx);
                    }
                    self.should_quit = true;
                } else {
                    #[cfg(feature = "terminal")]
                    if let Some(tx) = &self.save_tx {
                        let seed = rng.seed().unwrap_or(0);
                        let _ = tx.try_send(Some((self.game.clone(), seed)));
                    }
                }
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
            }
        }
    }
}

fn combat_player_hp(state: &GameState) -> Option<i32> {
    match state {
        GameState::Combat { state: cs, .. } => Some(cs.player.hp.0),
        _ => None,
    }
}

fn combat_enemy_hps(state: &GameState) -> Vec<i32> {
    match state {
        GameState::Combat { state: cs, .. } => cs.enemies.iter().map(|e| e.hp.0).collect(),
        _ => vec![],
    }
}

fn is_flash_active(flash: Option<std::time::Instant>) -> bool {
    flash.map(|t| t.elapsed() < FLASH_DURATION).unwrap_or(false)
}

fn hp_color(hp: i32, max_hp: i32) -> Color {
    if max_hp == 0 { return Color::White; }
    let pct = hp * 100 / max_hp;
    if pct <= 20 { Color::Red }
    else if pct <= 50 { Color::Yellow }
    else { Color::White }
}

fn phase_banner(before: &GameState, after: &GameState) -> Option<String> {
    if std::mem::discriminant(before) == std::mem::discriminant(after) {
        return None;
    }
    let banner = match after {
        GameState::Combat { state: cs, floor, is_boss, is_elite, .. } => {
            let enemies: Vec<&str> = cs.enemies.iter().map(|e| e.name()).collect();
            let label = if *is_boss { "Boss" } else if *is_elite { "Elite" } else { "Combat" };
            format!("══ ⚔️  Floor {} — {label}: {} ══", floor + 1, enemies.join(", "))
        }
        GameState::RestSite(rs) => format!("══ 🏕️  Floor {} — Rest Site ══", rs.floor + 1),
        GameState::Shop(s) => format!("══ 🛒  Floor {} — Merchant ══", s.floor + 1),
        GameState::CardReward(cr) => format!("══ 🃏  Floor {} — Card Reward ══", cr.floor),
        GameState::TreasureRoom(tr) => format!("══ 💰  Floor {} — Treasure ══", tr.floor + 1),
        GameState::EventRoom(er) => {
            let event_name = match er.event {
                EventKind::Ssssserpent => "Ssssserpent",
                EventKind::BigFish => "Big Fish",
                EventKind::Mushrooms => "Mushrooms",
                EventKind::GoldenIdol => "Golden Idol",
            };
            format!("══ 📜  Floor {} — {event_name} ══", er.floor + 1)
        }
        GameState::Map(_) | GameState::GameOver { .. } | GameState::Neow(_) => return None,
    };
    Some(banner)
}

fn player_relics(state: &GameState) -> &[Relic] {
    match state {
        GameState::Map(m)            => &m.player.relics,
        GameState::Combat { state, .. } => &state.player.relics,
        GameState::RestSite(rs)      => &rs.player.relics,
        GameState::TreasureRoom(tr)  => &tr.player.relics,
        GameState::CardReward(cr)    => &cr.player.relics,
        GameState::Shop(shop)        => &shop.player.relics,
        GameState::EventRoom(er)     => &er.player.relics,
        GameState::Neow(neow)        => &neow.player.relics,
        GameState::GameOver { .. }   => &[],
    }
}

/// Render one frame. Pure function over `TuiState`; safe to call from tests with `TestBackend`.
pub fn render_frame(f: &mut Frame, tui: &mut TuiState) {
    let top_bar_height: u16 = if player_relics(&tui.game).is_empty() { 1 } else { 2 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_bar_height), // top bar (+ relic row when present)
            Constraint::Min(0),                 // main area
            Constraint::Length(1),              // status line
            Constraint::Length(3),              // input box
        ])
        .split(f.area());

    render_top_bar(f, chunks[0], tui);
    render_main(f, chunks[1], tui);
    render_status_line(f, chunks[2], tui);
    render_input(f, chunks[3], tui);

    // Overlays (drawn last, on top)
    if let Some(view) = tui.show_pile {
        render_pile_overlay(f, f.area(), &tui.game, view);
    }
    if tui.show_relics {
        render_relic_overlay(f, f.area(), player_relics(&tui.game));
    }
    if tui.show_help {
        render_help_overlay(f, f.area(), &tui.game);
    }
    if tui.wipe_start.is_some() {
        render_wipe_overlay(f, f.area(), &tui.game);
    }
}

fn render_top_bar(f: &mut Frame, area: Rect, tui: &TuiState) {
    let state = &tui.game;
    let player_flash_active = is_flash_active(tui.player_flash);
    let player = match state {
        GameState::Map(m) => Some(&m.player),
        GameState::Combat { state, .. } => Some(&state.player),
        GameState::RestSite(rs) => Some(&rs.player),
        GameState::TreasureRoom(tr) => Some(&tr.player),
        GameState::CardReward(cr) => Some(&cr.player),
        GameState::Shop(shop) => Some(&shop.player),
        GameState::EventRoom(er) => Some(&er.player),
        GameState::Neow(neow) => Some(&neow.player),
        GameState::GameOver { .. } => None,
    };
    let lines: Vec<Line> = match player {
        Some(p) => {
            let potions: Vec<String> = p.potions.iter().enumerate()
                .map(|(i, pot)| format!("[{}]{}", i + 1, pot.name()))
                .collect();
            let potion_str = if potions.is_empty() {
                String::new()
            } else {
                format!("   🧪 {}", potions.join(" "))
            };
            let hp_str = format!("HP {}/{} {}", p.hp.0, p.max_hp.0, hp_bar(p.hp.0, p.max_hp.0, 20));
            let status_str = statuses_inline(&p.statuses);
            let rest = format!(
                "   ⚡ {}/{}   🛡 {}   🪙 {}   🃏 {} cards{}{}",
                p.energy.0, p.max_energy.0, p.block.0, p.gold, p.deck.len(), potion_str, status_str
            );
            let hp_fg = if player_flash_active {
                Color::LightRed
            } else {
                hp_color(p.hp.0, p.max_hp.0)
            };
            use ratatui::text::Span;
            let stats_line = Line::from(vec![
                Span::raw("🧙  "),
                Span::styled(hp_str, Style::default().fg(hp_fg)),
                Span::raw(rest),
            ]);
            let bar = relics_bar(&p.relics);
            if bar.is_empty() {
                vec![stats_line]
            } else {
                vec![stats_line, Line::raw(bar)]
            }
        }
        None => vec![Line::raw("")],
    };
    let para = Paragraph::new(lines).style(Style::default().fg(Color::White));
    f.render_widget(para, area);
}

fn render_main(f: &mut Frame, area: Rect, tui: &mut TuiState) {
    let TuiState { game, map_scroll, event_log, enemy_flashes, hand_scroll, .. } = &mut *tui;
    match game {
        GameState::Map(map) => render_map(f, area, map, map_scroll),
        GameState::Combat { state, .. } => render_combat(f, area, state, event_log, enemy_flashes, *hand_scroll),
        GameState::RestSite(rs) => render_rest(f, area, rs),
        GameState::TreasureRoom(tr) => render_treasure(f, area, tr),
        GameState::CardReward(cr) => render_card_reward(f, area, cr),
        GameState::Shop(shop) => render_shop(f, area, shop),
        GameState::EventRoom(er) => render_event_room(f, area, er),
        GameState::GameOver { victory } => render_game_over(f, area, *victory),
        GameState::Neow(neow) => render_neow(f, area, neow),
    }
}

fn render_neow(f: &mut Frame, area: Rect, neow: &slay_core::NeowState) {
    use ratatui::widgets::Paragraph;
    let mut lines = vec![
        ratatui::text::Line::raw("🌟 Neow's Blessings — choose:"),
        ratatui::text::Line::raw(""),
    ];
    for (i, blessing) in neow.blessings.iter().enumerate() {
        lines.push(ratatui::text::Line::raw(format!("  [{}] {}", i + 1, blessing.describe())));
    }
    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

fn render_map(f: &mut Frame, area: Rect, map: &MapState, map_scroll: &mut usize) {
    let [map_area, choices_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .areas(area);

    let max_cols = map.graph.rows.iter().map(|r| r.len()).max().unwrap_or(1).max(2);
    let max_floor = map.graph.rows.len().saturating_sub(1);
    let offset_of = |row: &[_]| (max_cols - row.len()) / 2;

    let map_content_width = max_cols * MAP_COL_STRIDE;
    let inner_width = map_area.width.saturating_sub(2) as usize;
    let left_margin = inner_width.saturating_sub(map_content_width) / 2;
    let margin = " ".repeat(left_margin);

    let available_height = map_area.height.saturating_sub(2) as usize;
    let floors_per_screen = (available_height / 3).max(1);

    let (view_bottom, view_top) = if max_floor + 1 <= floors_per_screen {
        (0, max_floor)
    } else {
        let clamped = (*map_scroll).min(max_floor.saturating_sub(floors_per_screen.saturating_sub(1)));
        *map_scroll = clamped;
        let view_top = (clamped + floors_per_screen - 1).min(max_floor);
        (clamped, view_top)
    };

    let mut lines: Vec<Line> = Vec::new();

    if view_top < max_floor {
        lines.push(Line::raw(format!("{margin}  ↑ more floors (↑ / W)")));
    }

    for floor in (view_bottom..=view_top).rev() {
        let row = &map.graph.rows[floor];
        let is_boss = row.iter().any(|n| matches!(n, MapNode::Boss(_)));
        let past = floor < map.floor;
        let current = floor == map.floor;
        let off = offset_of(row);

        if is_boss { lines.push(Line::raw("")); }

        // Node row: left margin + centering offset + one span per column + separators
        use ratatui::text::Span;
        let mut spans: Vec<Span> = vec![Span::raw(format!(
            "{margin}{}",
            " ".repeat(off * MAP_COL_STRIDE)
        ))];
        for col in 0..row.len() {
            let icon = map_node_icon(&row[col]);
            let visited = map.graph.path.get(floor) == Some(&col);
            let style = if past {
                Style::default().fg(Color::DarkGray)
            } else if current && map.available_cols.contains(&col) {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let text = if past && visited { icon.to_string() } else if past { "·· ".to_string() } else { icon.to_string() };
            spans.push(Span::styled(text, style));
            if col < row.len() - 1 {
                spans.push(Span::raw("        "));
            }
        }
        lines.push(Line::from(spans));

        // Connector rows between this floor and the one below
        if floor > view_bottom {
            let lower_row = &map.graph.rows[floor - 1];
            let lower_off = offset_of(lower_row);
            if is_boss {
                let arrows_off = off.saturating_sub(lower_row.len() / 2);
                let pad = " ".repeat(left_margin + arrows_off * MAP_COL_STRIDE);
                let arrows = lower_row.iter().map(|_| "🔺").collect::<Vec<_>>().join("        ");
                lines.push(Line::raw(""));
                lines.push(Line::raw(format!("{pad}{arrows}")));
            } else {
                let upper_off = off;
                let conn_style = Style::default().fg(Color::DarkGray);
                let (r0, r1) = connector_rows(&map.graph.edges[floor - 1], max_cols, lower_off, upper_off);
                lines.push(Line::styled(format!("{margin}{r0}"), conn_style));
                lines.push(Line::styled(format!("{margin}{r1}"), conn_style));
            }
        }
    }

    if view_bottom > 0 {
        lines.push(Line::raw(format!("{margin}  ↓ more floors (↓ / S)")));
    }

    let block = Block::default().borders(Borders::ALL).title(" 🗺️  Map ");
    f.render_widget(Paragraph::new(lines).block(block), map_area);

    let choices = map_choices_line(map);
    let choices_block = Block::default().borders(Borders::ALL).title(" Choose ");
    f.render_widget(Paragraph::new(choices).block(choices_block), choices_area);
}

fn map_choices_line(map: &MapState) -> String {
    if map.available_cols.len() == 1 {
        let col = map.available_cols[0];
        let node = &map.graph.rows[map.floor][col];
        format!("[Enter] {} {}", map_node_icon(node), map_node_name(node))
    } else {
        map.available_cols.iter()
            .map(|&col| {
                let node = &map.graph.rows[map.floor][col];
                format!("[{}] {} {}", col + 1, map_node_icon(node), map_node_name(node))
            })
            .collect::<Vec<_>>()
            .join("   ")
    }
}

fn render_treasure(f: &mut Frame, area: Rect, tr: &TreasureRoomState) {
    let lines = vec![
        Line::from("📦 Treasure Room"),
        Line::from(""),
        Line::from("You found a chest containing:"),
        Line::from(format!("  ✨ {}", tr.relic.name())),
        Line::from(""),
        Line::from("[leave / take] Take the relic and leave"),
    ];
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Treasure "))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn render_combat(f: &mut Frame, area: Rect, state: &CombatState, log: &VecDeque<String>, enemy_flashes: &[Option<std::time::Instant>], hand_scroll: usize) {
    let [left, right] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .areas(area);

    let [enemies_area, hand_area, piles_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .areas(left);

    render_enemies(f, enemies_area, state, enemy_flashes);
    render_hand(f, hand_area, state, hand_scroll);
    render_piles(f, piles_area, state);
    render_log(f, right, log);
}

fn render_enemies(f: &mut Frame, area: Rect, state: &CombatState, enemy_flashes: &[Option<std::time::Instant>]) {
    let multi = state.enemies.len() > 1;
    let items: Vec<ListItem> = state.enemies.iter().enumerate().map(|(i, e)| {
        let prefix = if multi { format!("[{}] ", i + 1) } else { String::new() };
        let intent = describe_intent(&e.effective_intent(&state.player.statuses));
        let bar = hp_bar(e.hp.0, e.max_hp.0, 10);
        let statuses = statuses_inline(&e.statuses);
        let line = format!(
            "{}{} {}  HP {}/{} {}  🛡 {}  | {}{}",
            prefix, enemy_icon(e), e.name(), e.hp.0, e.max_hp.0, bar, e.block.0, intent, statuses
        );
        let flash_active = is_flash_active(enemy_flashes.get(i).copied().flatten());
        let style = if flash_active {
            Style::default().fg(Color::LightRed)
        } else {
            Style::default().fg(hp_color(e.hp.0, e.max_hp.0))
        };
        ListItem::new(Line::styled(line, style))
    }).collect();

    let block = Block::default().borders(Borders::ALL).title(" Enemies ");
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn hp_bar(current: i32, max: i32, width: usize) -> String {
    if max <= 0 {
        return String::new();
    }
    let cur = current.max(0).min(max);
    let filled = if cur > 0 {
        ((cur as usize) * width).saturating_div(max as usize).max(1)
    } else {
        0
    };
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn render_hand(f: &mut Frame, area: Rect, state: &CombatState, hand_scroll: usize) {
    let dummy = StatusMap::new();
    let target_statuses = state.enemies.first().map_or(&dummy, |e| &e.statuses);
    let choose_mode = matches!(state.phase, CombatPhase::ChooseCard(_));

    if state.player.hand.is_empty() {
        let title = if choose_mode { " 🎴 Choose a card " } else { " 🤚 Hand " };
        let block = Block::default().borders(Borders::ALL).title(title);
        let list = List::new(vec![ListItem::new(Line::styled("(empty)", Style::default().fg(Color::DarkGray)))]).block(block);
        f.render_widget(list, area);
        return;
    }

    // Build all card items (1-indexed labels stay tied to original position)
    let all_items: Vec<ListItem> = state.player.hand.iter().enumerate().map(|(i, card)| {
        let style = if choose_mode {
            Style::default().fg(Color::Yellow)
        } else if card.card_cost().is_affordable(state.player.energy) {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let desc = card.effective_description(&state.player.statuses, target_statuses);
        let text = format!(
            "[{}] {}{} ({}) — {}",
            i + 1,
            card_type_icon(card.card_type()),
            card.name(),
            card.card_cost().display(),
            desc,
        );
        ListItem::new(Line::styled(text, style))
    }).collect();

    let total = all_items.len();
    let inner_h = area.height.saturating_sub(2) as usize; // minus borders
    let scroll = hand_scroll.min(total.saturating_sub(1));

    let mut items: Vec<ListItem> = Vec::new();

    if total <= inner_h {
        // Everything fits — no hints needed
        items = all_items;
    } else {
        let need_above = scroll > 0;
        // Reserve a row for the above hint and tentatively for below hint
        let card_rows = inner_h
            .saturating_sub(need_above as usize)
            .saturating_sub(1); // always reserve 1 for possible below hint
        let end = (scroll + card_rows).min(total);
        let need_below = end < total;

        let card_rows = if need_below {
            card_rows
        } else {
            // No below hint needed — reclaim that row
            inner_h.saturating_sub(need_above as usize)
        };
        let end = (scroll + card_rows).min(total);

        if need_above {
            items.push(ListItem::new(Line::styled(
                format!("  ↑ {} above", scroll),
                Style::default().fg(Color::DarkGray),
            )));
        }
        items.extend(all_items.into_iter().skip(scroll).take(end - scroll));
        if need_below {
            items.push(ListItem::new(Line::styled(
                format!("  ↓ {} more  (↑/↓)", total - end),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let title = if choose_mode {
        " 🎴 Choose a card ".to_string()
    } else {
        format!(" 🤚 Hand  (Turn {}) ", state.turn)
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_piles(f: &mut Frame, area: Rect, state: &CombatState) {
    let text = format!(
        "Draw: {}    Discard: {}    Exhaust: {}    [z] [x] [c] view",
        state.player.draw_pile.len(),
        state.player.discard_pile.len(),
        state.player.exhaust_pile.len(),
    );
    let block = Block::default().borders(Borders::ALL);
    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

fn render_log(f: &mut Frame, area: Rect, log: &VecDeque<String>) {
    let visible = area.height.saturating_sub(2) as usize;
    let start = log.len().saturating_sub(visible);
    let lines: Vec<Line> = log.iter().skip(start).map(|s| {
        let style = if s.starts_with("───") {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        Line::styled(s.clone(), style)
    }).collect();

    let block = Block::default().borders(Borders::ALL).title(" 📜 Log ");
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn render_rest(f: &mut Frame, area: Rect, rs: &RestSiteState) {
    let heal = (rs.player.max_hp.0 * 30 / 100).max(1);
    let healed_to = (rs.player.hp.0 + heal).min(rs.player.max_hp.0);

    let mut lines = vec![
        Line::raw(format!("❤️  HP: {}/{}", rs.player.hp.0, rs.player.max_hp.0)),
        Line::raw(""),
        Line::styled(
            format!("[rest] / [r]  ❤️‍🩹 Heal {} HP (to {})", heal, healed_to),
            Style::default().fg(Color::Green),
        ),
        Line::raw(""),
    ];

    let upgradeable: Vec<_> = rs.player.deck.iter().enumerate()
        .filter(|(_, c)| c.upgrade().is_some())
        .collect();
    if upgradeable.is_empty() {
        lines.push(Line::styled(
            "(no cards can be upgraded)",
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        lines.push(Line::raw("🃏 Upgrade a card with [upgrade N] / [u N]:"));
        for (i, card) in &upgradeable {
            lines.push(Line::raw(format!("  [{}]  ⬆️  {}", i + 1, card.name())));
        }
    }

    let block = Block::default().borders(Borders::ALL).title(" 🔥 Rest Site ");
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn render_card_reward(f: &mut Frame, area: Rect, cr: &CardRewardState) {
    let mut items: Vec<ListItem> = cr.options.iter().enumerate().map(|(i, card)| {
        let text = format!(
            "[{}]  {}{} ({}) — {}",
            i + 1,
            card_type_icon(card.card_type()),
            card.name(),
            card.card_cost().display(),
            card.description(),
        );
        ListItem::new(text)
    }).collect();
    items.push(ListItem::new(Line::styled(
        "[skip] / [s]  Take nothing",
        Style::default().fg(Color::DarkGray),
    )));

    if let Some(potion) = &cr.offered_potion {
        items.push(ListItem::new(Line::raw("")));
        items.push(ListItem::new(Line::styled(
            format!("🧪 Potion on the ground: {}", potion.name()),
            Style::default().fg(Color::Yellow),
        )));
        items.push(ListItem::new(Line::styled(
            "[discard N]  Drop potion slot N to pick it up",
            Style::default().fg(Color::Green),
        )));
    }

    let block = Block::default().borders(Borders::ALL).title(" ✨ Card Reward ");
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_shop(f: &mut Frame, area: Rect, shop: &ShopState) {
    let mut items: Vec<ListItem> = Vec::new();

    items.push(ListItem::new(Line::styled("Cards:", Style::default().add_modifier(Modifier::BOLD))));
    for (i, (card, purchased)) in shop.cards.iter().enumerate() {
        let text = if *purchased {
            format!("[{}]  {} — [sold]", i + 1, card.name())
        } else {
            format!(
                "[{}]  {} ({}) — {} — {}g",
                i + 1, card.name(), card.card_cost().display(), card.description(), CARD_PRICE,
            )
        };
        let style = if *purchased { Style::default().fg(Color::DarkGray) } else { Style::default() };
        items.push(ListItem::new(Line::styled(text, style)));
    }

    items.push(ListItem::new(Line::raw("")));
    items.push(ListItem::new(Line::styled("Relic:", Style::default().add_modifier(Modifier::BOLD))));
    match &shop.relic {
        Some((relic, true)) => items.push(ListItem::new(Line::styled(
            format!("[r]  {} — [sold]", relic.name()), Style::default().fg(Color::DarkGray),
        ))),
        Some((relic, false)) => items.push(ListItem::new(Line::raw(
            format!("[r]  {} — {}g", relic.name(), RELIC_PRICE),
        ))),
        None => items.push(ListItem::new(Line::styled("  (none)", Style::default().fg(Color::DarkGray)))),
    }

    items.push(ListItem::new(Line::raw("")));
    items.push(ListItem::new(Line::styled("Potion:", Style::default().add_modifier(Modifier::BOLD))));
    match &shop.potion {
        Some((potion, true)) => items.push(ListItem::new(Line::styled(
            format!("[p]  {} — [sold]", potion.name()), Style::default().fg(Color::DarkGray),
        ))),
        Some((potion, false)) => items.push(ListItem::new(Line::raw(
            format!("[p]  {} — {}g", potion.name(), POTION_PRICE),
        ))),
        None => items.push(ListItem::new(Line::styled("  (none)", Style::default().fg(Color::DarkGray)))),
    }

    items.push(ListItem::new(Line::raw("")));
    items.push(ListItem::new(Line::styled(
        "[leave] / [l]  Exit shop",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default().borders(Borders::ALL)
        .title(format!(" 🛒 Shop  —  🪙 {}g ", shop.player.gold));
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_event_room(f: &mut Frame, area: Rect, er: &EventRoomState) {
    let (title, options): (&str, Vec<&str>) = match er.event {
        EventKind::Ssssserpent => (
            "🐍 The Ssssserpent",
            vec![
                "[1] Agree — Gain 150 gold. Become Cursed — Doubt.",
                "[2] Disagree — Nothing happens.",
                "[3] Leave",
            ],
        ),
        EventKind::BigFish => (
            "🐟 Big Fish",
            vec![
                "[1] Banana — Heal ~30% of max HP.",
                "[2] Donut — Gain 3 Max HP.",
                "[3] Box — Obtain a random Relic. Become Cursed — Regret.",
                "[4] Leave",
            ],
        ),
        EventKind::Mushrooms => (
            "🍄 Mushrooms",
            vec![
                "[1] Eat — Heal 12 HP. Become Cursed — Parasite.",
                "[2] Leave",
            ],
        ),
        EventKind::GoldenIdol => (
            "🏺 Golden Idol",
            vec![
                "[1] Outrun — Gain 250 gold. Become Cursed — Injury.",
                "[2] Smash — Gain 250 gold. Take 25 damage.",
                "[3] Hide — Gain 250 gold. Lose 6 Max HP.",
                "[4] Leave",
            ],
        ),
    };
    let items: Vec<ListItem> = std::iter::once(ListItem::new(title))
        .chain(std::iter::once(ListItem::new("")))
        .chain(options.into_iter().map(ListItem::new))
        .collect();
    let block = ratatui::widgets::Block::default().borders(Borders::ALL).title("Event");
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_game_over(f: &mut Frame, area: Rect, victory: bool) {
    let (msg, color) = if victory {
        ("🏆 You conquered the Spire!", Color::Green)
    } else {
        ("💀 You have been slain.", Color::Red)
    };

    // Center vertically: pad with empty lines above
    let inner_height = area.height.saturating_sub(2) as usize;
    let pad = inner_height.saturating_sub(3) / 2;
    let mut padded: Vec<Line> = vec![Line::raw(""); pad];
    padded.push(Line::styled(msg, Style::default().fg(color).add_modifier(Modifier::BOLD)));
    padded.push(Line::raw(""));
    padded.push(Line::styled("Press any key to quit.", Style::default().fg(Color::DarkGray)));
    let para = Paragraph::new(padded)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(para, area);
}

fn render_status_line(f: &mut Frame, area: Rect, tui: &TuiState) {
    let (text, style) = if let Some(err) = &tui.last_error {
        (err.clone(), Style::default().fg(Color::Red))
    } else if let Some(last) = tui.event_log.back() {
        (last.clone(), Style::default().fg(Color::Cyan))
    } else {
        (String::new(), Style::default())
    };
    let para = Paragraph::new(text).style(style);
    f.render_widget(para, area);
}

fn help_lines(state: &GameState) -> Vec<String> {
    match state {
        GameState::Combat { state: cs, .. } if cs.phase == CombatPhase::PlayerTurn => {
            let mut lines = vec![
                "N / play N      play card N from your hand".to_string(),
                "N T / play N T  play card N targeting enemy T".to_string(),
                "end / e         end your turn".to_string(),
                "use N           use potion N (targets first enemy)".to_string(),
                "use N T         use potion N on enemy T".to_string(),
                "↑ / W           scroll hand up".to_string(),
                "↓ / S           scroll hand down".to_string(),
            ];
            lines.push("relics          view your relics".to_string());
            for (key, _, desc) in PILE_KEYS {
                lines.push(format!("{key:<16}{desc}"));
            }
            lines
        }
        GameState::Combat { .. } => vec![],
        GameState::Map(_) => vec![
            "N               choose node N".to_string(),
            "↑ / W           scroll map up".to_string(),
            "↓ / S           scroll map down".to_string(),
            "d               view full deck".to_string(),
            "relics          view your relics".to_string(),
        ],
        GameState::RestSite(_) => vec![
            "rest            heal 30% of max HP".to_string(),
            "upgrade N       upgrade card N in your deck".to_string(),
            "d               view full deck".to_string(),
            "relics          view your relics".to_string(),
        ],
        GameState::TreasureRoom(_) => vec![
            "take            take the relic".to_string(),
            "skip            leave without taking".to_string(),
            "d               view full deck".to_string(),
            "relics          view your relics".to_string(),
        ],
        GameState::CardReward(_) => vec![
            "N               add card N to your deck".to_string(),
            "skip            skip the reward".to_string(),
            "d               view full deck".to_string(),
            "relics          view your relics".to_string(),
        ],
        GameState::Shop(_) => vec![
            "N               buy card N".to_string(),
            "r               buy the relic".to_string(),
            "p               buy the potion".to_string(),
            "leave           leave the shop".to_string(),
            "d               view full deck".to_string(),
            "relics          view your relics".to_string(),
        ],
        GameState::EventRoom(_) => vec![
            "N               choose option N".to_string(),
            "d               view full deck".to_string(),
            "relics          view your relics".to_string(),
        ],
        GameState::GameOver { .. } => vec![],
        GameState::Neow(_) => vec![
            "N               choose blessing N".to_string(),
        ],
    }
}


fn render_input(f: &mut Frame, area: Rect, tui: &TuiState) {
    let prompt = format!("> {}", tui.input_buf);
    let title = match tui.game {
        GameState::GameOver { .. } => " Command ",
        _ => " Command [? help] ",
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let para = Paragraph::new(prompt).block(block);
    f.render_widget(para, area);
}

fn render_pile_overlay(f: &mut Frame, area: Rect, state: &GameState, view: PileView) {
    let player = match state {
        GameState::Map(m)               => Some(&m.player),
        GameState::Combat { state, .. } => Some(&state.player),
        GameState::RestSite(rs)         => Some(&rs.player),
        GameState::TreasureRoom(tr)     => Some(&tr.player),
        GameState::CardReward(cr)       => Some(&cr.player),
        GameState::Shop(shop)           => Some(&shop.player),
        GameState::EventRoom(er)        => Some(&er.player),
        GameState::Neow(neow)           => Some(&neow.player),
        GameState::GameOver { .. }      => None,
    };
    let Some(player) = player else { return };

    let combat = match state {
        GameState::Combat { state, .. } => Some(state),
        _ => None,
    };

    let (label, pile): (&str, &[slay_core::Card]) = match view {
        PileView::Deck    => ("🃏 Deck",          &player.deck),
        PileView::Draw    => ("🎴 Draw pile",    combat.map_or(&[][..], |cs| &cs.player.draw_pile)),
        PileView::Discard => ("🗑️  Discard pile", combat.map_or(&[][..], |cs| &cs.player.discard_pile)),
        PileView::Exhaust => ("🔥 Exhaust pile", combat.map_or(&[][..], |cs| &cs.player.exhaust_pile)),
    };
    let title = format!(" {label} ({}) ", pile.len());

    let lines: Vec<Line> = if pile.is_empty() {
        vec![Line::styled("(empty)", Style::default().fg(Color::DarkGray))]
    } else {
        pile.iter().map(|c| Line::raw(format!(" • {}", c.name()))).collect()
    };

    // Centered popup — 60% width, 60% height
    let w = (area.width * 6) / 10;
    let h = (area.height * 6) / 10;
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let popup = Rect { x, y, width: w, height: h };

    f.render_widget(ratatui::widgets::Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, popup);
}

fn render_relic_overlay(f: &mut Frame, area: Rect, relics: &[Relic]) {
    let title = format!(" 🎒 Relics ({}) ", relics.len());
    let lines: Vec<Line> = if relics.is_empty() {
        vec![Line::styled("(no relics)", Style::default().fg(Color::DarkGray))]
    } else {
        relics.iter().map(|r| {
            Line::raw(format!(" {} {}  —  {}", relic_emoji(r), r.name(), r.description()))
        }).collect()
    };

    let w = (area.width * 7) / 10;
    let h = (area.height * 6) / 10;
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let popup = Rect { x, y, width: w, height: h };

    f.render_widget(ratatui::widgets::Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black));
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, popup);
}

fn wipe_title_lines(state: &GameState) -> Vec<Line<'static>> {
    let divider = Line::styled(
        "━━━━━━━━━━━━━━━━━━━━━━━━",
        Style::default().fg(Color::DarkGray),
    );
    match state {
        GameState::Combat { floor, is_boss, is_elite, state: cs, .. } => {
            let floor_line = Line::styled(
                format!("Floor {}", floor + 1),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            );
            let label = if *is_boss { "BOSS BATTLE" } else if *is_elite { "ELITE" } else { "COMBAT" };
            let room_line = Line::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
            let enemies: String = cs.enemies.iter().map(|e| e.name()).collect::<Vec<_>>().join("  ·  ");
            let enemy_line = Line::styled(enemies, Style::default().fg(Color::Gray));
            vec![floor_line, divider, room_line, enemy_line]
        }
        GameState::RestSite(rs) => vec![
            Line::styled(format!("Floor {}", rs.floor + 1), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            divider,
            Line::styled("REST SITE", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ],
        GameState::Shop(s) => vec![
            Line::styled(format!("Floor {}", s.floor + 1), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            divider,
            Line::styled("MERCHANT", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ],
        GameState::CardReward(cr) => vec![
            Line::styled(format!("Floor {}", cr.floor), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            divider,
            Line::styled("CARD REWARD", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ],
        GameState::TreasureRoom(tr) => vec![
            Line::styled(format!("Floor {}", tr.floor + 1), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            divider,
            Line::styled("TREASURE", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ],
        GameState::EventRoom(er) => {
            let event_name = match er.event {
                EventKind::Ssssserpent => "THE SSSSSERPENT",
                EventKind::BigFish     => "BIG FISH",
                EventKind::Mushrooms   => "MUSHROOMS",
                EventKind::GoldenIdol  => "GOLDEN IDOL",
            };
            vec![
                Line::styled(format!("Floor {}", er.floor + 1), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                divider,
                Line::styled("EVENT", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Line::styled(event_name, Style::default().fg(Color::Gray)),
            ]
        }
        GameState::Map(_) | GameState::GameOver { .. } | GameState::Neow(_) => vec![],
    }
}

fn render_wipe_overlay(f: &mut Frame, area: Rect, state: &GameState) {
    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(Block::default().style(Style::default().bg(Color::Black)), area);

    let lines = wipe_title_lines(state);
    if lines.is_empty() { return; }

    let content_height = lines.len() as u16;
    let y_pad = area.height.saturating_sub(content_height) / 2;
    let card_area = Rect {
        x: area.x,
        y: area.y + y_pad,
        width: area.width,
        height: content_height,
    };
    let para = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black));
    f.render_widget(para, card_area);
}

fn render_help_overlay(f: &mut Frame, area: Rect, state: &GameState) {
    let lines: Vec<Line> = help_lines(state)
        .into_iter()
        .map(Line::raw)
        .collect();

    let w = (area.width * 7) / 10;
    let h = (area.height * 7) / 10;
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let popup = Rect { x, y, width: w, height: h };

    f.render_widget(ratatui::widgets::Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help — esc to close ")
        .style(Style::default().bg(Color::Black));
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, popup);
}

/// Processes one key, updating TuiState in place.
/// Returns `false` if the user requested quit (Esc / Ctrl-C); `true` otherwise.
pub fn handle_key(tui: &mut TuiState, rng: &mut AnyRng, key: crate::key::Key) -> bool {
    use crate::key::Key;

    if tui.wipe_start.is_some() {
        tui.wipe_start = None;
        return true;
    }
    if tui.show_pile.is_some() {
        tui.show_pile = None;
        return true;
    }
    if tui.show_relics {
        match key {
            Key::Esc | Key::Enter | Key::Char('r') => tui.show_relics = false,
            _ => {}
        }
        return true;
    }
    if tui.show_help {
        match key {
            Key::Esc | Key::Enter | Key::Char('?') => tui.show_help = false,
            _ => {}
        }
        return true;
    }

    match key {
        Key::Esc | Key::CtrlC => { tui.should_quit = true; false }
        Key::Char('?') => { tui.show_help = true; true }
        Key::Char('w') | Key::Char('W') if tui.input_buf.is_empty() => {
            if matches!(&tui.game, GameState::Map(_)) {
                tui.map_scroll += 1;
            } else {
                tui.hand_scroll = tui.hand_scroll.saturating_sub(1);
            }
            true
        }
        Key::Char('s') | Key::Char('S') if tui.input_buf.is_empty() => {
            if matches!(&tui.game, GameState::Map(_)) {
                tui.map_scroll = tui.map_scroll.saturating_sub(1);
            } else if let GameState::Combat { state: cs, .. } = &tui.game {
                let max = cs.player.hand.len().saturating_sub(1);
                tui.hand_scroll = (tui.hand_scroll + 1).min(max);
            }
            true
        }
        Key::Char(c) => { tui.input_buf.push(c); true }
        Key::Backspace => { tui.input_buf.pop(); true }
        Key::Enter => { tui.handle_enter(rng); true }
        Key::Up => {
            if matches!(&tui.game, GameState::Map(_)) {
                tui.map_scroll += 1;
            } else {
                tui.hand_scroll = tui.hand_scroll.saturating_sub(1);
            }
            true
        }
        Key::Down => {
            if matches!(&tui.game, GameState::Map(_)) {
                tui.map_scroll = tui.map_scroll.saturating_sub(1);
            } else if let GameState::Combat { state: cs, .. } = &tui.game {
                let max = cs.player.hand.len().saturating_sub(1);
                tui.hand_scroll = (tui.hand_scroll + 1).min(max);
            }
            true
        }
        Key::Other => true,
    }
}

// Public entry point: take over the terminal and run the ratatui event loop.
#[cfg(all(not(test), feature = "terminal"))]
pub fn run_tui(
    state: GameState,
    rng: &mut AnyRng,
    debug: bool,
    save_tx: Option<std::sync::mpsc::SyncSender<Option<(GameState, u64)>>>,
) -> std::io::Result<()> {
    use crossterm::{
        event::{self, Event as CxEvent, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
    };
    use ratatui::backend::CrosstermBackend;
    use std::io;
    use std::time::Duration;

    // Panic safety: restore terminal even on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut tui = TuiState::new_with_save(state, debug, save_tx);

    let result = (|| -> std::io::Result<()> {
        loop {
            terminal.draw(|f| render_frame(f, &mut tui))?;

            if tui.should_quit {
                // Wait for any key, then exit
                if event::poll(Duration::from_millis(5000))? {
                    let _ = event::read()?;
                }
                break;
            }

            if tui.wipe_start.map(|t| t.elapsed() >= WIPE_DURATION).unwrap_or(false) {
                tui.wipe_start = None;
            }
            if tui.player_flash.map(|t| t.elapsed() >= FLASH_DURATION).unwrap_or(false) {
                tui.player_flash = None;
            }
            for flash in &mut tui.enemy_flashes {
                if flash.map(|t| t.elapsed() >= FLASH_DURATION).unwrap_or(false) {
                    *flash = None;
                }
            }

            if !event::poll(Duration::from_millis(100))? {
                continue;
            }
            let CxEvent::Key(cx_key) = event::read()? else { continue };
            if cx_key.kind != KeyEventKind::Press { continue; }
            let key = {
                use crate::key::Key;
                match cx_key.code {
                    KeyCode::Char('c') if cx_key.modifiers.contains(KeyModifiers::CONTROL) => Key::CtrlC,
                    KeyCode::Char(c) => Key::Char(c),
                    KeyCode::Enter => Key::Enter,
                    KeyCode::Backspace => Key::Backspace,
                    KeyCode::Esc => Key::Esc,
                    KeyCode::Up => Key::Up,
                    KeyCode::Down => Key::Down,
                    _ => Key::Other,
                }
            };
            if !handle_key(&mut tui, rng, key) {
                break;
            }
        }
        Ok(())
    })();

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();
    if let Some(tx) = &tui.save_tx {
        let _ = tx.send(None);
    }
    result
}

// Convenience for tests: render to a TestBackend and return the buffer as a String.
#[cfg(test)]
pub(crate) fn render_to_string(tui: &mut TuiState, width: u16, height: u16) -> String {
    use ratatui::backend::TestBackend;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render_frame(f, tui)).unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use slay_core::{
        AnyRng, Card, Command, EnemyKind, Grade, MapNode, MapState, NeowContext, NoOpRng,
        new_run, new_simple_run,
    };

    fn rng() -> AnyRng { AnyRng::NoOp(NoOpRng) }
    fn seeded_rng(seed: u64) -> AnyRng { AnyRng::seeded(seed) }

    fn make_combat_tui() -> TuiState {
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        TuiState::new(state, false)
    }

    fn make_map_tui() -> TuiState {
        let state = new_simple_run();
        TuiState::new(state, false)
    }

    fn make_map_tui_with_node(node: MapNode) -> TuiState {
        use slay_core::{Block, Energy, Hp, MapGraph, Player, Scenario, StatusMap};
        let state = GameState::Map(MapState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 99, relics: vec![], potions: vec![],
                neow_lament_combats_remaining: 0, reached_boss: false, potion_chance: 0.40,
            },
            floor: 0,
            graph: MapGraph { rows: vec![vec![node.clone()]], edges: vec![vec![vec![]]], path: Vec::new() },
            available_cols: vec![0],
            next_enemies: None,
            scenario: Scenario::Main,
        });
        TuiState::new(state, false)
    }

    fn make_main_run_map_tui() -> TuiState {
        let mut r = seeded_rng(1);
        let ctx = NeowContext::default();
        let state = new_run(&mut r, &ctx);
        let (state, _) = apply_and_drain(state, Command::ChooseNeowBlessing(0), &mut r).unwrap();
        TuiState::new(state, false)
    }

    // ─── render_frame ─────────────────────────────────────────────

    #[test]
    fn combat_screen_shows_enemy_name() {
        let mut tui = make_combat_tui();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Louse"), "expected 'Louse' in:\n{out}");
    }

    #[test]
    fn combat_screen_shows_player_hp_in_top_bar() {
        let mut tui = make_combat_tui();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("80/80"), "expected '80/80' in:\n{out}");
    }

    #[test]
    fn top_bar_hp_bar_reflects_current_health() {
        use slay_core::{Block, Energy, Hp, MapState, Player, Scenario, StatusMap};
        let mut r = rng();
        let graph = slay_core::generate_map(&slay_core::MapConfig::default(), &mut r);
        let state = GameState::Map(MapState {
            player: Player {
                hp: Hp(40), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0, relics: vec![], potions: vec![],
                neow_lament_combats_remaining: 0, reached_boss: false, potion_chance: 0.40,
            },
            floor: 0, graph, available_cols: vec![0], next_enemies: None,
            scenario: Scenario::Main,
        });
        let mut tmp = TuiState::new(state, false);
        let out = render_to_string(&mut tmp, 120, 30);
        // 40/80 with width 20 → 10 filled, 10 empty
        assert!(out.contains("[██████████░░░░░░░░░░]"), "expected half-full bar in:\n{out}");
    }

    #[test]
    fn hp_bar_at_1_hp_is_not_fully_empty() {
        // width 20 = player bar, width 10 = enemy bar
        assert!(hp_bar(1, 80, 20).contains('█'), "player bar at 1/80 should show at least one filled block");
        assert!(hp_bar(1, 20, 10).contains('█'), "enemy bar at 1/20 should show at least one filled block");
    }

    #[test]
    fn combat_screen_shows_pile_counts() {
        let mut tui = make_combat_tui();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Draw:"), "expected 'Draw:' in:\n{out}");
        assert!(out.contains("Discard:"), "expected 'Discard:' in:\n{out}");
    }

    #[test]
    fn combat_screen_shows_log_panel_title() {
        let mut tui = make_combat_tui();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Log"), "expected 'Log' in:\n{out}");
    }

    #[test]
    fn map_screen_shows_node_icons() {
        // Use a tall terminal so all 15 floors fit without scrolling.
        // 15 floors × 3 lines + 4 boss lines + 2 borders + 9 fixed rows = ~60 minimum.
        let mut tui = make_main_run_map_tui();
        let out = render_to_string(&mut tui,100, 65);
        assert!(out.contains("⚔️"), "expected ⚔️ icon in:\n{out}");
        assert!(out.contains("💀"), "expected 💀 boss icon in:\n{out}");
    }

    #[test]
    fn map_scroll_up_reveals_boss_icon() {
        use crate::key::Key;
        let mut rng = seeded_rng(1);
        let mut tui = make_main_run_map_tui();
        for _ in 0..20 {
            handle_key(&mut tui, &mut rng, Key::Up);
        }
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("💀"), "expected 💀 boss icon after scrolling up in:\n{out}");
    }

    #[test]
    fn map_scroll_w_key_same_as_up() {
        use crate::key::Key;
        let mut rng = seeded_rng(1);
        let mut tui_up = make_main_run_map_tui();
        let mut tui_w = make_main_run_map_tui();
        for _ in 0..20 {
            handle_key(&mut tui_up, &mut rng, Key::Up);
            handle_key(&mut tui_w, &mut rng, Key::Char('w'));
        }
        assert_eq!(tui_up.map_scroll, tui_w.map_scroll);
    }

    #[test]
    fn map_scroll_s_key_decrements_after_scrolling_up() {
        use crate::key::Key;
        let mut rng = seeded_rng(1);
        let mut tui = make_main_run_map_tui();
        for _ in 0..5 {
            handle_key(&mut tui, &mut rng, Key::Up);
        }
        let scroll_after_up = tui.map_scroll;
        handle_key(&mut tui, &mut rng, Key::Char('s'));
        assert_eq!(tui.map_scroll, scroll_after_up - 1);
    }

    #[test]
    fn map_scroll_down_key_decrements_after_scrolling_up() {
        use crate::key::Key;
        let mut rng = seeded_rng(1);
        let mut tui = make_main_run_map_tui();
        for _ in 0..5 {
            handle_key(&mut tui, &mut rng, Key::Up);
        }
        let scroll_after_up = tui.map_scroll;
        handle_key(&mut tui, &mut rng, Key::Down);
        assert_eq!(tui.map_scroll, scroll_after_up - 1);
    }

    #[test]
    fn map_scroll_down_does_not_underflow_at_zero() {
        use crate::key::Key;
        let mut rng = seeded_rng(1);
        let mut tui = make_main_run_map_tui();
        handle_key(&mut tui, &mut rng, Key::Down);
        assert_eq!(tui.map_scroll, 0, "scrolling down at zero should stay at zero");
    }

    #[test]
    fn map_scroll_clamped_after_render_so_s_always_moves_view() {
        use crate::key::Key;
        let mut rng = seeded_rng(1);
        let mut tui = make_main_run_map_tui();
        // Scroll up far beyond the effective max — raw value will be too high.
        for _ in 0..100 {
            handle_key(&mut tui, &mut rng, Key::Up);
        }
        // Render into a 100×30 frame; this should write back the clamped scroll.
        let out_before = render_to_string(&mut tui, 100, 30);
        // Now S should decrement by 1 and visibly change the view.
        handle_key(&mut tui, &mut rng, Key::Down);
        let out_after = render_to_string(&mut tui, 100, 30);
        assert_ne!(out_before, out_after, "S should move the view after render has clamped map_scroll");
    }

    #[test]
    fn w_s_do_not_scroll_when_input_buf_nonempty() {
        use crate::key::Key;
        let mut rng = seeded_rng(1);
        let mut tui = make_main_run_map_tui();
        for _ in 0..5 {
            handle_key(&mut tui, &mut rng, Key::Up);
        }
        let scroll_before = tui.map_scroll;
        tui.input_buf = "1".to_string();
        handle_key(&mut tui, &mut rng, Key::Char('w'));
        handle_key(&mut tui, &mut rng, Key::Char('s'));
        assert_eq!(tui.map_scroll, scroll_before, "w/s should not scroll when input_buf is non-empty");
    }

    #[test]
    fn combat_w_key_scrolls_hand_up() {
        use crate::key::Key;
        let mut rng = rng();
        let mut tui = make_combat_tui_with_many_cards();
        tui.hand_scroll = 3;
        handle_key(&mut tui, &mut rng, Key::Char('w'));
        assert_eq!(tui.hand_scroll, 2);
    }

    #[test]
    fn combat_s_key_scrolls_hand_down() {
        use crate::key::Key;
        let mut rng = rng();
        let mut tui = make_combat_tui_with_many_cards();
        tui.hand_scroll = 0;
        handle_key(&mut tui, &mut rng, Key::Char('s'));
        assert_eq!(tui.hand_scroll, 1);
    }

    #[test]
    fn combat_up_key_scrolls_hand_up() {
        use crate::key::Key;
        let mut rng = rng();
        let mut tui = make_combat_tui_with_many_cards();
        tui.hand_scroll = 3;
        handle_key(&mut tui, &mut rng, Key::Up);
        assert_eq!(tui.hand_scroll, 2);
    }

    #[test]
    fn combat_down_key_scrolls_hand_down() {
        use crate::key::Key;
        let mut rng = rng();
        let mut tui = make_combat_tui_with_many_cards();
        tui.hand_scroll = 0;
        handle_key(&mut tui, &mut rng, Key::Down);
        assert_eq!(tui.hand_scroll, 1);
    }

    #[test]
    fn map_screen_choices_panel_shows_node_name() {
        let mut tui = make_map_tui();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Choose"), "expected 'Choose' panel in:\n{out}");
        assert!(out.contains("[Enter]") || out.contains("[1]"), "expected choices in:\n{out}");
    }

    #[test]
    fn map_screen_shows_available_choices() {
        let mut tui = make_main_run_map_tui();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("[1]"), "expected '[1]' choice in:\n{out}");
        assert!(out.contains("[2]"), "expected '[2]' choice in:\n{out}");
    }

    #[test]
    fn map_grid_shows_both_node_icons_on_two_column_floor() {
        let graph = slay_core::MapGraph {
            rows: vec![vec![MapNode::Combat(vec![]), MapNode::RestSite]],
            edges: vec![vec![vec![], vec![]]],
            path: Vec::new(),
        };
        use slay_core::{Block, Energy, Hp, Player, Scenario, StatusMap};
        let state = GameState::Map(MapState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0, relics: vec![], potions: vec![],
                neow_lament_combats_remaining: 0, reached_boss: false, potion_chance: 0.40,
            },
            floor: 0,
            graph,
            available_cols: vec![0, 1],
            next_enemies: None,
            scenario: Scenario::Main,
        });
        let mut tui = TuiState::new(state, false);
        let out = render_to_string(&mut tui,120, 30);
        assert!(out.contains("⚔️"), "expected ⚔️ combat icon in:\n{out}");
        assert!(out.contains("🔥"), "expected 🔥 rest site icon in:\n{out}");
        let choices = out.lines().find(|l| l.contains("Combat")).expect("expected 'Combat' in choices panel");
        assert!(choices.contains("Rest Site"), "expected both choices shown in:\n{choices}");
    }

    #[test]
    fn rest_site_screen_shows_heal_amount() {
        let mut r = AnyRng::NoOp(NoOpRng);
        let ctx = NeowContext::default();
        let state = new_run(&mut r, &ctx);
        let (state, _) = apply_and_drain(state, Command::ChooseNeowBlessing(0), &mut r).unwrap();
        // Force into rest site by hand
        let player = match state {
            GameState::Map(m) => m.player,
            _ => panic!(),
        };
        let rs_state = GameState::RestSite(slay_core::RestSiteState {
            player,
            floor: 3,
            graph: slay_core::generate_map(&slay_core::MapConfig::default(), &mut r),
            available_cols: vec![0, 1],
        });
        let mut tui = TuiState::new(rs_state, false);
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Rest Site"), "expected 'Rest Site' in:\n{out}");
        assert!(out.contains("Heal"), "expected 'Heal' in:\n{out}");
    }

    #[test]
    fn card_reward_screen_shows_card_options() {
        let mut r = AnyRng::NoOp(NoOpRng);
        let ctx = NeowContext::default();
        let state = new_run(&mut r, &ctx);
        let (state, _) = apply_and_drain(state, Command::ChooseNeowBlessing(0), &mut r).unwrap();
        let player = match state {
            GameState::Map(m) => m.player,
            _ => panic!(),
        };
        let cr = GameState::CardReward(slay_core::CardRewardState {
            player,
            floor: 1,
            options: vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base), Card::Bash(Grade::Base)],
            offered_potion: None,
            graph: slay_core::generate_map(&slay_core::MapConfig::default(), &mut r),
            available_cols: vec![0, 1],
        });
        let mut tui = TuiState::new(cr, false);
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Card Reward"), "expected 'Card Reward' in:\n{out}");
        assert!(out.contains("Strike"), "expected 'Strike' in:\n{out}");
        assert!(out.contains("skip"), "expected 'skip' option in:\n{out}");
    }

    #[test]
    fn game_over_victory_screen_shows_message() {
        let mut tui = TuiState::new(GameState::GameOver { victory: true }, false);
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("conquered"), "expected 'conquered' in:\n{out}");
    }

    #[test]
    fn game_over_defeat_screen_shows_message() {
        let mut tui = TuiState::new(GameState::GameOver { victory: false }, false);
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("slain"), "expected 'slain' in:\n{out}");
    }

    #[test]
    fn input_box_shows_typed_characters() {
        let mut tui = make_combat_tui();
        tui.input_buf = "play 1".to_string();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("> play 1"), "expected '> play 1' in:\n{out}");
    }

    #[test]
    fn relics_command_opens_relic_overlay() {
        use slay_core::{Block, Energy, Hp, MapState, Player, Relic, Scenario, StatusMap};
        let mut r = rng();
        let graph = slay_core::generate_map(&slay_core::MapConfig::default(), &mut r);
        let state = GameState::Map(MapState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0,
                relics: vec![Relic::Anchor],
                potions: vec![],
                neow_lament_combats_remaining: 0, reached_boss: false, potion_chance: 0.40,
            },
            floor: 0, graph, available_cols: vec![0], next_enemies: None,
            scenario: Scenario::Main,
        });
        let mut tui = TuiState::new(state, false);
        let mut r = rng();
        tui.input_buf = "relics".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.show_relics, "show_relics should be true after 'relics' command");
        let out = render_to_string(&mut tui,120, 30);
        assert!(out.contains("Anchor"), "expected Anchor name in relic overlay:\n{out}");
        assert!(out.contains("10 Block"), "expected Anchor description in relic overlay:\n{out}");
    }

    #[test]
    fn top_bar_shows_relic_emoji_when_player_has_relics() {
        use slay_core::{Block, Energy, Hp, MapState, Player, Relic, Scenario, StatusMap};
        let mut r = rng();
        let graph = slay_core::generate_map(&slay_core::MapConfig::default(), &mut r);
        let state = GameState::Map(MapState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0,
                relics: vec![Relic::Anchor, Relic::BurningBlood],
                potions: vec![],
                neow_lament_combats_remaining: 0, reached_boss: false, potion_chance: 0.40,
            },
            floor: 0, graph, available_cols: vec![0], next_enemies: None,
            scenario: Scenario::Main,
        });
        let mut tmp = TuiState::new(state, false);
        let out = render_to_string(&mut tmp, 120, 30);
        assert!(out.contains("⚓"), "expected Anchor emoji in:\n{out}");
        assert!(out.contains("🔥"), "expected BurningBlood emoji in:\n{out}");
    }

    #[test]
    fn top_bar_omits_relic_row_when_no_relics() {
        use slay_core::{Block, Energy, Hp, MapState, Player, Scenario, StatusMap};
        let mut r = rng();
        let graph = slay_core::generate_map(&slay_core::MapConfig::default(), &mut r);
        let state = GameState::Map(MapState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0, relics: vec![], potions: vec![],
                neow_lament_combats_remaining: 0, reached_boss: false, potion_chance: 0.40,
            },
            floor: 0, graph, available_cols: vec![0], next_enemies: None,
            scenario: Scenario::Main,
        });
        // Height 6 = 1 top bar + 1 main + 1 status + 3 input.
        // Row index 1 (second row, 0-indexed) is the first row of main area — should not be relic bar.
        let mut tmp = TuiState::new(state, false);
        let out = render_to_string(&mut tmp, 120, 6);
        let second_row = out.lines().nth(1).unwrap_or("");
        assert!(!second_row.contains("⚓"), "relic bar should not appear when no relics: {second_row}");
    }

    #[test]
    #[ignore = "visual check; run with --ignored to dump"]
    fn visual_dump_combat_screen() {
        // Build a realistic combat: spawn a Louse, enter combat, add some cards to hand
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::AddCard(Card::Strike(Grade::Base)), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::AddCard(Card::Defend(Grade::Base)), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::AddCard(Card::Bash(Grade::Base)), &mut r).unwrap().0;

        let mut tui = TuiState::new(state, true);
        tui.push_log("─── Turn 1 ───".to_string());
        tui.push_log("👁  Enemy prepares: ⚔️ Attack 8.".to_string());
        tui.input_buf = "play 1".to_string();

        eprintln!("{}", render_to_string(&mut tui,100, 30));
    }

    // ─── hp_color ─────────────────────────────────────────────────

    #[test]
    fn hp_color_full_is_white() {
        assert_eq!(hp_color(80, 80), Color::White);
    }

    #[test]
    fn hp_color_half_is_yellow() {
        assert_eq!(hp_color(40, 80), Color::Yellow);
    }

    #[test]
    fn hp_color_just_above_half_is_white() {
        assert_eq!(hp_color(41, 80), Color::White);
    }

    #[test]
    fn hp_color_low_is_red() {
        assert_eq!(hp_color(8, 80), Color::Red);
    }

    #[test]
    fn hp_color_exactly_twenty_percent_is_red() {
        assert_eq!(hp_color(16, 80), Color::Red);
    }

    #[test]
    fn hp_color_just_above_twenty_percent_is_yellow() {
        assert_eq!(hp_color(17, 80), Color::Yellow);
    }

    // ─── phase banners ────────────────────────────────────────────

    #[test]
    fn entering_combat_pushes_banner_with_enemy_names() {
        let mut tui = make_map_tui();
        let mut r = rng();
        tui.input_buf = "spawn red-louse".to_string();
        tui.handle_enter(&mut r);
        tui.input_buf = "1".to_string();
        tui.handle_enter(&mut r);
        let log: Vec<&str> = tui.event_log.iter().map(String::as_str).collect();
        assert!(
            log.iter().any(|l| l.contains("⚔️") && l.contains("Red Louse")),
            "expected combat banner with enemy name, log: {log:?}"
        );
    }

    #[test]
    fn phase_banner_combat_contains_floor_and_enemies() {
        let mut state = new_simple_run();
        let mut r = rng();
        let before = state.clone();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        let banner = phase_banner(&before, &state).expect("should produce a banner");
        assert!(banner.contains("⚔️"), "banner: {banner}");
        assert!(banner.contains("Red Louse"), "banner: {banner}");
    }

    #[test]
    fn phase_banner_returns_none_for_same_phase() {
        let state = new_simple_run();
        assert!(phase_banner(&state, &state.clone()).is_none());
    }

    // ─── handle_enter ─────────────────────────────────────────────

    #[test]
    fn handle_enter_unknown_command_sets_last_error() {
        let mut tui = make_combat_tui();
        let mut r = rng();
        tui.input_buf = "fireball".to_string();
        tui.handle_enter(&mut r);
        assert_eq!(tui.last_error.as_deref(), Some("Unknown command."));
        assert!(tui.input_buf.is_empty(), "buffer should be cleared");
    }

    #[test]
    fn handle_enter_clears_buffer_on_success() {
        let mut tui = make_combat_tui();
        let mut r = rng();
        tui.input_buf = "end".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.input_buf.is_empty(), "buffer should be cleared");
        assert!(tui.last_error.is_none(), "no error expected, got {:?}", tui.last_error);
    }

    #[test]
    fn handle_enter_command_error_sets_last_error() {
        // PlayCard on empty hand → InvalidCard
        let mut tui = make_combat_tui();
        let mut r = rng();
        tui.input_buf = "1".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.last_error.is_some(), "expected last_error to be set");
        let err = tui.last_error.as_deref().unwrap();
        assert!(err.contains("No card") || err.contains("card"), "unexpected error: {err}");
    }

    #[test]
    fn handle_enter_success_appends_events_to_log() {
        let mut tui = make_combat_tui();
        let mut r = rng();
        let log_before = tui.event_log.len();
        tui.input_buf = "end".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.event_log.len() > log_before, "expected log to grow");
    }

    #[test]
    fn handle_enter_combat_pile_shortcut_z_sets_show_pile() {
        let mut tui = make_combat_tui();
        let mut r = rng();
        tui.input_buf = "z".to_string();
        tui.handle_enter(&mut r);
        assert_eq!(tui.show_pile, Some(PileView::Draw));
    }

    #[test]
    fn handle_enter_d_in_map_shows_deck_overlay() {
        let mut tui = make_map_tui();
        let mut r = rng();
        tui.input_buf = "d".to_string();
        tui.handle_enter(&mut r);
        assert_eq!(tui.show_pile, Some(PileView::Deck));
    }

    #[test]
    fn handle_enter_d_in_combat_shows_deck_overlay() {
        let mut tui = make_combat_tui();
        let mut r = rng();
        tui.input_buf = "d".to_string();
        tui.handle_enter(&mut r);
        assert_eq!(tui.show_pile, Some(PileView::Deck));
    }

    #[test]
    fn handle_enter_z_in_combat_still_shows_draw_pile() {
        let mut tui = make_combat_tui();
        let mut r = rng();
        tui.input_buf = "z".to_string();
        tui.handle_enter(&mut r);
        assert_eq!(tui.show_pile, Some(PileView::Draw));
    }

    #[test]
    fn deck_overlay_renders_card_names_from_player_deck() {
        use slay_core::Card;
        let mut tui = make_map_tui();
        if let GameState::Map(ref mut m) = tui.game {
            m.player.deck = vec![Card::Strike(slay_core::Grade::Base), Card::Defend(slay_core::Grade::Base)];
        }
        tui.show_pile = Some(PileView::Deck);
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Strike"), "expected Strike in deck overlay:\n{out}");
        assert!(out.contains("Defend"), "expected Defend in deck overlay:\n{out}");
    }

    #[test]
    fn handle_enter_game_over_sets_should_quit() {
        // Build a combat about to lose: player at 1 HP, end turn lets enemy kill.
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        if let GameState::Combat { state: ref mut cs, .. } = state {
            cs.player.hp = slay_core::Hp(1);
        }
        let mut tui = TuiState::new(state, false);
        tui.input_buf = "end".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.should_quit, "expected should_quit after defeat");
    }

    // ─── log capacity ─────────────────────────────────────────────

    #[test]
    fn push_log_caps_at_capacity() {
        let mut tui = TuiState::new(new_simple_run(), false);
        for i in 0..(LOG_CAPACITY + 50) {
            tui.push_log(format!("line {}", i));
        }
        assert_eq!(tui.event_log.len(), LOG_CAPACITY);
        // Oldest line should be "line 50"
        assert_eq!(tui.event_log.front().unwrap(), "line 50");
    }

    #[test]
    fn push_log_ignores_empty_strings() {
        let mut tui = TuiState::new(new_simple_run(), false);
        let before = tui.event_log.len();
        tui.push_log(String::new());
        assert_eq!(tui.event_log.len(), before);
    }

    // ─── show_help ─────────────────────────────────────────────────

    #[test]
    fn show_help_defaults_to_false() {
        let tui = make_map_tui();
        assert!(!tui.show_help);
    }

    // ─── PILE_KEYS ─────────────────────────────────────────────────

    #[test]
    fn pile_keys_table_matches_handle_enter_behaviour() {
        let mut tui = make_combat_tui();
        let mut r = rng();
        for (key, view, _desc) in PILE_KEYS {
            tui.input_buf = key.to_string();
            tui.handle_enter(&mut r);
            assert_eq!(tui.show_pile, Some(*view), "key '{key}' should open {view:?}");
        }
    }

    #[test]
    fn help_lines_combat_pile_entries_match_pile_keys() {
        let tui = make_combat_tui();
        let lines = help_lines(&tui.game);
        for (key, _view, desc) in PILE_KEYS {
            let expected = key.to_string();
            assert!(
                lines.iter().any(|l| l.starts_with(&expected) && l.contains(desc)),
                "help_lines should contain entry for key '{key}' with desc '{desc}'"
            );
        }
    }

    // ─── help_lines ────────────────────────────────────────────────

    #[test]
    fn help_lines_combat_contains_play_and_end_entries() {
        let tui = make_combat_tui();
        let lines = help_lines(&tui.game);
        assert!(lines.iter().any(|l| l.contains("play")), "should have play entry");
        assert!(lines.iter().any(|l| l.contains("end")), "should have end entry");
    }

    #[test]
    fn help_lines_map_is_non_empty() {
        let tui = make_map_tui();
        assert!(!help_lines(&tui.game).is_empty());
    }

    #[test]
    fn help_lines_game_over_is_empty() {
        let tui = TuiState::new(GameState::GameOver { victory: false }, false);
        assert!(help_lines(&tui.game).is_empty());
    }

    // ─── hand scroll ──────────────────────────────────────────────

    fn make_combat_tui_with_many_cards() -> TuiState {
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        for _ in 0..10 {
            state = apply_and_drain(state, Command::AddCard(Card::Strike(Grade::Base)), &mut r).unwrap().0;
        }
        TuiState::new(state, false)
    }

    #[test]
    fn hand_scroll_defaults_to_zero() {
        let tui = make_combat_tui();
        assert_eq!(tui.hand_scroll, 0);
    }

    #[test]
    fn handle_enter_resets_hand_scroll_on_success() {
        let mut tui = make_combat_tui_with_many_cards();
        tui.hand_scroll = 3;
        let mut r = rng();
        tui.input_buf = "end".to_string();
        tui.handle_enter(&mut r);
        assert_eq!(tui.hand_scroll, 0, "scroll should reset after a successful command");
    }

    #[test]
    fn render_hand_shows_above_hint_when_scrolled() {
        let mut tui = make_combat_tui_with_many_cards();
        tui.hand_scroll = 2;
        // Height 20 gives hand panel ~6 inner rows — 10 cards definitely overflows
        let out = render_to_string(&mut tui,100, 20);
        assert!(out.contains("above"), "should show 'above' hint when scrolled: {out}");
    }

    #[test]
    fn render_hand_shows_below_hint_when_cards_overflow() {
        let mut tui = make_combat_tui_with_many_cards();
        // Height 20, 10 cards — will overflow
        let out = render_to_string(&mut tui,100, 20);
        assert!(out.contains("more"), "should show 'more' hint when cards overflow: {out}");
    }

    #[test]
    fn render_hand_hides_first_card_when_scrolled_past_it() {
        let mut tui = make_combat_tui_with_many_cards();
        tui.hand_scroll = 0;
        let out_unscrolled = render_to_string(&mut tui,100, 20);
        tui.hand_scroll = 4;
        let out_scrolled = render_to_string(&mut tui,100, 20);
        // First card [1] visible when scroll=0, hidden when scroll=4
        assert!(out_unscrolled.contains("[1]"), "card 1 should be visible at scroll=0");
        assert!(!out_scrolled.contains("[1]"), "card 1 should be hidden at scroll=4");
        // Card [5] hidden when scroll=0 (overflows), visible when scroll=4
        assert!(out_scrolled.contains("[5]"), "card 5 should be visible at scroll=4");
    }

    // ─── damage flash ─────────────────────────────────────────────

    #[test]
    fn player_flash_defaults_to_none() {
        let tui = make_combat_tui();
        assert!(tui.player_flash.is_none());
    }

    #[test]
    fn enemy_flashes_empty_outside_combat() {
        let tui = make_map_tui();
        assert!(tui.enemy_flashes.is_empty());
    }

    #[test]
    fn enemy_flashes_initialized_for_each_enemy() {
        let tui = make_combat_tui();
        assert_eq!(tui.enemy_flashes.len(), 1);
        assert!(tui.enemy_flashes[0].is_none());
    }

    #[test]
    fn playing_damaging_card_sets_enemy_flash() {
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::AddCard(Card::Strike(Grade::Base)), &mut r).unwrap().0;
        let mut tui = TuiState::new(state, false);
        tui.input_buf = "1".to_string();
        tui.handle_enter(&mut r);
        assert!(
            tui.enemy_flashes.first().copied().flatten().is_some(),
            "enemy flash should be set after Strike deals damage"
        );
    }

    #[test]
    fn taking_damage_sets_player_flash() {
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        if let GameState::Combat { state: ref mut cs, .. } = state {
            cs.player.block = slay_core::Block(0);
        }
        let mut tui = TuiState::new(state, false);
        tui.input_buf = "end".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.player_flash.is_some(), "player flash should be set after enemy attacks");
    }

    #[test]
    fn enemy_flashes_reset_on_new_combat_entry() {
        let mut tui = make_combat_tui();
        tui.enemy_flashes = vec![now()];
        // Leave combat by letting the enemy die isn't easy; instead synthesise a new combat state
        let mut r = rng();
        let mut state = new_simple_run();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::RedLouse, EnemyKind::RedLouse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        tui.game = state;
        // Trigger any command that goes through handle_enter so flashes get re-initialised
        tui.input_buf = "end".to_string();
        tui.handle_enter(&mut r);
        assert_eq!(tui.enemy_flashes.len(), 2, "flash slots should match new enemy count");
    }

    // ─── wipe overlay ─────────────────────────────────────────────

    #[test]
    fn wipe_start_defaults_to_none() {
        let tui = make_combat_tui();
        assert!(tui.wipe_start.is_none());
    }

    #[test]
    fn entering_combat_from_map_sets_wipe_start() {
        let mut tui = make_map_tui();
        let mut r = rng();
        tui.input_buf = "spawn red-louse".to_string();
        tui.handle_enter(&mut r);
        tui.input_buf = "1".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.wipe_start.is_some(), "wipe_start should be set after Map → Combat transition");
    }

    #[test]
    fn entering_rest_site_from_map_sets_wipe_start() {
        let mut tui = make_map_tui_with_node(MapNode::RestSite);
        let mut r = rng();
        tui.input_buf = "1".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.wipe_start.is_some(), "wipe_start should be set after Map → RestSite transition");
    }

    #[test]
    fn entering_shop_from_map_sets_wipe_start() {
        let mut tui = make_map_tui_with_node(MapNode::Merchant);
        let mut r = rng();
        tui.input_buf = "1".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.wipe_start.is_some(), "wipe_start should be set after Map → Shop transition");
    }

    #[test]
    fn entering_treasure_from_map_sets_wipe_start() {
        let mut tui = make_map_tui_with_node(MapNode::Treasure);
        let mut r = rng();
        tui.input_buf = "1".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.wipe_start.is_some(), "wipe_start should be set after Map → Treasure transition");
    }

    #[test]
    fn entering_event_from_map_sets_wipe_start() {
        let mut tui = make_map_tui_with_node(MapNode::Event);
        let mut r = rng();
        tui.input_buf = "1".to_string();
        tui.handle_enter(&mut r);
        assert!(tui.wipe_start.is_some(), "wipe_start should be set after Map → Event transition");
    }

    #[test]
    fn wipe_active_blacks_out_frame() {
        let mut tui = make_combat_tui();
        tui.wipe_start = now();
        let out = render_to_string(&mut tui,100, 30);
        assert!(!out.contains("Draw:"), "pile counts should not be visible during wipe");
        assert!(!out.contains("Hand"), "hand panel should not be visible during wipe");
    }

    #[test]
    fn wipe_shows_floor_number_and_room_type() {
        let mut tui = make_combat_tui();
        tui.wipe_start = now();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Floor"), "should show floor label during wipe");
        assert!(out.contains("COMBAT"), "should show room type in caps during wipe");
    }

    #[test]
    fn wipe_shows_enemy_names_during_combat_transition() {
        let mut tui = make_combat_tui();
        tui.wipe_start = now();
        let out = render_to_string(&mut tui,100, 30);
        assert!(out.contains("Louse"), "should show enemy name on title card during wipe");
    }

    // ─── help overlay render ───────────────────────────────────────

    #[test]
    fn help_overlay_visible_when_show_help_true() {
        let mut tui = make_combat_tui();
        tui.show_help = true;
        let frame = render_to_string(&mut tui,100, 30);
        assert!(frame.contains("Help"), "overlay should show Help title");
        assert!(frame.contains("play"), "overlay should list play command");
        assert!(frame.contains("end"), "overlay should list end command");
    }

    #[test]
    fn help_overlay_hidden_when_show_help_false() {
        let mut tui = make_combat_tui();
        let frame = render_to_string(&mut tui,100, 30);
        assert!(!frame.contains("Help"), "overlay should not appear by default");
    }

    // ─── handle_key ────────────────────────────────────────────────

    #[test]
    fn handle_key_char_appends_to_input_buf() {
        let mut tui = make_map_tui();
        let mut r = rng();
        handle_key(&mut tui, &mut r, crate::key::Key::Char('1'));
        assert_eq!(tui.input_buf, "1");
    }

    #[test]
    fn handle_key_backspace_removes_last_char() {
        let mut tui = make_map_tui();
        let mut r = rng();
        tui.input_buf = "12".to_string();
        handle_key(&mut tui, &mut r, crate::key::Key::Backspace);
        assert_eq!(tui.input_buf, "1");
    }

    #[test]
    fn handle_key_question_mark_opens_help() {
        let mut tui = make_map_tui();
        let mut r = rng();
        handle_key(&mut tui, &mut r, crate::key::Key::Char('?'));
        assert!(tui.show_help);
    }

    #[test]
    fn handle_key_esc_sets_should_quit() {
        let mut tui = make_map_tui();
        let mut r = rng();
        handle_key(&mut tui, &mut r, crate::key::Key::Esc);
        assert!(tui.should_quit);
    }

    #[test]
    fn handle_key_enter_submits_input() {
        let mut tui = make_map_tui();
        let mut r = rng();
        tui.input_buf = "1".to_string();
        handle_key(&mut tui, &mut r, crate::key::Key::Enter);
        assert!(tui.input_buf.is_empty());
    }

    #[test]
    fn handle_key_other_is_no_op() {
        let mut tui = make_map_tui();
        let mut r = rng();
        handle_key(&mut tui, &mut r, crate::key::Key::Other);
        assert!(tui.input_buf.is_empty());
    }

}
