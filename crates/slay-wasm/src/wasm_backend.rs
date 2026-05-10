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
        let mut last_x: Option<u16> = None;
        let mut last_y: Option<u16> = None;

        for (x, y, cell) in content {
            // Move cursor when not at the natural next position.
            if last_y != Some(y) || last_x.map_or(true, |lx| lx + 1 != x) {
                self.buf.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
            }

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
            last_x = Some(x);
            last_y = Some(y);
        }

        // Final reset so the next render starts clean.
        if last_x.is_some() {
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
