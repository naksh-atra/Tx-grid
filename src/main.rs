#![allow(dead_code, unused_imports, unused_variables)]

// NOTE: dead_code allowed for planned API surface not yet fully wired.
// These will be removed as features are completed in v0.2.0.

mod cli;
mod config;
mod logging;
mod models;
mod services;
mod ui;
mod utils;

use anyhow::Context;
use crossterm::{
    cursor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log::{info, warn};
use ratatui::prelude::*;
use std::io::stdout;

use crate::cli::Args;
use crate::config::Config;
use crate::services::task_service::TaskService;
use crate::services::tmux_service;
use crate::ui::events::{poll_event, Event};
use crate::ui::render::draw;
use crate::ui::state::{App, AppMode, ConfirmAction};

fn main() -> anyhow::Result<()> {
    let args = Args::parse_args();

    if args.debug {
        std::env::set_var("RUST_LOG", "debug");
    }
    logging::init();

    if args.doctor {
        return run_doctor();
    }

    // Verify tmux environment
    verify_tmux_env()?;

    let mut task_service = TaskService::new();
    let current_pane = tmux_service::current_pane_id();

    if args.check || args.json {
        return run_non_interactive(&mut task_service, current_pane.as_deref(), args.json);
    }

    run_tui(&mut task_service, current_pane.as_deref())
}

fn verify_tmux_env() -> anyhow::Result<()> {
    let (version, supported) = tmux_service::check_tmux()?;
    if !supported {
        anyhow::bail!("tmux version {} is too old; need 3.2+", version);
    }
    info!("tmux {} detected", version);
    Ok(())
}

fn run_non_interactive(
    service: &mut TaskService,
    current_pane: Option<&str>,
    as_json: bool,
) -> anyhow::Result<()> {
    let tasks = service.discover_tasks(current_pane)?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&tasks)?);
    } else {
        println!("tmux-taskgrid: {} tasks found", tasks.len());
        for task in &tasks {
            let runtime = task
                .runtime
                .map(|r| crate::utils::format::format_duration(r))
                .unwrap_or_else(|| "—".to_string());
            println!(
                "  [{:8}] {:16} {:30} {:>10}",
                task.state.as_str(),
                task.pane.locator(),
                task.command_display,
                runtime
            );
        }
    }
    Ok(())
}

fn run_doctor() -> anyhow::Result<()> {
    println!("=== tmux-taskgrid doctor ===");

    // Check tmux
    print!("tmux: ");
    match tmux_service::check_tmux() {
        Ok((ver, supported)) => {
            if supported {
                println!("{} [OK]", ver);
            } else {
                println!("{} [TOO OLD — need 3.2+]", ver);
            }
        }
        Err(e) => println!("[NOT FOUND] {}", e),
    }

    // Check if inside tmux
    print!("inside tmux: ");
    if std::env::var("TMUX").is_ok() {
        println!("yes [OK]");
    } else {
        println!("no [WARN — taskgrid works best inside tmux]");
    }

    // Check for popup support
    print!("popup support: ");
    match std::process::Command::new("tmux")
        .args(["display-popup", "-h"])
        .output()
    {
        Ok(out) if out.status.success() || String::from_utf8_lossy(&out.stderr).contains("usage") => {
            println!("yes [OK]");
        }
        _ => {
            println!("no [WARN — display-popup not available]");
        }
    }

    // Try listing panes
    print!("pane discovery: ");
    match tmux_service::list_panes() {
        Ok(panes) => println!("{} panes found [OK]", panes.len()),
        Err(e) => println!("[FAILED] {}", e),
    }

    println!("\n=== doctor complete ===");
    Ok(())
}

fn run_tui(service: &mut TaskService, current_pane: Option<&str>) -> anyhow::Result<()> {
    let tasks = service
        .discover_tasks(current_pane)
        .context("initial task discovery failed")?;

    let config = Config::from_tmux();
    let mut app = App::new(tasks);
    app.refresh_interval = config.refresh_interval;
    app.apply_filter_and_sort();

    // Initialize terminal
    let mut stdout = stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    stdout
        .execute(EnterAlternateScreen)
        .context("failed to enter alternate screen")?;
    stdout
        .execute(cursor::Hide)
        .context("failed to hide cursor")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    let result = run_app_loop(&mut terminal, &mut app, service, current_pane);

    // Cleanup terminal
    disable_raw_mode().ok();
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .ok();
    terminal.backend_mut().execute(cursor::Show).ok();

    result
}

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    service: &mut TaskService,
    current_pane: Option<&str>,
) -> anyhow::Result<()> {
    let tick_rate = std::time::Duration::from_millis(250);

    loop {
        terminal.draw(|f| draw(f, app))?;

        let event = poll_event(tick_rate)?;

        match event {
            Some(Event::Key(code, modifiers)) => {
                if handle_key(app, code, modifiers, service, current_pane)? {
                    break;
                }
            }
            Some(Event::Resize(_, _)) => {
                // Terminal will redraw on next iteration
            }
            Some(Event::Tick) | None => {
                if app.should_refresh() {
                    refresh_tasks(app, service, current_pane);
                }
            }
        }
    }

    Ok(())
}

