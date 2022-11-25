use api::get_tasks;
use config::{get_config, Config};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use key_events::{get_key_event, EventExit};
use menu::{create_chunks, render_key_tabs, render_menu_tabs};
use menu::{render_active_menu_widget, Database, MenuItem};

use project::{render_project_item, ProjectItem, ProjectStatus};
use std::{io, sync::mpsc};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use std::{sync::Mutex, thread};
use task::{render_active_task_input_widget, TaskItem, TaskStatus};
use tui::backend::Backend;
use tui::{backend::CrosstermBackend, Terminal};

pub mod api;
pub mod config;
pub mod input;
pub mod key_events;
pub mod menu;
pub mod navigation;
pub mod project;
pub mod task;
pub mod ui;
use crate::api::get_projects;
use crate::task::AddTaskHighlight;

#[derive(Copy, Clone, Debug)]
enum Event<I> {
    Input(I),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let config: Config = get_config();

    let res = run_app(&mut terminal, config);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, config: Config) -> io::Result<()> {
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

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let mut active_menu_item = MenuItem::Home;
    let mut task_status = TaskStatus::default();
    task_status.task_table_state.select(Some(0));

    let mut project_status = ProjectStatus::default();
    project_status.project_table_state.select(Some(0));

    let database = Arc::new(Mutex::new(Database::default()));
    let database2 = Arc::clone(&database);

    let token2 = config.token.clone();
    tokio::spawn(async move {
        database2.lock().unwrap().projects = get_projects(token2).await.unwrap();
    });

    let database2 = Arc::clone(&database);
    let token2 = config.token.clone();
    tokio::spawn(async move {
        database2.lock().unwrap().tasks = get_tasks(token2).await.unwrap();
    });

    loop {
        terminal.draw(|rect| {
            let size = rect.size();

            let (
                left_right_top,
                left_right_bottom,
                bottom_fullscreen,
                task_chunks_add,
                project_chunks_2,
                project_chunks_add,
            ) = create_chunks(size);

            let menu_tabs = render_menu_tabs(active_menu_item, config.color.clone());
            rect.render_widget(menu_tabs, left_right_top[0]);
            let key_tabs = render_key_tabs(config.color.clone());
            rect.render_widget(key_tabs, left_right_top[1]);

            render_active_menu_widget(
                rect,
                active_menu_item,
                Arc::clone(&database),
                &mut project_status,
                &mut task_status,
                &config,
                left_right_bottom.clone(),
                bottom_fullscreen,
                task_chunks_add,
                project_chunks_2,
                project_chunks_add.clone(),
            );

            if project_status.active_project_item == ProjectItem::Name {
                render_project_item(
                    rect,
                    project_chunks_add.clone(),
                    &project_status.project_item,
                    config.color.clone(),
                );
            }

            render_active_task_input_widget(rect, &task_status, left_right_bottom);
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
                EventExit::Break => break,
                EventExit::Continue => {}
            },
            Event::Tick => {}
        }
    }
    Ok(())
}
