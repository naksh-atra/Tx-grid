use crate::models::task::TaskState;
use crate::ui::state::{App, AppMode};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

// Render the entire application UI.
pub fn draw(f: &mut Frame, app: &App) {
let notes_active = app.mode == AppMode::Notes;
let chunks = crate::ui::layout::main_layout(f, notes_active);

draw_header(f, chunks[0], app);
draw_task_list(f, chunks[1], app, notes_active);

if notes_active && chunks.len() > 2 && chunks[2].width > 0 {
    draw_notes_panel(f, chunks[2], app);
}

draw_footer(f, chunks[chunks.len() - 1], app);

// Draw confirmation dialog if needed
if app.mode == AppMode::Confirm {
    draw_confirm_dialog(f, app);
}
}

fn draw_header(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let title = format!(
        " tmux-taskgrid — {} tasks ",
        app.filtered_indices.len()
    );

    let sort_label = match app.sort_order {
        crate::ui::state::SortOrder::Runtime => "runtime",
        crate::ui::state::SortOrder::Session => "session",
        crate::ui::state::SortOrder::State => "state",
    };

    let subtitle = format!("[sort: {}] [s] cycle sort", sort_label);

    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            &title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            subtitle,
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(header, area);
}

fn draw_task_list(f: &mut Frame, area: ratatui::layout::Rect, app: &App, notes_active: bool) {
    if app.filtered_indices.is_empty() {
        let empty_msg = if app.tasks.is_empty() {
            "No panes found. Press q to quit."
        } else {
            "No tasks match the filter. Press Esc to clear filter."
        };
        let msg = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(msg, area);
        return;
    }

    let header = if notes_active {
        Row::new(vec!["#", "Pane", "Command"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Row::new(vec!["#", "Pane", "Command", "Runtime", "State"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    };

    // Build rows with window group headers
    let mut rows: Vec<Row> = Vec::new();
    let mut last_window: Option<String> = None;
    let mut display_idx: usize = 0;

    for task_idx in &app.filtered_indices {
        let task = &app.tasks[*task_idx];
        let window_key = format!("{}:{}", task.pane.session_name, task.pane.window_index);

        // Insert a window header row when we encounter a new window
        if last_window.as_ref() != Some(&window_key) {
            let window_label = if task.pane.window_name.is_empty() {
                format!("▸ {}:{}", task.pane.session_name, task.pane.window_index)
            } else {
                format!("▸ {}:{} ({})", task.pane.session_name, task.pane.window_index, task.pane.window_name)
            };
            rows.push(
                Row::new(vec![Cell::from(window_label)])
                    .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
            );
            last_window = Some(window_key);
        }

        let is_selected = display_idx == app.selection;

        let state_style = match task.state {
            TaskState::Running => Style::default().fg(Color::Green),
            TaskState::Exited => Style::default().fg(Color::Red),
            TaskState::Idle => Style::default().fg(Color::DarkGray),
            TaskState::Unknown => Style::default().fg(Color::Yellow),
        };

        let pane_name = if task.pane.pane_title.is_empty() {
            task.pane.locator()
        } else {
            task.pane.pane_title.clone()
        };

        let row = if notes_active {
            Row::new(vec![
                Cell::from(format!("{}", display_idx + 1)),
                Cell::from(pane_name),
                Cell::from(task.command_display.clone()),
            ])
        } else {
            let runtime_str = task
                .runtime
                .map(|r| format_runtime(r))
                .unwrap_or_else(|| "—".to_string());

            Row::new(vec![
                Cell::from(format!("{}", display_idx + 1)),
                Cell::from(pane_name),
                Cell::from(task.command_display.clone()),
                Cell::from(runtime_str),
                Cell::from(task.state.as_str()).style(state_style),
            ])
        };

            let styled_row = if task.pane.window_name.starts_with("▸") {
                // Window header row — already styled above
                row
            } else if is_selected {
                let bg = if app.mode == AppMode::Rename {
                    Color::Yellow
                } else if app.mode == AppMode::Notes {
                    Color::Cyan
                } else {
                    Color::DarkGray
                };
                row.style(Style::default().bg(bg).add_modifier(Modifier::BOLD))
            } else {
                row
            };

        rows.push(styled_row);
        display_idx += 1;
    }

    let table = if notes_active {
        Table::new(
            rows,
            [
                ratatui::layout::Constraint::Length(4),
                ratatui::layout::Constraint::Length(12),
                ratatui::layout::Constraint::Min(15),
            ],
        )
    } else {
        Table::new(
            rows,
            [
                ratatui::layout::Constraint::Length(4),
                ratatui::layout::Constraint::Length(16),
                ratatui::layout::Constraint::Min(20),
                ratatui::layout::Constraint::Length(10),
                ratatui::layout::Constraint::Length(8),
            ],
        )
    }
    .header(header)
    .block(Block::default().borders(Borders::NONE))
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );


    f.render_widget(table, area);
}

fn draw_footer(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let content = match app.mode {
        AppMode::Filter => {
            Line::from(vec![
                Span::styled(
                    " Filter: ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(&app.filter_text),
                Span::styled(" | Esc: cancel | Enter: apply", Style::default().fg(Color::DarkGray)),
            ])
        }
        AppMode::Confirm => {
            Line::from(vec![
                Span::styled(
                    " Confirm kill? (y/n) ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ])
        }
        AppMode::Rename => {
            Line::from(vec![
                Span::styled(" Rename: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(&app.rename_text),
                Span::styled(" | Esc: cancel | Enter: apply", Style::default().fg(Color::DarkGray)),
            ])
        }
        AppMode::Notes => {
            let display = if app.notes_text.len() > 40 {
                format!("{}...", &app.notes_text[..40])
            } else {
                app.notes_text.clone()
            };
            Line::from(vec![
                Span::styled(" Notes: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(display),
                Span::styled(" | Esc: save & quit", Style::default().fg(Color::DarkGray)),
            ])
        }
        AppMode::Normal => {
            let help = " j/k: move | Enter: jump | x: kill | r: rename | n: notes | /: filter | s: sort | q: quit ";
            let status = app
                .status_message
                .as_ref()
                .map(|s| format!(" | {}", s))
                .unwrap_or_default();
            Line::from(vec![
                Span::styled(help, Style::default().fg(Color::DarkGray)),
                Span::styled(status, Style::default().fg(Color::Yellow)),
            ])
        }
    };

    let footer = Paragraph::new(content).block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, area);
}

fn draw_notes_panel(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let pane_label = app
        .selected_task()
        .map(|t| format!("{} — {}", t.pane.locator(), t.pane.pane_title))
        .unwrap_or_else(|| "Notes".to_string());

    let max_width = area.width.saturating_sub(4) as usize;
    let notes_text = &app.notes_text;
    let mut lines: Vec<Line> = Vec::new();

    if notes_text.is_empty() {
        // Placeholder text with cursor at end
        let mut placeholder_line = Line::from(Span::styled(
            "Type notes for this pane...",
            Style::default().fg(Color::DarkGray),
        ));
        placeholder_line.spans.push(Span::styled(
            " ▌",
            Style::default().fg(Color::Cyan),
        ));
        lines.push(placeholder_line);
    } else {
        // Word-wrap the notes text
        let mut current_line = String::new();
        for ch in notes_text.chars() {
            if ch == '\n' {
                lines.push(Line::from(vec![Span::raw(current_line.clone())]));
                current_line.clear();
            } else if current_line.len() >= max_width {
                lines.push(Line::from(vec![Span::raw(current_line.clone())]));
                current_line.clear();
                current_line.push(ch);
            } else {
                current_line.push(ch);
            }
        }
        // Last line — text + blinking cursor as separate spans
        let text = current_line.clone();
        let cursor_span = Span::styled("▌", Style::default().fg(Color::Cyan));
        lines.push(Line::from(vec![
            Span::raw(text),
            cursor_span,
        ]));
    }

    let notes_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" Notes: {} ", pane_label))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(notes_widget, area);
}

fn draw_confirm_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, f.area());
    f.render_widget(Clear, area);

    let task_info = app
        .selected_task()
        .map(|t| format!("Kill pane {} ({})?", t.pane.locator(), t.command_display))
        .unwrap_or_else(|| "Kill selected pane?".to_string());

    let dialog = Paragraph::new(vec![
        Line::from(Span::styled(
            task_info,
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            " y: yes  n: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );

    f.render_widget(dialog, area);
}

fn centered_rect(width: u16, height: u16, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    ratatui::layout::Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
        ..area
    }
}

/// Format runtime in human-readable form.
fn format_runtime(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_runtime() {
        assert_eq!(format_runtime(45), "45s");
        assert_eq!(format_runtime(125), "2m 5s");
        assert_eq!(format_runtime(3661), "1h 1m");
        assert_eq!(format_runtime(90061), "1d 1h");
    }
}
