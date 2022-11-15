use api::{delete_task, get_tasks, post_projects, Task};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
#[allow(unused_imports)]
use std::{collections::HashMap, thread};
use std::{io, sync::mpsc, vec};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::Mutex as TMutex;
use tui::widgets::TableState;
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

#[derive(Copy, Clone, Debug)]
pub struct AddTaskHighlight {
    pub name: Color,
    pub desc: Color,
    pub prio: Color,
}

impl Default for AddTaskHighlight {
    fn default() -> Self {
        Self {
            name: Color::White,
            desc: Color::White,
            prio: Color::White,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AddTaskItem {
    Name,
    Desc,
    Prio,
}

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
    AddProject,
    AddTask,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Projects => 1,
            MenuItem::Tasks => 2,
            MenuItem::AddTask => 3,
            MenuItem::AddProject => 4,
        }
    }
}

#[allow(unused_must_use, unused_variables)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can runin raw mode");
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    let project_id_tasks: HashMap<String, Vec<String>> = HashMap::new();

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
    let mut project_list_state = TableState::default();
    project_list_state.select(Some(0));
    let mut task_list_state = TableState::default();
    task_list_state.select(Some(0));

    let mut highlight = AddTaskHighlight::default();
    let projects = Arc::new(Mutex::new(vec![Project::name("Loading...")]));
    let mut tasks = Arc::new(Mutex::new(vec![Task::new(
        "Loading...".to_string(),
        "".to_string(),
    )]));
    let projects2 = Arc::clone(&projects);
    tokio::spawn(async move {
        *projects2.lock().unwrap() = get_projects().await.unwrap();
    });
    let tasks2 = Arc::clone(&tasks);
    tokio::spawn(async move {
        *tasks2.lock().unwrap() = get_tasks().await.unwrap();
        tasks2
            .lock()
            .unwrap()
            .sort_by(|a, b| a.project_id.cmp(&b.project_id));
    });

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

            let project_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[1]);

            let menu_tabs = render_menu_tabs(active_menu_item);
            rect.render_widget(menu_tabs, menu_chunks[0]);
            let key_tabs = render_key_tabs();
            rect.render_widget(key_tabs, menu_chunks[1]);

            match active_menu_item {
                MenuItem::Home => rect.render_widget(ui::render_home(), chunks[1]),
                MenuItem::Projects => {
                    let (left, right) = ui::render_projects(
                        &project_list_state,
                        projects.lock().unwrap().clone(),
                        &mut tasks.lock().unwrap().clone(),
                    );
                    rect.render_stateful_widget(left, project_chunks[0], &mut project_list_state);
                    rect.render_widget(right, project_chunks[1]);
                }
                MenuItem::Tasks => {
                    let (left, right) = ui::render_projects(
                        &project_list_state,
                        projects.lock().unwrap().clone(),
                        &mut tasks.lock().unwrap().clone(),
                    );
                    rect.render_stateful_widget(left, project_chunks[0], &mut project_list_state);
                    rect.render_stateful_widget(right, project_chunks[1], &mut task_list_state);
                }
                MenuItem::AddTask => {
                    let task_width_33 = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(33),
                                Constraint::Percentage(33),
                                Constraint::Percentage(33),
                            ]
                            .as_ref(),
                        )
                        .split(project_chunks[1]);

                    let task_width_full = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(100)].as_ref())
                        .split(project_chunks[1]);

                    let (left, right) = ui::render_projects(
                        &project_list_state,
                        projects.lock().unwrap().clone(),
                        &mut tasks.lock().unwrap().clone(),
                    );

                    let desc_width = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                        ])
                        .split(task_width_full[0]);

                    let add_task_chunks_left = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                        ])
                        .split(task_width_33[0]);

                    let add_task_chunks_mid = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                            Constraint::Percentage(8),
                        ])
                        .split(task_width_33[1]);

                    rect.set_cursor(desc_width[0].x + 1 + 6, desc_width[0].y + 1);
                    let (outer, name, desc, prio, label, due) = ui::render_add_tasks(&highlight);
                    let name_paragraph = Paragraph::new("Buy milk")
                        .style(Style::default().fg(Color::White))
                        .block(name);
                    rect.render_stateful_widget(left, project_chunks[0], &mut project_list_state);
                    rect.render_widget(outer, project_chunks[1]);
                    rect.render_widget(name_paragraph, desc_width[0]);
                    rect.render_widget(desc, desc_width[1]);
                    rect.render_widget(label, desc_width[2]);
                    rect.render_widget(prio, add_task_chunks_left[3]);
                    rect.render_widget(due, add_task_chunks_mid[3]);
                }
                MenuItem::AddProject => {}
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
                    MenuItem::Projects => {
                        active_menu_item = MenuItem::Home;
                        project_list_state.select(Some(0));
                    }
                    MenuItem::Tasks => {
                        active_menu_item = MenuItem::Projects;
                    }
                    _ => {}
                },
                KeyCode::Char('l') => match active_menu_item {
                    MenuItem::Home => active_menu_item = MenuItem::Projects,
                    MenuItem::Projects => {
                        task_list_state.select(Some(0));
                        let project_id =
                            &projects.lock().unwrap()[project_list_state.selected().unwrap()].id;
                        let tasks2 = Arc::clone(&tasks);
                        let tasks = tasks2.lock().unwrap();
                        let tasks_from_project: Vec<_> = tasks
                            .iter()
                            .filter(|x| x.project_id == project_id.clone())
                            .collect();

                        if tasks_from_project.len() != 0 {
                            active_menu_item = MenuItem::Tasks;
                        }
                    }
                    MenuItem::Tasks => {
                        active_menu_item = MenuItem::Projects;
                    }
                    _ => {}
                },
                KeyCode::Char('p') => match active_menu_item {
                    MenuItem::Projects => {
                        let projects2 = Arc::clone(&projects);
                        projects2.lock().unwrap().push(Project::name("PogU"));
                        let projects2 = Arc::clone(&projects);
                        tokio::spawn(async move {
                            post_projects("PogU".to_string()).await;
                            *projects2.lock().unwrap() = get_projects().await.unwrap();
                        });
                    }
                    _ => {}
                },
                KeyCode::Char('a') => match active_menu_item {
                    MenuItem::Projects | MenuItem::Tasks => {
                        if let Some(selected) = project_list_state.selected() {
                            //active_menu_item = MenuItem::AddTask;
                            //highlight.name = Color::LightRed;
                            //let mut active_add_task_item = AddTaskItem::Name;
                            let tasks2 = Arc::clone(&tasks);
                            let projects2 = Arc::clone(&projects);
                            let current_selected_project = &projects2.lock().unwrap()[selected].id;
                            tasks.lock().unwrap().push(Task::new(
                                "TestTask".to_string(),
                                current_selected_project.to_string(),
                            ));
                            let map = get_map(current_selected_project.to_string());
                            let tasks4 = Arc::clone(&tasks);
                            tokio::spawn(async move {
                                post_task(map).await;
                                *tasks2.lock().unwrap() = get_tasks().await.unwrap();
                                tasks4
                                    .lock()
                                    .unwrap()
                                    .sort_by(|a, b| a.project_id.cmp(&b.project_id));
                            });
                        }
                    }
                    _ => {}
                },
                KeyCode::Char('d') => match active_menu_item {
                    MenuItem::Tasks => {
                        if let Some(selected) = task_list_state.selected() {
                            if let Some(selected_project) = project_list_state.selected() {
                                let mut task_count = 0;
                                let projects2 = Arc::clone(&projects);
                                let projects2 = projects2.lock().unwrap();
                                let mut tasks_up = vec![];
                                for i in 0..selected_project {
                                    tasks_up.push(projects2[i].id.clone());
                                }
                                let tasks2 = Arc::clone(&tasks);
                                let tasks2 = tasks2.lock().unwrap();
                                tasks2.iter().for_each(|task| {
                                    if tasks_up.iter().any(|s| s.to_string() == task.project_id) {
                                        task_count += 1;
                                    }
                                });
                                let task_at_select =
                                    tasks2[task_count + selected].id.to_owned().clone();
                                let tasks3 = Arc::clone(&tasks);
                                tokio::spawn(async move {
                                    tasks3.lock().unwrap().remove(selected + task_count);
                                    delete_task(task_at_select.to_string()).await;
                                    *tasks3.lock().unwrap() = get_tasks().await.unwrap();
                                    tasks3
                                        .lock()
                                        .unwrap()
                                        .sort_by(|a, b| a.project_id.cmp(&b.project_id));
                                });

                                let mut counter = 0;
                                (0..tasks2.len()).for_each(|i| {
                                    if projects2[selected_project].id == tasks2[i].project_id {
                                        counter += 1;
                                    }
                                });
                                if selected == 0 && counter == 1 {
                                    active_menu_item = MenuItem::Projects;
                                    task_list_state.select(Some(0));
                                } else if selected > 0 {
                                    task_list_state.select(Some(selected - 1));
                                }
                            }
                        }
                    }
                    MenuItem::Projects => {
                        if let Some(selected) = project_list_state.selected() {
                            if selected == 0 {
                                continue;
                            }
                            let id = projects.lock().unwrap()[selected].id.to_owned();
                            tokio::spawn(async move { delete_project(id).await });
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
                                .lock()
                                .unwrap()
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
                                .lock()
                                .unwrap()
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

pub fn get_map(current_selected_project: String) -> Arc<Mutex<HashMap<String, String>>> {
    let mut_map: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let map2 = Arc::clone(&mut_map);
    map2.lock()
        .unwrap()
        .insert("project_id".to_owned(), current_selected_project);
    let map = Arc::clone(&mut_map);
    map.lock()
        .unwrap()
        .insert("content".to_owned(), "TestTask".to_owned());
    return mut_map;
}
