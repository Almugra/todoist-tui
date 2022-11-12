use api::{delete_task, get_tasks, post_projects, Task};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
#[allow(unused_imports)]
use std::{collections::HashMap, thread};
use std::{io, sync::mpsc};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
#[allow(unused_imports)]
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};
#[allow(unused_imports)]
use tui::{style, symbols};
use ui::{render_key_tabs, render_menu_tabs};
pub mod api;
pub mod config;
pub mod ui;
#[allow(unused_imports)]
use crate::api::{delete_project, get_projects, post_task, Project};
use crate::config::Config;

#[derive(Copy, Clone, Debug)]
enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Home,
    Projects,
    Tasks,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Projects => 1,
            MenuItem::Tasks => 2,
        }
    }
}

#[allow(unused_must_use)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = Arc::new(Mutex::new(Config::get_token()));
    let projects = Arc::new(Mutex::new(
        get_projects(&token.lock().unwrap().token).await.unwrap(),
    ));
    let mut tasks = get_tasks(&token.lock().unwrap().token).await.unwrap();
    enable_raw_mode().expect("can runin raw mode");
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

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut active_menu_item = MenuItem::Home;
    let mut project_list_state = ListState::default();
    project_list_state.select(Some(0));
    let mut task_list_state = ListState::default();
    task_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(size);

            let menu_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[0]);

            let menu_tabs = render_menu_tabs(active_menu_item);
            rect.render_widget(menu_tabs, menu_chunks[0]);
            let key_tabs = render_key_tabs();
            rect.render_widget(key_tabs, menu_chunks[1]);

            match active_menu_item {
                MenuItem::Home => rect.render_widget(ui::render_home(), chunks[1]),
                MenuItem::Projects => {
                    let project_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = ui::render_projects(
                        &project_list_state,
                        projects.lock().unwrap().clone(),
                        tasks.clone(),
                    );
                    rect.render_stateful_widget(left, project_chunks[0], &mut project_list_state);
                    rect.render_widget(right, project_chunks[1]);
                }
                MenuItem::Tasks => {
                    let project_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = ui::render_projects(
                        &project_list_state,
                        projects.lock().unwrap().clone(),
                        tasks.clone(),
                    );
                    rect.render_widget(left, project_chunks[0]);
                    rect.render_stateful_widget(right, project_chunks[1], &mut task_list_state);
                }
            }
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => match active_menu_item {
                    MenuItem::Home => active_menu_item = MenuItem::Projects,
                    MenuItem::Projects => active_menu_item = MenuItem::Home,
                    MenuItem::Tasks => {
                        active_menu_item = MenuItem::Projects;
                    }
                },
                KeyCode::Char('l') => match active_menu_item {
                    MenuItem::Home => active_menu_item = MenuItem::Projects,
                    MenuItem::Projects => {
                        let project_id =
                            &projects.lock().unwrap()[project_list_state.selected().unwrap()].id;
                        let tasks_from_project: Vec<_> = tasks
                            .iter()
                            .filter(|x| x.project_id == project_id.clone())
                            .collect();
                        if tasks_from_project.len() != 0 {
                            active_menu_item = MenuItem::Tasks;
                            task_list_state.select(Some(0));
                        }
                    }
                    MenuItem::Tasks => {
                        active_menu_item = MenuItem::Projects;
                    }
                },
                KeyCode::Char('p') => match active_menu_item {
                    MenuItem::Projects => {
                        let token3 = token.lock().unwrap().token.clone();
                        let projects2 = Arc::clone(&projects);
                        projects2.lock().unwrap().push(Project::name("PogU"));
                        let token2 = Arc::clone(&token);
                        let token2 = &token2.lock().unwrap().token;
                        post_projects(&token3, "PogU").await;
                        let projects2 = Arc::clone(&projects);
                        *projects2.lock().unwrap() = get_projects(&token2).await.unwrap();
                    }
                    _ => {}
                },
                KeyCode::Char('a') => match active_menu_item {
                    MenuItem::Projects => {
                        let mut map: HashMap<&str, &str> = HashMap::new();
                        let projects2 = Arc::clone(&projects);
                        let current_selected_project =
                            &projects2.lock().unwrap()[project_list_state.selected().unwrap()].id;
                        tasks.push(Task::new(
                            String::from("funny lol"),
                            current_selected_project.to_owned().clone(),
                        ));
                        map.insert("project_id", &current_selected_project);
                        post_task("funny lol", &mut map).await;
                        tasks = get_tasks(&token.lock().unwrap().token).await.unwrap();
                    }
                    _ => {}
                },
                KeyCode::Char('d') => match active_menu_item {
                    MenuItem::Tasks => {
                        if let Some(selected) = task_list_state.selected() {
                            let id = tasks[selected].id.to_owned();
                            let projects2 = Arc::clone(&projects);
                            let project_id = &projects2.lock().unwrap()
                                [project_list_state.selected().unwrap()]
                            .id;
                            let token2 = token.lock().unwrap().token.clone();
                            tokio::spawn(async move { delete_task(&token2, id).await });
                            tasks.remove(selected);
                            if tasks.len() == 0 {
                                active_menu_item = MenuItem::Projects;
                            }
                            if selected > 0 {
                                task_list_state.select(Some(selected - 1));
                            } else {
                                task_list_state.select(Some(0));
                            }
                        }
                    }
                    MenuItem::Projects => {
                        if let Some(selected) = project_list_state.selected() {
                            if selected == 0 {
                                continue;
                            }
                            let id = projects.lock().unwrap()[selected].id.to_owned();
                            let token2 = token.lock().unwrap().token.clone();
                            tokio::spawn(async move { delete_project(&token2, id).await });
                            projects.lock().unwrap().remove(selected);
                            if selected > 0 {
                                project_list_state.select(Some(selected - 1));
                            } else {
                                project_list_state.select(Some(0));
                            }
                        }
                    }
                    _ => {}
                },
                KeyCode::Char('j') => match active_menu_item {
                    MenuItem::Projects => {
                        if let Some(selected) = project_list_state.selected() {
                            let project_amount = projects.lock().unwrap().len();
                            if selected >= project_amount - 1 {
                                project_list_state.select(Some(0));
                            } else {
                                project_list_state.select(Some(selected + 1));
                            }
                        }
                    }
                    MenuItem::Tasks => {
                        if let Some(selected) = task_list_state.selected() {
                            let selected_project = projects
                                .lock()
                                .unwrap()
                                .clone()
                                .get(
                                    project_list_state
                                        .selected()
                                        .expect("there is always a selected project"),
                                )
                                .expect("exists")
                                .clone();

                            let task_items: Vec<_> = tasks
                                .clone()
                                .iter()
                                .filter(|task| task.project_id == selected_project.id)
                                .map(|task| {
                                    ListItem::new(Spans::from(vec![Span::styled(
                                        task.content.clone(),
                                        Style::default(),
                                    )]))
                                })
                                .collect();
                            let amount_tasks = task_items.len();
                            if selected >= amount_tasks - 1 {
                                task_list_state.select(Some(0));
                            } else {
                                task_list_state.select(Some(selected + 1));
                            }
                        }
                    }
                    _ => {}
                },
                KeyCode::Char('k') => match active_menu_item {
                    MenuItem::Projects => {
                        if let Some(selected) = project_list_state.selected() {
                            let project_amount = projects.lock().unwrap().len();
                            if selected > 0 {
                                project_list_state.select(Some(selected - 1));
                            } else {
                                project_list_state.select(Some(project_amount - 1));
                            }
                        }
                    }
                    MenuItem::Tasks => {
                        if let Some(selected) = task_list_state.selected() {
                            let selected_project = projects
                                .lock()
                                .unwrap()
                                .clone()
                                .get(
                                    project_list_state
                                        .selected()
                                        .expect("there is always a selected project"),
                                )
                                .expect("exists")
                                .clone();

                            let task_items: Vec<_> = tasks
                                .clone()
                                .iter()
                                .filter(|task| task.project_id == selected_project.id)
                                .map(|task| {
                                    ListItem::new(Spans::from(vec![Span::styled(
                                        task.content.clone(),
                                        Style::default(),
                                    )]))
                                })
                                .collect();
                            let amount_tasks = task_items.len();
                            if selected > 0 {
                                task_list_state.select(Some(selected - 1));
                            } else {
                                task_list_state.select(Some(amount_tasks - 1));
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