/// Handle a key event. Returns true if the app should quit.
fn handle_key(
    app: &mut App,
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
    service: &mut TaskService,
    current_pane: Option<&str>,
) -> anyhow::Result<bool> {
    match app.mode {
        AppMode::Normal => match code {
            crossterm::event::KeyCode::Char('c')
                if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                return Ok(true);
            }
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                return Ok(true);
            }
            crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                app.move_down(20);
            }
            crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                app.move_up();
            }
            crossterm::event::KeyCode::Char('g') | crossterm::event::KeyCode::Home => {
                app.move_to_first();
            }
            crossterm::event::KeyCode::Char('G') | crossterm::event::KeyCode::End => {
                app.move_to_last(20);
            }
            crossterm::event::KeyCode::PageUp => {
                app.page_up(10);
            }
            crossterm::event::KeyCode::PageDown => {
                app.page_down(10, 20);
            }
            crossterm::event::KeyCode::Char('/') => {
                app.enter_filter_mode();
            }
            crossterm::event::KeyCode::Char('s') => {
                app.cycle_sort();
                app.set_status(format!("sort: {:?}", app.sort_order));
            }
            crossterm::event::KeyCode::Enter => {
                if let Some(pane_id) = app.selected_task().map(|t| t.pane.pane_id.clone()) {
                    app.set_status(format!("jumped to {}", pane_id.as_str()));
                    disable_raw_mode().ok();
                    if let Err(e) = tmux_service::select_pane(&pane_id) {
                        app.set_status(format!("jump failed: {}", e));
                    }
                    enable_raw_mode().ok();
                    return Ok(true);
                }
            }
            crossterm::event::KeyCode::Char('x') => {
                app.prompt_kill();
            }
            crossterm::event::KeyCode::Char('r') => {
                app.enter_rename_mode();
            }
            _ => {}
        },
        AppMode::Filter => match code {
            crossterm::event::KeyCode::Esc => {
                app.exit_filter_mode();
                app.clear_status();
            }
            crossterm::event::KeyCode::Enter => {
                app.exit_filter_mode();
                app.set_status(format!("filter: {}", app.filter_text));
            }
            crossterm::event::KeyCode::Char(c) => {
                app.append_filter(c);
            }
            crossterm::event::KeyCode::Backspace => {
                app.backspace_filter();
            }
            _ => {}
        },
        AppMode::Confirm => match code {
            crossterm::event::KeyCode::Char('y') => {
                if let Some(ConfirmAction::Kill(ref pane_id)) = app.confirm() {
                    let pid = crate::models::pane::PaneId::new(pane_id.clone());
                    match tmux_service::kill_pane(&pid) {
                        Ok(()) => {
                            app.set_status("pane killed");
                            refresh_tasks(app, &mut *service, current_pane);
                        }
                        Err(e) => {
                            app.set_status(format!("kill failed: {}", e));
                        }
                    }
                }
            }
            crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Esc => {
                app.cancel_confirm();
            }
            _ => {}
        },
        AppMode::Rename => match code {
            crossterm::event::KeyCode::Esc => {
                app.exit_rename_mode();
                app.set_status("rename cancelled");
            }
            crossterm::event::KeyCode::Enter => {
                if let Some(task) = app.selected_task() {
                    let new_name = app.get_rename_text().trim().to_string();
                    if !new_name.is_empty() {
                        match crate::services::tmux_service::tmux_command(&[
                            "select-pane",
                            "-t",
                            task.pane.pane_id.as_str(),
                            "-T",
                            &new_name,
                        ]) {
                            Ok(_) => {
                                // Update in-memory so grid reflects the change immediately
                                if let Some(idx) = app.filtered_indices.get(app.selection) {
                                    app.tasks[*idx].pane.pane_title = new_name.clone();
                                }
                                app.set_status(format!("renamed to {}", new_name));
                            }
                            Err(e) => {
                                app.set_status(format!("rename failed: {}", e));
                            }
                        }
                    }
                }
                app.exit_rename_mode();
            }
            crossterm::event::KeyCode::Char(c) => {
                app.append_rename(c);
            }
            crossterm::event::KeyCode::Backspace => {
                app.backspace_rename();
            }
            _ => {}
        },
        AppMode::Notes => match code {
            crossterm::event::KeyCode::Esc => {
                app.exit_notes_mode();
                app.set_status("notes saved");
            }
            crossterm::event::KeyCode::Enter => {
                app.append_notes('\n');
            }
            crossterm::event::KeyCode::Char(c) => {
                app.append_notes(c);
            }
            crossterm::event::KeyCode::Backspace => {
                app.backspace_notes();
            }
            _ => {}
        },
    }

    Ok(false)
}

fn refresh_tasks(app: &mut App, service: &mut TaskService, current_pane: Option<&str>) {
    match service.discover_tasks(current_pane) {
        Ok(tasks) => {
            app.tasks = tasks;
            app.apply_filter_and_sort();
            app.mark_refreshed();
        }
        Err(e) => {
            warn!("refresh failed: {}", e);
            app.set_status(format!("refresh failed: {}", e));
        }
    }
}


