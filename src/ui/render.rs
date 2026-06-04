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
    let title = format!(" tmux-taskgrid — {} tasks ", app.filtered_indices.len());

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
        Line::from(Span::styled(subtitle, Style::default().fg(Color::DarkGray))),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(header, area);
}

fn draw_task_list(f: &mut Frame, area: ratatui::layout::Rect, app: &App, notes_active: bool) {
    if app.filtered_indices.is_empty() {
        let empty_msg = if app.tasks.is_empty() {
            vec![
                Line::from(Span::styled(
                    "No panes found.",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "Create some panes in tmux first, then press q to quit.",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            vec![
                Line::from(Span::styled(
                    format!("No tasks match the filter: \"{}\"", app.filter_text),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "Press Esc to clear the filter.",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        };
        let msg = Paragraph::new(empty_msg).block(Block::default().borders(Borders::NONE));
        f.render_widget(msg, area);
        return;
    }

    let header = if notes_active {
        Row::new(vec!["#", "Pane", "Command"]).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Row::new(vec!["#", "Pane", "Command", "Runtime", "State"]).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    };

    // Build rows with window group headers
    let mut rows: Vec<Row> = Vec::new();
    let mut last_window: Option<String> = None;

    for (display_idx, task_idx) in app.filtered_indices.iter().enumerate() {
        let task = &app.tasks[*task_idx];
        let window_key = format!("{}:{}", task.pane.session_name, task.pane.window_index);

        // Insert a window header row when we encounter a new window
        if last_window.as_ref() != Some(&window_key) {
            let window_label = if task.pane.window_name.is_empty() {
                format!("▸ {}:{}", task.pane.session_name, task.pane.window_index)
            } else {
                format!(
                    "▸ {}:{} ({})",
                    task.pane.session_name, task.pane.window_index, task.pane.window_name
                )
            };
            // Full-width header: put label in first cell, empty in rest
            // The table column constraints will handle width
            if notes_active {
                rows.push(Row::new(vec![
                    Cell::from(window_label).style(
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ),
                    Cell::from(""),
                    Cell::from(""),
                ]));
            } else {
                rows.push(Row::new(vec![
                    Cell::from(window_label).style(
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ]));
            }
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
                .map(format_runtime)
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
    }

    // Use flexible constraints so window headers can expand
    let table = if notes_active {
        Table::new(
            rows,
            [
                ratatui::layout::Constraint::Percentage(30),
                ratatui::layout::Constraint::Percentage(20),
                ratatui::layout::Constraint::Percentage(50),
            ],
        )
    } else {
        Table::new(
            rows,
            [
                ratatui::layout::Constraint::Percentage(25),
                ratatui::layout::Constraint::Percentage(15),
                ratatui::layout::Constraint::Percentage(30),
                ratatui::layout::Constraint::Percentage(15),
                ratatui::layout::Constraint::Percentage(15),
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
    // Build a two-line footer: mode-specific hints on top, global info on bottom
    let mode_line = match app.mode {
        AppMode::Filter => Line::from(vec![
            Span::styled(
                " Filter: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&app.filter_text),
            Span::styled(
                " | Esc: cancel | Enter: apply",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        AppMode::Confirm => Line::from(vec![Span::styled(
            " Confirm kill? (y/n) ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        AppMode::Rename => Line::from(vec![
            Span::styled(
                " Rename: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&app.rename_text),
            Span::styled(
                " | Esc: cancel | Enter: apply",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        AppMode::Notes => {
            let display = if app.notes_text.len() > 40 {
                format!("{}...", &app.notes_text[..40])
            } else {
                app.notes_text.clone()
            };
            Line::from(vec![
                Span::styled(
                    " Notes: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(display),
                Span::styled(" | Esc: save & quit", Style::default().fg(Color::DarkGray)),
            ])
        }
        AppMode::Normal => {
            let help = " j/k: move | Enter: jump | x: kill | r: rename | n: notes | /: filter | s: sort | q: quit ";
            Line::from(Span::styled(help, Style::default().fg(Color::DarkGray)))
        }
    };

    // Second line: status message + filter indicator + sort mode
    let sort_label = match app.sort_order {
        crate::ui::state::SortOrder::Runtime => "runtime",
        crate::ui::state::SortOrder::Session => "session",
        crate::ui::state::SortOrder::State => "state",
    };

    let mut status_parts: Vec<Span> = Vec::new();

    if let Some(ref msg) = app.status_message {
        status_parts.push(Span::styled(
            msg.clone(),
            Style::default().fg(Color::Yellow),
        ));
        status_parts.push(Span::raw("  "));
    }

    if !app.filter_text.is_empty() {
        status_parts.push(Span::styled(
            format!("filter: \"{}\"", app.filter_text),
            Style::default().fg(Color::Cyan),
        ));
        status_parts.push(Span::raw("  "));
    }

    status_parts.push(Span::styled(
        format!("sort: {}  [s] cycle", sort_label),
        Style::default().fg(Color::DarkGray),
    ));

    let status_line = Line::from(status_parts);

    let footer =
        Paragraph::new(vec![mode_line, status_line]).block(Block::default().borders(Borders::TOP));
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
        placeholder_line
            .spans
            .push(Span::styled(" ▌", Style::default().fg(Color::Cyan)));
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
        lines.push(Line::from(vec![Span::raw(text), cursor_span]));
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
    // Centered modal overlay
    let area = centered_rect(60, 7, f.area());
    f.render_widget(Clear, area);

    let task_info = app
        .selected_task()
        .map(|t| {
            format!(
                "Kill pane {} ({})
Running: {}",
                t.pane.locator(),
                t.command_display,
                t.runtime
                    .map(format_runtime)
                    .unwrap_or_else(|| "—".to_string())
            )
        })
        .unwrap_or_else(|| "Kill selected pane?".to_string());

    let dialog = Paragraph::new(vec![
        Line::from(Span::styled("⚠ ", Style::default().fg(Color::Red))),
        Line::from(Span::styled(task_info, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(
            "  y: yes    n: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .title(" Confirm Kill ")
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
