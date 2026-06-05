// ── Input bar widget ──────────────────────────────────────────────────────────
//
// Multi-line input with history, cursor tracking, and mode/model label.

use crate::tui::cell::{Color, Style};
use crate::tui::event::{Event, Key, KeyEvent};
use crate::tui::layout::Rect;
use crate::tui::screen::Screen;
use crate::tui::widget::{Action, Widget, styles};

/// The input bar at the bottom of the screen.
///
/// Displays a prompt with mode and model labels, and accepts
/// multi-line text input. Enter submits, Shift+Enter inserts a newline.
pub struct InputBarWidget {
    /// Current input text.
    content: String,
    /// Cursor position (byte offset in content).
    cursor: usize,
    /// Scroll offset for the input area (for multi-line input).
    scroll_offset: usize,
    /// Input history (previous messages).
    history: Vec<String>,
    /// Current position in history navigation (None = not navigating).
    history_index: Option<usize>,
    /// The mode label to display (e.g., "agent").
    mode_label: String,
    /// The mode color.
    mode_color: Color,
    /// The model name to display.
    model_name: String,
    /// Whether the input bar is focused.
    focused: bool,
}

impl InputBarWidget {
    pub fn new(mode_label: &str, model_name: &str) -> Self {
        let mode_color = match mode_label {
            "casual" => styles::MODE_CASUAL_FG,
            "planning" => styles::MODE_PLANNING_FG,
            "agent" => styles::MODE_AGENT_FG,
            "research" => styles::MODE_RESEARCH_FG,
            _ => Color::WHITE,
        };

        Self {
            content: String::new(),
            cursor: 0,
            scroll_offset: 0,
            history: Vec::new(),
            history_index: None,
            mode_label: mode_label.to_string(),
            mode_color,
            model_name: model_name.to_string(),
            focused: true,
        }
    }

    /// Get the current input text and clear the buffer.
    pub fn take_input(&mut self) -> String {
        let text = self.content.clone();
        if !text.is_empty() {
            self.history.push(text.clone());
        }
        self.content.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.history_index = None;
        text
    }

    /// Set the input text (e.g., from --prompt flag).
    pub fn set_input(&mut self, text: &str) {
        self.content = text.to_string();
        self.cursor = self.content.len();
    }

    /// Update the mode label and model name.
    pub fn update_labels(&mut self, mode_label: &str, model_name: &str) {
        self.mode_label = mode_label.to_string();
        self.mode_color = match mode_label {
            "casual" => styles::MODE_CASUAL_FG,
            "planning" => styles::MODE_PLANNING_FG,
            "agent" => styles::MODE_AGENT_FG,
            "research" => styles::MODE_RESEARCH_FG,
            _ => Color::WHITE,
        };
        self.model_name = model_name.to_string();
    }

    /// Calculate which line and column the cursor is on.
    #[allow(dead_code)]
    fn cursor_line_col(&self) -> (usize, usize) {
        let text_before_cursor = &self.content[..self.cursor];
        let line = text_before_cursor.lines().count().saturating_sub(1);
        let col = text_before_cursor
            .lines()
            .next_back()
            .map(|l| l.len())
            .unwrap_or(0);
        (line, col)
    }

    /// Count the number of lines in the input.
    #[allow(dead_code)]
    fn line_count(&self) -> usize {
        self.content.lines().count().max(1)
    }
}

