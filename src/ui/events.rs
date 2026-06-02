use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers, KeyEventKind};
use std::time::Duration;

/// Application events.
#[derive(Debug, Clone)]
pub enum Event {
    /// A terminal input event.
    Key(KeyCode, KeyModifiers),
    /// Terminal resize.
    Resize(u16, u16),
    /// Periodic tick.
    Tick,
}

/// Poll for events with a timeout.
pub fn poll_event(timeout: Duration) -> anyhow::Result<Option<Event>> {
    if event::poll(timeout)? {
        match event::read()? {
            CrosstermEvent::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    Ok(Some(Event::Key(key.code, key.modifiers)))
                } else {
                    Ok(None)
                }
            }
            CrosstermEvent::Resize(cols, rows) => Ok(Some(Event::Resize(cols, rows))),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}
