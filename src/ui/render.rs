use crate::models::task::TaskState;
use crate::ui::state::{App, AppMode};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

/// Render the entire application UI.
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = crate::ui::layout::main_layout(f);

    draw_header(f, chunks[0], app);
    draw_task_list(f, chunks[1], app);
    draw_footer(f, chunks[2], app);

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

fn draw_task_list(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
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

    let header = Row::new(vec!["#", "Pane", "Command", "Runtime", "State"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(display_idx, task_idx)| {
            let task = &app.tasks[*task_idx];
            let is_selected = display_idx == app.selection;

            let state_style = match task.state {
                TaskState::Running => Style::default().fg(Color::Green),
                TaskState::Exited => Style::default().fg(Color::Red),
                TaskState::Idle => Style::default().fg(Color::DarkGray),
                TaskState::Unknown => Style::default().fg(Color::Yellow),
            };

            let runtime_str = task
                .runtime
                .map(|r| format_runtime(r))
                .unwrap_or_else(|| "—".to_string());

            let row = Row::new(vec![
                Cell::from(format!("{}", display_idx + 1)),
                Cell::from(task.pane.locator()),
                Cell::from(task.command_display.clone()),
                Cell::from(runtime_str),
                Cell::from(task.state.as_str()).style(state_style),
            ]);

            if is_selected {
                row.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Length(4),
            ratatui::layout::Constraint::Length(16),
            ratatui::layout::Constraint::Min(20),
            ratatui::layout::Constraint::Length(10),
            ratatui::layout::Constraint::Length(8),
        ],
    )
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
            Line::from(vec![Span::styled(
                " Confirm kill? (y/n) ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )])
        }
        AppMode::Rename => {
            Line::from(vec![
                Span::styled(
                    " Rename: ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(&app.rename_text),
                Span::styled(
                    " | Esc: cancel | Enter: apply",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        }
        AppMode::Normal => {
            let help = " j/k: move | Enter: jump | x: kill | r: rename | /: filter | s: sort | q: quit ";
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
