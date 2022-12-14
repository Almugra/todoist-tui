use anyhow::Result;
use api::get_tasks;
use chunks::Chunks;
use config::{get_config, Config};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use key_events::{get_key_event, EventExit};
use menu::{render_active_menu_widget, Database, MenuItem};
use menu::{render_key_tabs, render_menu_tabs};
use project::{render_project_item, ProjectItem, ProjectStatus};
use task::{render_active_task_input_widget, TaskStatus};

use std::{io, sync::mpsc};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use std::{sync::Mutex, thread};
use tui::backend::Backend;
use tui::{backend::CrosstermBackend, Terminal};

pub mod api;
pub mod chunks;
pub mod config;
pub mod handler;
pub mod home;
pub mod input;
pub mod key_events;
pub mod menu;
pub mod navigation;
pub mod project;
pub mod task;
use crate::api::get_projects;

#[derive(Copy, Clone, Debug)]
enum Event<I> {
    Input(I),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let config: Config = get_config();

    let res = run_app(&mut terminal, config);

    if let Err(err) = res {
        println!("{:?}", err)
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    config: Config,
) -> Result<(), anyhow::Error> {
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    let mut active_menu_item = MenuItem::Home;
    let mut task_status = TaskStatus::default();
    task_status.task_table_state.select(Some(0));

    let mut project_status = ProjectStatus::default();
    project_status.project_table_state.select(Some(0));

    let database = Arc::new(Mutex::new(Database::default()));

    let (database_mutex, token_mutex) = (Arc::clone(&database), config.token.clone());
    tokio::spawn(async move {
        database_mutex.lock().unwrap().projects = get_projects(token_mutex).await.unwrap();
    });

    let (database_mutex, token_mutex) = (Arc::clone(&database), config.token.clone());
    tokio::spawn(async move {
        database_mutex.lock().unwrap().tasks = get_tasks(token_mutex).await.unwrap();
    });

    loop {
        terminal.draw(|rect| {
            let size = rect.size();

            let chunks = Chunks::create_chunks(size);

            let menu_tabs = render_menu_tabs(active_menu_item, config.color);
            rect.render_widget(menu_tabs, chunks.menu_or_keybinds[0]);
            let key_tabs = render_key_tabs(config.color);
            rect.render_widget(key_tabs, chunks.menu_or_keybinds[1]);

            render_active_menu_widget(
                rect,
                active_menu_item,
                Arc::clone(&database),
                &mut project_status,
                &mut task_status,
                &config,
                &chunks,
            );

            if project_status.active_project_item == ProjectItem::Name {
                render_project_item(
                    rect,
                    chunks.add_project_with_projects,
                    &project_status.project_item,
                    config.color,
                );
            }

            render_active_task_input_widget(rect, &task_status, chunks.projects_or_tasks);
        })?;

        match rx.recv().unwrap() {
            Event::Input(event) => match get_key_event(
                event,
                &mut active_menu_item,
                &mut task_status,
                &mut project_status,
                &config,
                Arc::clone(&database),
            ) {
                EventExit::Exit => break,
                EventExit::Error(err) => return Err(err),
                EventExit::Continue => {}
            },
            Event::Tick => {}
        }
    }
    Ok(())
}
