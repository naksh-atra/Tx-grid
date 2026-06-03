use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Create the main layout: header, body, footer.
/// When notes mode is active, the body is split horizontally:
/// left = task grid, right = notes editor.
pub fn main_layout(frame: &mut Frame, notes_active: bool) -> Vec<Rect> {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(1),    // body
            Constraint::Length(2), // footer
        ])
        .split(frame.area());

    let header = vertical[0];
    let body = vertical[1];
    let footer = vertical[2];

    if notes_active {
        // Split body horizontally: left = tasks, right = notes
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // task grid (left)
                Constraint::Percentage(50), // notes editor (right)
            ])
            .split(body);

        // Return [header, task_area, notes_area, footer]
        vec![header, horizontal[0], horizontal[1], footer]
    } else {
        // Return [header, body, notes_empty, footer]
        let empty = Rect::new(0, 0, 0, 0);
        vec![header, body, empty, footer]
    }
}

/// Create the body layout with optional status message.
pub fn body_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // task list
            Constraint::Length(1), // filter bar (if active)
        ])
        .split(area)
        .to_vec()
}