impl Widget for InputBarWidget {
    fn render(&mut self, area: Rect, screen: &mut Screen) {
        if area.is_empty() || area.height < 2 {
            return;
        }

        let row = area.y;
        let _width = area.width as usize;

        // Draw top border
        screen.hline(
            row,
            area.x,
            area.x + area.width - 1,
            '─',
            Color::Ansi(240),
            Color::Default,
        );

        // Draw prompt and input on the next rows
        let input_row = row + 1;
        let prompt = format!("[{}] ", self.mode_label);
        let _model_suffix = format!(" {}{}", self.model_name, Color::Default.fg_escape());

        // Draw mode label
        let mut col = area.x;
        screen.write_str(
            input_row,
            col,
            &prompt,
            self.mode_color,
            styles::INPUT_BAR_BG,
            Style::bold(),
        );
        col += prompt.len() as u16;

        // Draw input content (with wrapping if needed)
        let available_width = area.width.saturating_sub(col - area.x);
        let display_text = if self.content.len() > available_width as usize {
            // Show the end of the text that fits, scrolled to cursor
            let start = self.content.len().saturating_sub(available_width as usize);
            &self.content[start..]
        } else {
            &self.content
        };

        screen.write_str(
            input_row,
            col,
            display_text,
            Color::WHITE,
            styles::INPUT_BAR_BG,
            Style::default(),
        );

        // Draw cursor (blinking is handled by terminal, we just position it)
        if self.focused {
            let cursor_col = col + self.cursor.min(display_text.len()) as u16;
            if cursor_col < area.x + area.width {
                // Underline the character under the cursor
                if let Some(cell) = screen.get_mut(input_row, cursor_col) {
                    cell.style.underline = true;
                }
            }
        }

        // Fill the rest of the input line with background
        let text_end = col + display_text.len() as u16;
        if text_end < area.x + area.width {
            // Background is already filled by write_str
        }

        // For multi-line input, render additional lines
        let lines: Vec<&str> = self.content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                continue; // Already rendered the first line
            }
            let line_row = input_row + i as u16;
            if line_row >= area.y + area.height {
                break;
            }
            screen.write_str(
                line_row,
                area.x,
                line,
                Color::WHITE,
                styles::INPUT_BAR_BG,
                Style::default(),
            );
        }
    }

    fn handle_event(&mut self, event: &Event) -> Action {
        let Event::Key(key) = event else {
            return Action::None;
        };

        match key {
            KeyEvent {
                key: Key::Enter,
                modifiers,
            } => {
                if modifiers.shift {
                    // Shift+Enter: insert newline
                    self.content.insert(self.cursor, '\n');
                    self.cursor += 1;
                    Action::None
                } else {
                    // Enter: submit the message
                    let text = self.take_input();
                    if text.trim().is_empty() {
                        Action::None
                    } else {
                        Action::SendMessage(text)
                    }
                }
            }
            KeyEvent {
                key: Key::Backspace,
                ..
            } => {
                if self.cursor > 0 {
                    // Handle deleting across newlines
                    let prev_char = self.content[..self.cursor].chars().next_back();
                    if let Some(ch) = prev_char {
                        self.cursor -= ch.len_utf8();
                        self.content.remove(self.cursor);
                    }
                }
                Action::None
            }
            KeyEvent {
                key: Key::Delete, ..
            } => {
                if self.cursor < self.content.len() {
                    // Find the next character boundary
                    let next_char = self.content[self.cursor..].chars().next();
                    if let Some(ch) = next_char {
                        let end = self.cursor + ch.len_utf8();
                        self.content.replace_range(self.cursor..end, "");
                    }
                }
                Action::None
            }
            KeyEvent { key: Key::Left, .. } => {
                if self.cursor > 0 {
                    if let Some(ch) = self.content[..self.cursor].chars().next_back() {
                        self.cursor -= ch.len_utf8();
                    }
                }
                Action::None
            }
            KeyEvent {
                key: Key::Right, ..
            } => {
                if self.cursor < self.content.len() {
                    if let Some(ch) = self.content[self.cursor..].chars().next() {
                        self.cursor += ch.len_utf8();
                    }
                }
                Action::None
            }
            KeyEvent { key: Key::Home, .. } => {
                // Move to start of current line
                let line_start = self.content[..self.cursor]
                    .rfind('\n')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                self.cursor = line_start;
                Action::None
            }
            KeyEvent { key: Key::End, .. } => {
                // Move to end of current line
                let line_end = self.content[self.cursor..]
                    .find('\n')
                    .map(|p| self.cursor + p)
                    .unwrap_or(self.content.len());
                self.cursor = line_end;
                Action::None
            }
            KeyEvent {
                key: Key::Up,
                modifiers,
            } if !modifiers.alt && !modifiers.ctrl => {
                // History navigation
                if !self.history.is_empty() {
                    let idx = self.history_index.unwrap_or(self.history.len());
                    if idx > 0 {
                        let new_idx = idx - 1;
                        self.history_index = Some(new_idx);
                        self.content = self.history[new_idx].clone();
                        self.cursor = self.content.len();
                    }
                }
                Action::None
            }
            KeyEvent {
                key: Key::Down,
                modifiers,
            } if !modifiers.alt && !modifiers.ctrl => {
                // History navigation
                if let Some(idx) = self.history_index {
                    if idx + 1 < self.history.len() {
                        self.history_index = Some(idx + 1);
                        self.content = self.history[idx + 1].clone();
                    } else {
                        self.history_index = None;
                        self.content.clear();
                    }
                    self.cursor = self.content.len();
                }
                Action::None
            }
            KeyEvent {
                key: Key::Char(c),
                modifiers,
            } => {
                if modifiers.ctrl || modifiers.alt {
                    // Handle Ctrl+key shortcuts
                    match c {
                        'c' => Action::Quit,
                        'd' => Action::Quit,
                        _ => Action::None,
                    }
                } else {
                    self.content.insert(self.cursor, *c);
                    self.cursor += c.len_utf8();
                    Action::None
                }
            }
            KeyEvent {
                key: Key::Escape, ..
            } => {
                if self.content.is_empty() {
                    Action::Quit
                } else {
                    // Clear input on Escape
                    self.content.clear();
                    self.cursor = 0;
                    Action::None
                }
            }
            _ => Action::None,
        }
    }

    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::event::Modifiers;

    #[test]
    fn test_input_bar_new() {
        let bar = InputBarWidget::new("agent", "llama3.1:8b");
        assert!(bar.content.is_empty());
        assert_eq!(bar.cursor, 0);
        assert!(bar.focused);
    }

    #[test]
    fn test_input_bar_take_input() {
        let mut bar = InputBarWidget::new("agent", "llama3.1:8b");
        bar.content = "hello".to_string();
        bar.cursor = 5;
        let text = bar.take_input();
        assert_eq!(text, "hello");
        assert!(bar.content.is_empty());
        assert_eq!(bar.cursor, 0);
        assert_eq!(bar.history.len(), 1);
    }

    #[test]
    fn test_input_bar_type_chars() {
        let mut bar = InputBarWidget::new("agent", "llama3.1:8b");
        let event = Event::Key(KeyEvent {
            key: Key::Char('h'),
            modifiers: Modifiers::new(),
        });
        bar.handle_event(&event);
        assert_eq!(bar.content, "h");
        assert_eq!(bar.cursor, 1);

        let event = Event::Key(KeyEvent {
            key: Key::Char('i'),
            modifiers: Modifiers::new(),
        });
        bar.handle_event(&event);
        assert_eq!(bar.content, "hi");
        assert_eq!(bar.cursor, 2);
    }

    #[test]
    fn test_input_bar_backspace() {
        let mut bar = InputBarWidget::new("agent", "llama3.1:8b");
        bar.content = "hello".to_string();
        bar.cursor = 5;

        let event = Event::Key(KeyEvent {
            key: Key::Backspace,
            modifiers: Modifiers::new(),
        });
        bar.handle_event(&event);
        assert_eq!(bar.content, "hell");
        assert_eq!(bar.cursor, 4);
    }

    #[test]
    fn test_input_bar_enter_submits() {
        let mut bar = InputBarWidget::new("agent", "llama3.1:8b");
        bar.content = "hello".to_string();
        bar.cursor = 5;

        let event = Event::Key(KeyEvent {
            key: Key::Enter,
            modifiers: Modifiers::new(),
        });
        let action = bar.handle_event(&event);
        assert!(matches!(action, Action::SendMessage(ref s) if s == "hello"));
        assert!(bar.content.is_empty());
    }

    #[test]
    fn test_input_bar_shift_enter_newline() {
        let mut bar = InputBarWidget::new("agent", "llama3.1:8b");
        bar.content = "hello".to_string();
        bar.cursor = 5;

        let event = Event::Key(KeyEvent {
            key: Key::Enter,
            modifiers: Modifiers::shift(),
        });
        let action = bar.handle_event(&event);
        assert!(matches!(action, Action::None));
        assert_eq!(bar.content, "hello\n");
    }

    #[test]
    fn test_input_bar_escape_clears() {
        let mut bar = InputBarWidget::new("agent", "llama3.1:8b");
        bar.content = "hello".to_string();
        bar.cursor = 5;

        let event = Event::Key(KeyEvent {
            key: Key::Escape,
            modifiers: Modifiers::new(),
        });
        let action = bar.handle_event(&event);
        assert!(matches!(action, Action::None));
        assert!(bar.content.is_empty());
    }
}
