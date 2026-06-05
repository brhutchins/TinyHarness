// ── Sidebar widget ──────────────────────────────────────────────────────────
//
// Displays project context, pinned files, and active skills in a
// right-side panel.

use crate::tui::cell::{Cell, Color, Style};
use crate::tui::event::Event;
use crate::tui::layout::Rect;
use crate::tui::screen::Screen;
use crate::tui::widget::{Action, Widget, styles};

/// The sidebar widget showing project context.
pub struct SidebarWidget {
    pub project_name: String,
    pub project_type: String,
    pub git_branch: Option<String>,
    pub build_command: String,
    pub test_command: String,
    pub pinned_files: Vec<String>,
    pub active_skills: Vec<(String, String)>, // (name, description)
    pub visible: bool,
}

impl SidebarWidget {
    pub fn new() -> Self {
        Self {
            project_name: String::new(),
            project_type: String::new(),
            git_branch: None,
            build_command: String::new(),
            test_command: String::new(),
            pinned_files: Vec::new(),
            active_skills: Vec::new(),
            visible: true,
        }
    }
}

impl Widget for SidebarWidget {
    fn render(&mut self, area: Rect, screen: &mut Screen) {
        if !self.visible || area.is_empty() {
            return;
        }

        // Fill background
        screen.fill_rect(
            area,
            Cell {
                char: ' ',
                fg: styles::SIDEBAR_FG,
                bg: styles::SIDEBAR_BG,
                style: Style::default(),
            },
        );

        // Draw left border
        screen.vline(
            area.x,
            area.y,
            area.y + area.height - 1,
            '│',
            styles::SIDEBAR_BORDER,
            styles::SIDEBAR_BG,
        );

        let mut row = area.y + 1;
        let max_width = (area.width as usize).saturating_sub(4); // account for border + padding

        // ── Project section ────────────────────────────────────────────
        row = self.draw_section_header(screen, row, area.x + 2, max_width, "Project");
        row += 1;

        if !self.project_name.is_empty() {
            row = self.draw_labeled_value(
                screen,
                row,
                area.x + 2,
                max_width,
                "Name:",
                &self.project_name,
                Color::WHITE,
            );
        }
        if !self.project_type.is_empty() {
            row = self.draw_labeled_value(
                screen,
                row,
                area.x + 2,
                max_width,
                "Type:",
                &self.project_type,
                Color::Ansi(14),
            );
        }
        if let Some(ref branch) = self.git_branch {
            row = self.draw_labeled_value(
                screen,
                row,
                area.x + 2,
                max_width,
                "Git:",
                branch,
                Color::GREEN,
            );
        }
        if !self.build_command.is_empty() {
            row = self.draw_labeled_value(
                screen,
                row,
                area.x + 2,
                max_width,
                "Build:",
                &self.build_command,
                Color::Ansi(252),
            );
        }
        if !self.test_command.is_empty() {
            row = self.draw_labeled_value(
                screen,
                row,
                area.x + 2,
                max_width,
                "Test:",
                &self.test_command,
                Color::Ansi(252),
            );
        }

        row += 1;

        // ── Pinned files section ────────────────────────────────────────
        if !self.pinned_files.is_empty() {
            row = self.draw_section_header(screen, row, area.x + 2, max_width, "Files");
            row += 1;
            for file in &self.pinned_files {
                if row >= area.y + area.height - 1 {
                    break;
                }
                let display = format!("• {}", file);
                let truncated = if display.len() > max_width {
                    format!("• {}…", &file[..max_width - 3])
                } else {
                    display
                };
                screen.write_str(
                    row,
                    area.x + 2,
                    &truncated,
                    Color::Ansi(252),
                    styles::SIDEBAR_BG,
                    Style::default(),
                );
                row += 1;
            }
            row += 1;
        }

        // ── Active skills section ───────────────────────────────────────
        if !self.active_skills.is_empty() {
            row = self.draw_section_header(screen, row, area.x + 2, max_width, "Skills");
            row += 1;
            for (name, _desc) in &self.active_skills {
                if row >= area.y + area.height - 1 {
                    break;
                }
                let display = format!("⚡ {}", name);
                screen.write_str(
                    row,
                    area.x + 2,
                    &display,
                    Color::CYAN,
                    styles::SIDEBAR_BG,
                    Style::bold(),
                );
                row += 1;
            }
        }
    }

    fn handle_event(&mut self, _event: &Event) -> Action {
        Action::None
    }
}

impl SidebarWidget {
    fn draw_section_header(
        &self,
        screen: &mut Screen,
        row: u16,
        col: u16,
        max_width: usize,
        title: &str,
    ) -> u16 {
        let header = format!("┌─ {} ", title);
        screen.write_str(
            row,
            col,
            &header,
            styles::SIDEBAR_BORDER,
            styles::SIDEBAR_BG,
            Style::bold(),
        );
        // Fill remaining space with ─
        let remaining = max_width.saturating_sub(header.len());
        if remaining > 0 {
            screen.write_str(
                row,
                col + header.len() as u16,
                &"─".repeat(remaining),
                styles::SIDEBAR_BORDER,
                styles::SIDEBAR_BG,
                Style::default(),
            );
        }
        row + 1
    }

    fn draw_labeled_value(
        &self,
        screen: &mut Screen,
        row: u16,
        col: u16,
        max_width: usize,
        label: &str,
        value: &str,
        value_color: Color,
    ) -> u16 {
        screen.write_str(
            row,
            col,
            label,
            Color::Ansi(244),
            styles::SIDEBAR_BG,
            Style::dim(),
        );
        let value_col = col + label.len() as u16 + 1;
        let available = max_width.saturating_sub(label.len() + 1);
        let display = if value.len() > available {
            format!("{}…", &value[..available.saturating_sub(1)])
        } else {
            value.to_string()
        };
        screen.write_str(
            row,
            value_col,
            &display,
            value_color,
            styles::SIDEBAR_BG,
            Style::default(),
        );
        row + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_new() {
        let sidebar = SidebarWidget::new();
        assert!(sidebar.visible);
        assert!(sidebar.project_name.is_empty());
    }

    #[test]
    fn test_sidebar_render() {
        let mut screen = Screen::new(80, 24);
        let mut sidebar = SidebarWidget::new();
        sidebar.project_name = "TinyHarness".to_string();
        sidebar.project_type = "Rust".to_string();
        sidebar.build_command = "cargo build".to_string();
        sidebar.pinned_files = vec!["src/main.rs".to_string(), "Cargo.toml".to_string()];

        let area = Rect::new(60, 1, 20, 22);
        sidebar.render(area, &mut screen);

        // Should have rendered content in the sidebar area
        assert!(screen.get(1, 60).unwrap().char == '│');
    }

    #[test]
    fn test_sidebar_hidden() {
        let mut screen = Screen::new(80, 24);
        let mut sidebar = SidebarWidget::new();
        sidebar.visible = false;

        let area = Rect::new(60, 1, 20, 22);
        sidebar.render(area, &mut screen);

        // Should not have rendered anything
        assert_eq!(screen.get(1, 60).unwrap().char, ' '); // default
    }
}
