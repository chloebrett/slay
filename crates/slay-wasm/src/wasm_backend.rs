use ratatui::{
    backend::Backend,
    buffer::Cell,
    layout::Size,
    style::{Color, Modifier, Style},
};
use std::io;

/// A ratatui `Backend` that renders to an in-memory ANSI string.
/// Pass to `Terminal::new(WasmBackend::new(cols, rows))`, draw into it,
/// then call `.output()` or `.take_output()` to get the ANSI sequence string
/// for forwarding to xterm.js via `term.write(ansi)`.
pub struct WasmBackend {
    width: u16,
    height: u16,
    buf: String,
}

impl WasmBackend {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height, buf: String::new() }
    }

    pub fn output(&self) -> &str {
        &self.buf
    }

    pub fn take_output(&mut self) -> String {
        std::mem::take(&mut self.buf)
    }
}

impl Backend for WasmBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        // Sentinel: no style applied yet.
        let mut current_style = Style::default()
            .fg(Color::Indexed(255)) // impossible initial value to force first emit
            .bg(Color::Indexed(255));
        let mut current_modifier = Modifier::empty();
        let mut wrote = false;

        for (x, y, cell) in content {
            // Spacer cells (right half of a wide character) carry stale or empty
            // symbols — skip them entirely to avoid double-writing wide glyphs.
            if cell.skip {
                continue;
            }

            // Always emit absolute cursor position. xterm.js renders some emoji
            // (e.g. ⚔️) as 2 columns wide while unicode-width returns 1, so
            // adjacency tracking drifts and corrupts layout.
            self.buf.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
            wrote = true;

            let style_changed = current_style.fg != Some(cell.fg)
                || current_style.bg != Some(cell.bg)
                || current_modifier != cell.modifier;

            if style_changed {
                // Reset then re-apply full style.
                self.buf.push_str("\x1b[0m");

                match cell.fg {
                    Color::Reset => {}
                    Color::Black => self.buf.push_str("\x1b[30m"),
                    Color::Red => self.buf.push_str("\x1b[31m"),
                    Color::Green => self.buf.push_str("\x1b[32m"),
                    Color::Yellow => self.buf.push_str("\x1b[33m"),
                    Color::Blue => self.buf.push_str("\x1b[34m"),
                    Color::Magenta => self.buf.push_str("\x1b[35m"),
                    Color::Cyan => self.buf.push_str("\x1b[36m"),
                    Color::White => self.buf.push_str("\x1b[37m"),
                    Color::Gray => self.buf.push_str("\x1b[90m"),
                    Color::DarkGray => self.buf.push_str("\x1b[90m"),
                    Color::LightRed => self.buf.push_str("\x1b[91m"),
                    Color::LightGreen => self.buf.push_str("\x1b[92m"),
                    Color::LightYellow => self.buf.push_str("\x1b[93m"),
                    Color::LightBlue => self.buf.push_str("\x1b[94m"),
                    Color::LightMagenta => self.buf.push_str("\x1b[95m"),
                    Color::LightCyan => self.buf.push_str("\x1b[96m"),
                    Color::Rgb(r, g, b) => self.buf.push_str(&format!("\x1b[38;2;{r};{g};{b}m")),
                    Color::Indexed(i) => self.buf.push_str(&format!("\x1b[38;5;{i}m")),
                }
                match cell.bg {
                    Color::Reset => {}
                    Color::Black => self.buf.push_str("\x1b[40m"),
                    Color::Red => self.buf.push_str("\x1b[41m"),
                    Color::Green => self.buf.push_str("\x1b[42m"),
                    Color::Yellow => self.buf.push_str("\x1b[43m"),
                    Color::Blue => self.buf.push_str("\x1b[44m"),
                    Color::Magenta => self.buf.push_str("\x1b[45m"),
                    Color::Cyan => self.buf.push_str("\x1b[46m"),
                    Color::White => self.buf.push_str("\x1b[47m"),
                    Color::Gray => self.buf.push_str("\x1b[100m"),
                    Color::DarkGray => self.buf.push_str("\x1b[100m"),
                    Color::LightRed => self.buf.push_str("\x1b[101m"),
                    Color::LightGreen => self.buf.push_str("\x1b[102m"),
                    Color::LightYellow => self.buf.push_str("\x1b[103m"),
                    Color::LightBlue => self.buf.push_str("\x1b[104m"),
                    Color::LightMagenta => self.buf.push_str("\x1b[105m"),
                    Color::LightCyan => self.buf.push_str("\x1b[106m"),
                    Color::Rgb(r, g, b) => self.buf.push_str(&format!("\x1b[48;2;{r};{g};{b}m")),
                    Color::Indexed(i) => self.buf.push_str(&format!("\x1b[48;5;{i}m")),
                }
                if cell.modifier.contains(Modifier::BOLD)       { self.buf.push_str("\x1b[1m"); }
                if cell.modifier.contains(Modifier::DIM)        { self.buf.push_str("\x1b[2m"); }
                if cell.modifier.contains(Modifier::ITALIC)     { self.buf.push_str("\x1b[3m"); }
                if cell.modifier.contains(Modifier::UNDERLINED) { self.buf.push_str("\x1b[4m"); }
                if cell.modifier.contains(Modifier::REVERSED)   { self.buf.push_str("\x1b[7m"); }

                current_style = Style::default().fg(cell.fg).bg(cell.bg);
                current_modifier = cell.modifier;
            }

            self.buf.push_str(cell.symbol());
        }

        // Final reset so the next render starts clean.
        if wrote {
            self.buf.push_str("\x1b[0m");
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.buf.push_str("\x1b[?25l");
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.buf.push_str("\x1b[?25h");
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<ratatui::layout::Position> {
        Ok(ratatui::layout::Position { x: 0, y: 0 })
    }

    fn set_cursor_position<P: Into<ratatui::layout::Position>>(&mut self, position: P) -> io::Result<()> {
        let p = position.into();
        self.buf.push_str(&format!("\x1b[{};{}H", p.y + 1, p.x + 1));
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        self.buf.push_str("\x1b[2J\x1b[H");
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(Size { width: self.width, height: self.height })
    }

    fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
        Ok(ratatui::backend::WindowSize {
            columns_rows: Size { width: self.width, height: self.height },
            pixels: Size { width: 0, height: 0 },
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::Backend;
    use ratatui::buffer::Cell;

    fn make_cell(symbol: &str) -> Cell {
        let mut c = Cell::default();
        c.set_symbol(symbol);
        c
    }

    fn draw(backend: &mut WasmBackend, cells: &[(u16, u16, Cell)]) {
        backend.draw(cells.iter().map(|(x, y, c)| (*x, *y, c))).unwrap();
    }

    #[test]
    fn single_char_emits_absolute_cursor_and_symbol() {
        let mut b = WasmBackend::new(10, 5);
        draw(&mut b, &[(0, 0, make_cell("A"))]);
        let out = b.take_output();
        assert!(out.contains("\x1b[1;1H"), "cursor at row 1 col 1");
        assert!(out.contains('A'), "symbol written");
        assert!(out.ends_with("\x1b[0m"), "trailing reset");
    }

    #[test]
    fn cursor_at_correct_row_and_col() {
        let mut b = WasmBackend::new(20, 10);
        draw(&mut b, &[(4, 2, make_cell("Z"))]);
        let out = b.take_output();
        assert!(out.contains("\x1b[3;5H"), "row 3 (1-based), col 5 (1-based)");
    }

    #[test]
    fn every_visible_cell_gets_absolute_cursor() {
        let mut b = WasmBackend::new(20, 5);
        draw(&mut b, &[
            (0, 0, make_cell("A")),
            (1, 0, make_cell("B")),
            (5, 0, make_cell("C")),
        ]);
        let out = b.take_output();
        assert!(out.contains("\x1b[1;1H"), "cursor for A");
        assert!(out.contains("\x1b[1;2H"), "cursor for B");
        assert!(out.contains("\x1b[1;6H"), "cursor for C (non-adjacent, still gets cursor)");
    }

    #[test]
    fn skip_cells_are_not_rendered() {
        let mut b = WasmBackend::new(10, 5);
        let mut spacer = make_cell(" ");
        spacer.set_skip(true);
        draw(&mut b, &[(0, 0, make_cell("⚔️")), (1, 0, spacer)]);
        let out = b.take_output();
        assert_eq!(out.matches("⚔️").count(), 1, "emoji written exactly once");
        assert!(!out.contains("\x1b[1;2H"), "no cursor emitted for spacer cell");
    }

    #[test]
    fn wide_emoji_followed_by_skip_not_doubled() {
        let mut b = WasmBackend::new(10, 5);
        let mut spacer = make_cell(" ");
        spacer.set_skip(true);
        draw(&mut b, &[
            (0, 0, make_cell("⚔️")),
            (1, 0, spacer),
            (2, 0, make_cell("💀")),
        ]);
        let out = b.take_output();
        assert_eq!(out.matches("⚔️").count(), 1, "sword not doubled");
        assert_eq!(out.matches("💀").count(), 1, "skull not doubled");
    }

    #[test]
    fn color_change_emits_escape_codes() {
        let mut b = WasmBackend::new(10, 5);
        let mut c = make_cell("X");
        c.set_fg(Color::Red);
        draw(&mut b, &[(0, 0, c)]);
        let out = b.take_output();
        assert!(out.contains("\x1b[31m"), "red foreground code");
    }

    #[test]
    fn same_style_consecutive_cells_no_redundant_codes() {
        let mut b = WasmBackend::new(10, 5);
        let mut c1 = make_cell("A");
        c1.set_fg(Color::Green);
        let mut c2 = make_cell("B");
        c2.set_fg(Color::Green);
        draw(&mut b, &[(0, 0, c1), (1, 0, c2)]);
        let out = b.take_output();
        assert_eq!(out.matches("\x1b[32m").count(), 1, "green fg emitted once only");
    }

    #[test]
    fn empty_draw_produces_no_output() {
        let mut b = WasmBackend::new(10, 5);
        draw(&mut b, &[]);
        assert_eq!(b.take_output(), "", "no output for empty draw");
    }
}
