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
