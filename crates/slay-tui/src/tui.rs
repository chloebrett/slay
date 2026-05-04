use crate::engine::{
    apply_and_drain, card_type_icon, describe_event, describe_intent, enemy_icon, statuses_inline,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use slay_core::{
    AnyRng, CardRewardState, CombatState, GameState, MapNode, MapState, RestSiteState, ShopState,
    StatusMap, CARD_PRICE, RELIC_PRICE, POTION_PRICE,
};
use std::collections::VecDeque;

const LOG_CAPACITY: usize = 200;

#[derive(Debug)]
pub struct TuiState {
    pub game: GameState,
    pub input_buf: String,
    pub event_log: VecDeque<String>,
    pub last_error: Option<String>,
    pub show_pile: Option<PileView>,
    pub debug: bool,
    pub should_quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PileView {
    Draw,
    Discard,
    Exhaust,
}

impl TuiState {
    pub fn new(game: GameState, debug: bool) -> Self {
        let mut s = Self {
            game,
            input_buf: String::new(),
            event_log: VecDeque::new(),
            last_error: None,
            show_pile: None,
            debug,
            should_quit: false,
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

        // Pile view shortcuts in combat
        if matches!(self.game, GameState::Combat { .. }) {
            match trimmed {
                "z" => { self.show_pile = Some(PileView::Draw);    return; }
                "x" => { self.show_pile = Some(PileView::Discard); return; }
                "c" => { self.show_pile = Some(PileView::Exhaust); return; }
                _ => {}
            }
        }

        // Dismiss pile overlay on any other key event
        self.show_pile = None;

        let Some(command) = crate::command::parse(trimmed, &self.game, self.debug) else {
            self.last_error = Some("Unknown command.".to_string());
            return;
        };

        match apply_and_drain(self.game.clone(), command, rng) {
            Ok((new_state, events)) => {
                self.game = new_state;
                for ev in &events {
                    let msg = describe_event(ev);
                    self.push_log(msg);
                }
                self.last_error = None;
                if matches!(self.game, GameState::GameOver { .. }) {
                    self.should_quit = true;
                }
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
            }
        }
    }
}

/// Render one frame. Pure function over `TuiState`; safe to call from tests with `TestBackend`.
pub fn render_frame(f: &mut Frame, tui: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top bar
            Constraint::Min(0),    // main area
            Constraint::Length(1), // status line
            Constraint::Length(3), // input box
        ])
        .split(f.area());

    render_top_bar(f, chunks[0], &tui.game);
    render_main(f, chunks[1], tui);
    render_status_line(f, chunks[2], tui);
    render_input(f, chunks[3], tui);

    // Pile overlay (drawn last, on top)
    if let Some(view) = tui.show_pile {
        render_pile_overlay(f, f.area(), &tui.game, view);
    }
}

fn render_top_bar(f: &mut Frame, area: Rect, state: &GameState) {
    let player = match state {
        GameState::Map(m) => Some(&m.player),
        GameState::Combat { state, .. } => Some(&state.player),
        GameState::RestSite(rs) => Some(&rs.player),
        GameState::CardReward(cr) => Some(&cr.player),
        GameState::Shop(shop) => Some(&shop.player),
        GameState::GameOver { .. } => None,
    };
    let line = match player {
        Some(p) => {
            let potions: Vec<String> = p.potions.iter().enumerate()
                .map(|(i, pot)| format!("[{}]{}", i + 1, pot.name()))
                .collect();
            let potion_str = if potions.is_empty() {
                String::new()
            } else {
                format!("   🧪 {}", potions.join(" "))
            };
            format!(
                "🧙  HP {}/{} {}   ⚡ {}/{}   🛡 {}   🪙 {}   🃏 {} cards{}",
                p.hp.0, p.max_hp.0, hp_bar(p.hp.0, p.max_hp.0, 20),
                p.energy.0, p.max_energy.0, p.block.0, p.gold, p.deck.len(), potion_str
            )
        }
        None => String::new(),
    };
    let para = Paragraph::new(line).style(Style::default().fg(Color::White));
    f.render_widget(para, area);
}

fn render_main(f: &mut Frame, area: Rect, tui: &TuiState) {
    match &tui.game {
        GameState::Map(map) => render_map(f, area, map),
        GameState::Combat { state, .. } => render_combat(f, area, state, &tui.event_log),
        GameState::RestSite(rs) => render_rest(f, area, rs),
        GameState::CardReward(cr) => render_card_reward(f, area, cr),
        GameState::Shop(shop) => render_shop(f, area, shop),
        GameState::GameOver { victory } => render_game_over(f, area, *victory),
    }
}

fn render_map(f: &mut Frame, area: Rect, map: &MapState) {
    let [map_area, choices_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .areas(area);

    let items: Vec<ListItem> = map.graph.rows.iter().enumerate().rev()
        .map(|(i, row)| {
            let nodes: String = row.iter()
                .map(|node| { let (icon, name) = node_label(node); format!("{icon} {name}") })
                .collect::<Vec<_>>()
                .join("   ");
            let marker = match i.cmp(&map.floor) {
                std::cmp::Ordering::Less    => "✓",
                std::cmp::Ordering::Equal   => "▶",
                std::cmp::Ordering::Greater => " ",
            };
            let text = format!(" {marker}  {}.  {nodes}", i + 1);
            let style = match i.cmp(&map.floor) {
                std::cmp::Ordering::Equal   => Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow),
                std::cmp::Ordering::Less    => Style::default().fg(Color::DarkGray),
                std::cmp::Ordering::Greater => Style::default(),
            };
            ListItem::new(Line::styled(text, style))
        })
        .collect();
    let block = Block::default().borders(Borders::ALL).title(" 🗺️  Map ");
    let list = List::new(items).block(block);
    f.render_widget(list, map_area);

    let choices = map_choices_line(map);
    let choices_block = Block::default().borders(Borders::ALL).title(" Choose ");
    f.render_widget(Paragraph::new(choices).block(choices_block), choices_area);
}

fn map_choices_line(map: &MapState) -> String {
    if map.available_cols.len() == 1 {
        let node = &map.graph.rows[map.floor][map.available_cols[0]];
        let (icon, name) = node_label(node);
        format!("[Enter] {icon} {name}")
    } else {
        map.available_cols.iter()
            .map(|&col| {
                let node = &map.graph.rows[map.floor][col];
                let (icon, name) = node_label(node);
                format!("[{}] {icon} {name}", col + 1)
            })
            .collect::<Vec<_>>()
            .join("   ")
    }
}

fn node_label(node: &MapNode) -> (&'static str, &'static str) {
    match node {
        MapNode::Combat(_) => ("⚔️", "Combat"),
        MapNode::RestSite  => ("🔥", "Rest Site"),
        MapNode::Boss(_)   => ("💀", "Boss"),
        MapNode::Merchant  => ("🛒", "Shop"),
    }
}

fn render_combat(f: &mut Frame, area: Rect, state: &CombatState, log: &VecDeque<String>) {
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

    render_enemies(f, enemies_area, state);
    render_hand(f, hand_area, state);
    render_piles(f, piles_area, state);
    render_log(f, right, log);
}

fn render_enemies(f: &mut Frame, area: Rect, state: &CombatState) {
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
        ListItem::new(line)
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

fn render_hand(f: &mut Frame, area: Rect, state: &CombatState) {
    let dummy = StatusMap::new();
    let target_statuses = state.enemies.first().map_or(&dummy, |e| &e.statuses);

    let items: Vec<ListItem> = if state.player.hand.is_empty() {
        vec![ListItem::new(Line::styled(
            "(empty)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        state.player.hand.iter().enumerate().map(|(i, card)| {
            let affordable = card.energy_cost() <= state.player.energy;
            let style = if affordable {
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
                card.energy_cost().0,
                desc,
            );
            ListItem::new(Line::styled(text, style))
        }).collect()
    };

    let title = format!(" 🤚 Hand  (Turn {}) ", state.turn);
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
            card.energy_cost().0,
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
                i + 1, card.name(), card.energy_cost().0, card.description(), CARD_PRICE,
            )
        };
        let style = if *purchased { Style::default().fg(Color::DarkGray) } else { Style::default() };
        items.push(ListItem::new(Line::styled(text, style)));
    }

    items.push(ListItem::new(Line::raw("")));
    items.push(ListItem::new(Line::styled("Relic:", Style::default().add_modifier(Modifier::BOLD))));
    match &shop.relic {
        Some((relic, true)) => items.push(ListItem::new(Line::styled(
            format!("[r]  {} — [sold]", relic.id()), Style::default().fg(Color::DarkGray),
        ))),
        Some((relic, false)) => items.push(ListItem::new(Line::raw(
            format!("[r]  {} — {}g", relic.id(), RELIC_PRICE),
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

fn render_input(f: &mut Frame, area: Rect, tui: &TuiState) {
    let prompt = format!("> {}", tui.input_buf);
    let block = Block::default().borders(Borders::ALL).title(" Command ");
    let para = Paragraph::new(prompt).block(block);
    f.render_widget(para, area);
}

fn render_pile_overlay(f: &mut Frame, area: Rect, state: &GameState, view: PileView) {
    let GameState::Combat { state: cs, .. } = state else { return };
    let (label, pile) = match view {
        PileView::Draw    => ("🎴 Draw pile",    &cs.player.draw_pile),
        PileView::Discard => ("🗑️  Discard pile", &cs.player.discard_pile),
        PileView::Exhaust => ("🔥 Exhaust pile", &cs.player.exhaust_pile),
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

// Public entry point: take over the terminal and run the ratatui event loop.
#[cfg(not(test))]
pub fn run_tui(state: GameState, rng: &mut AnyRng, debug: bool) -> std::io::Result<()> {
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

    let mut tui = TuiState::new(state, debug);

    let result = (|| -> std::io::Result<()> {
        loop {
            terminal.draw(|f| render_frame(f, &tui))?;

            if tui.should_quit {
                // Wait for any key, then exit
                if event::poll(Duration::from_millis(5000))? {
                    let _ = event::read()?;
                }
                break;
            }

            if !event::poll(Duration::from_millis(100))? {
                continue;
            }
            let CxEvent::Key(key) = event::read()? else { continue };
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Dismiss pile overlay on any key
            if tui.show_pile.is_some() {
                tui.show_pile = None;
                continue;
            }

            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char(c) => tui.input_buf.push(c),
                KeyCode::Backspace => { tui.input_buf.pop(); }
                KeyCode::Enter => tui.handle_enter(rng),
                _ => {}
            }
        }
        Ok(())
    })();

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();
    result
}

// Convenience for tests: render to a TestBackend and return the buffer as a String.
#[cfg(test)]
pub(crate) fn render_to_string(tui: &TuiState, width: u16, height: u16) -> String {
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
        AnyRng, Card, Command, EnemyKind, Grade, NoOpRng, new_run, new_simple_run,
    };

    fn rng() -> AnyRng { AnyRng::NoOp(NoOpRng) }

    fn make_combat_tui() -> TuiState {
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::Louse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        TuiState::new(state, false)
    }

    fn make_map_tui() -> TuiState {
        let state = new_simple_run();
        TuiState::new(state, false)
    }

    fn make_main_run_map_tui() -> TuiState {
        let mut noop = NoOpRng;
        let state = new_run(&mut noop);
        TuiState::new(state, false)
    }

    // ─── render_frame ─────────────────────────────────────────────

    #[test]
    fn combat_screen_shows_enemy_name() {
        let tui = make_combat_tui();
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("Louse"), "expected 'Louse' in:\n{out}");
    }

    #[test]
    fn combat_screen_shows_player_hp_in_top_bar() {
        let tui = make_combat_tui();
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("80/80"), "expected '80/80' in:\n{out}");
    }

    #[test]
    fn top_bar_hp_bar_reflects_current_health() {
        use slay_core::{Block, Energy, Hp, MapState, Player, Scenario, StatusMap};
        let mut r = rng();
        let graph = slay_core::generate_map(&mut r);
        let state = GameState::Map(MapState {
            player: Player {
                hp: Hp(40), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0, relics: vec![], potions: vec![],
            },
            floor: 0, graph, available_cols: vec![0], next_enemies: None,
            scenario: Scenario::Main,
        });
        let out = render_to_string(&TuiState::new(state, false), 120, 30);
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
        let tui = make_combat_tui();
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("Draw:"), "expected 'Draw:' in:\n{out}");
        assert!(out.contains("Discard:"), "expected 'Discard:' in:\n{out}");
    }

    #[test]
    fn combat_screen_shows_log_panel_title() {
        let tui = make_combat_tui();
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("Log"), "expected 'Log' in:\n{out}");
    }

    #[test]
    fn map_screen_shows_node_names() {
        let tui = make_main_run_map_tui();
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("Combat"), "expected 'Combat' in:\n{out}");
        assert!(out.contains("Boss"), "expected 'Boss' in:\n{out}");
    }

    #[test]
    fn map_screen_shows_current_floor_marker() {
        let tui = make_map_tui();
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("▶"), "expected '▶' marker in:\n{out}");
    }

    #[test]
    fn map_screen_shows_available_choices() {
        let tui = make_main_run_map_tui();
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("[1]"), "expected '[1]' choice in:\n{out}");
        assert!(out.contains("[2]"), "expected '[2]' choice in:\n{out}");
    }

    #[test]
    fn map_grid_shows_both_nodes_on_two_column_floor() {
        let graph = slay_core::MapGraph {
            rows: vec![vec![MapNode::Combat(vec![]), MapNode::RestSite]],
            edges: vec![vec![vec![], vec![]]],
        };
        use slay_core::{Block, Energy, Hp, Player, Scenario, StatusMap};
        let state = GameState::Map(MapState {
            player: Player {
                hp: Hp(80), max_hp: Hp(80), block: Block(0),
                energy: Energy(3), max_energy: Energy(3),
                hand: vec![], draw_pile: vec![], discard_pile: vec![],
                exhaust_pile: vec![], statuses: StatusMap::new(),
                deck: vec![], gold: 0, relics: vec![], potions: vec![],
            },
            floor: 0,
            graph,
            available_cols: vec![0, 1],
            next_enemies: None,
            scenario: Scenario::Main,
        });
        let tui = TuiState::new(state, false);
        let out = render_to_string(&tui, 120, 30);
        let marker_line = out.lines().find(|l| l.contains('▶')).expect("expected ▶ marker line");
        assert!(marker_line.contains("Combat"), "expected 'Combat' on ▶ line: {marker_line}");
        assert!(marker_line.contains("Rest Site"), "expected 'Rest Site' on ▶ line: {marker_line}");
    }

    #[test]
    fn rest_site_screen_shows_heal_amount() {
        let mut r = NoOpRng;
        let state = new_run(&mut r);
        // Force into rest site by hand
        let player = match state {
            GameState::Map(m) => m.player,
            _ => panic!(),
        };
        let rs_state = GameState::RestSite(slay_core::RestSiteState {
            player,
            floor: 3,
            graph: slay_core::generate_map(&mut r),
            available_cols: vec![0, 1],
        });
        let tui = TuiState::new(rs_state, false);
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("Rest Site"), "expected 'Rest Site' in:\n{out}");
        assert!(out.contains("Heal"), "expected 'Heal' in:\n{out}");
    }

    #[test]
    fn card_reward_screen_shows_card_options() {
        let mut r = NoOpRng;
        let state = new_run(&mut r);
        let player = match state {
            GameState::Map(m) => m.player,
            _ => panic!(),
        };
        let cr = GameState::CardReward(slay_core::CardRewardState {
            player,
            floor: 1,
            options: vec![Card::Strike(Grade::Base), Card::Defend(Grade::Base), Card::Bash(Grade::Base)],
            offered_potion: None,
            graph: slay_core::generate_map(&mut r),
            available_cols: vec![0, 1],
        });
        let tui = TuiState::new(cr, false);
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("Card Reward"), "expected 'Card Reward' in:\n{out}");
        assert!(out.contains("Strike"), "expected 'Strike' in:\n{out}");
        assert!(out.contains("skip"), "expected 'skip' option in:\n{out}");
    }

    #[test]
    fn game_over_victory_screen_shows_message() {
        let tui = TuiState::new(GameState::GameOver { victory: true }, false);
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("conquered"), "expected 'conquered' in:\n{out}");
    }

    #[test]
    fn game_over_defeat_screen_shows_message() {
        let tui = TuiState::new(GameState::GameOver { victory: false }, false);
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("slain"), "expected 'slain' in:\n{out}");
    }

    #[test]
    fn input_box_shows_typed_characters() {
        let mut tui = make_combat_tui();
        tui.input_buf = "play 1".to_string();
        let out = render_to_string(&tui, 100, 30);
        assert!(out.contains("> play 1"), "expected '> play 1' in:\n{out}");
    }

    #[test]
    #[ignore = "visual check; run with --ignored to dump"]
    fn visual_dump_combat_screen() {
        // Build a realistic combat: spawn a Louse, enter combat, add some cards to hand
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::Louse]), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::ChooseNode(0), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::AddCard(Card::Strike(Grade::Base)), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::AddCard(Card::Defend(Grade::Base)), &mut r).unwrap().0;
        state = apply_and_drain(state, Command::AddCard(Card::Bash(Grade::Base)), &mut r).unwrap().0;

        let mut tui = TuiState::new(state, true);
        tui.push_log("─── Turn 1 ───".to_string());
        tui.push_log("👁  Enemy prepares: ⚔️ Attack 8.".to_string());
        tui.input_buf = "play 1".to_string();

        eprintln!("{}", render_to_string(&tui, 100, 30));
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
    fn handle_enter_game_over_sets_should_quit() {
        // Build a combat about to lose: player at 1 HP, end turn lets enemy kill.
        let mut state = new_simple_run();
        let mut r = rng();
        state = apply_and_drain(state, Command::Spawn(vec![EnemyKind::Louse]), &mut r).unwrap().0;
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
}
