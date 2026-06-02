use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Create the main layout: header, body, footer.
pub fn main_layout(frame: &mut Frame) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(1),    // body (task list)
            Constraint::Length(2), // footer
        ])
        .split(frame.area())
        .to_vec()
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
