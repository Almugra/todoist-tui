use api::{delete_task, get_tasks, post_projects, PostProject, Task, TaskContent};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::thread;
use std::{io, sync::mpsc, vec};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tui::{backend::Backend, widgets::TableState};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::ListItem,
    Terminal,
};
use ui::{render_key_tabs, render_menu_tabs, render_project_item, render_task_item};
pub mod api;
pub mod config;
pub mod ui;
use crate::api::{delete_project, get_projects, post_task, Project};

#[derive(Copy, Clone, Debug)]
pub struct AddTaskHighlight {
    pub name: Color,
    pub desc: Color,
    pub label: Color,
    pub prio: Color,
    pub due: Color,
}

impl Default for AddTaskHighlight {
    fn default() -> Self {
        Self {
            name: Color::White,
            desc: Color::White,
            label: Color::White,
            prio: Color::White,
            due: Color::White,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TaskItem {
    Empty,
    Name,
    Desc,
    Prio,
    Label,
    Due,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProjectItem {
    Empty,
    Name,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
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
    let mut project_list_state = TableState::default();
    project_list_state.select(Some(0));
    let mut task_list_state = TableState::default();
    task_list_state.select(Some(0));

    let projects = Arc::new(Mutex::new(vec![Project::name("Loading...")]));
    let tasks = Arc::new(Mutex::new(vec![]));
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
    let mut active_task_item = TaskItem::Empty;
    let mut highlight = AddTaskHighlight::default();
    let mut task_content = TaskContent::default();

    let mut active_project_item = ProjectItem::Empty;
    let mut project_item = PostProject::default();

    loop {
        terminal.draw(|rect| {
            let size = rect.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(size);

            let project_add_chunk = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(6), Constraint::Min(50)].as_ref())
                .split(size);

            let project_add_chunk2 = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(50),
                    ]
                    .as_ref(),
                )
                .split(size);

            let menu_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[0]);

            let project_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(chunks[1]);

            let project_chunks_2 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(project_add_chunk[1]);

            let project_chunks_add = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(project_add_chunk2[1]);

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
                    let (left, _) = ui::render_projects(
                        &project_list_state,
                        projects.lock().unwrap().clone(),
                        &mut tasks.lock().unwrap().clone(),
                    );
                    rect.render_stateful_widget(left, project_chunks[0], &mut project_list_state);
                }
                MenuItem::AddProject => {
                    let (left, right) = ui::render_projects(
                        &project_list_state,
                        projects.lock().unwrap().clone(),
                        &mut tasks.lock().unwrap().clone(),
                    );
                    rect.set_cursor(
                        project_chunks_add[0].x + project_item.name.len() as u16 + 1,
                        project_chunks_add[0].y + 1,
                    );
                    rect.render_widget(left, project_chunks_2[0]);
                    rect.render_widget(right, project_chunks[1]);
                }
            }

            if active_project_item == ProjectItem::Name {
                render_project_item(rect, project_chunks_add.clone(), project_item.clone());
            }

            match active_task_item {
                _ if active_task_item != TaskItem::Empty => {
                    render_task_item(
                        rect,
                        project_chunks.clone(),
                        &highlight,
                        task_content.clone(),
                    );
                }
                _ => {}
            }
        })?;

        match rx.recv().unwrap() {
            Event::Input(event) => match event.code {
                KeyCode::Esc
                    if active_menu_item == MenuItem::AddTask
                        || active_menu_item == MenuItem::AddProject =>
                {
                    active_menu_item = MenuItem::Projects;

                    task_content = TaskContent::default();
                    highlight = AddTaskHighlight::default();
                    active_task_item = TaskItem::Empty;

                    project_item = PostProject::default();
                    active_project_item = ProjectItem::Empty;
                }
                KeyCode::Char(e)
                    if active_menu_item == MenuItem::AddTask
                        || active_menu_item == MenuItem::AddProject =>
                {
                    match active_task_item {
                        TaskItem::Name => task_content.content.push(e),
                        TaskItem::Desc => task_content.description.push(e),
                        TaskItem::Label => task_content.labels.push(e),
                        TaskItem::Prio => {
                            task_content.priority.push(e);
                            if task_content.priority.len() > 1 {}
                        }
                        TaskItem::Due => task_content.due_string.push(e),
                        _ => {}
                    }
                    match active_project_item {
                        ProjectItem::Name => project_item.name.push(e),
                        ProjectItem::Empty => {}
                    };
                }
                KeyCode::Backspace
                    if active_menu_item == MenuItem::AddTask
                        || active_menu_item == MenuItem::AddProject =>
                {
                    match active_task_item {
                        TaskItem::Name => task_content.content.pop(),
                        TaskItem::Desc => task_content.description.pop(),
                        TaskItem::Label => task_content.labels.pop(),
                        TaskItem::Prio => task_content.priority.pop(),
                        TaskItem::Due => task_content.due_string.pop(),
                        _ => None,
                    };
                    match active_project_item {
                        ProjectItem::Name => project_item.name.pop(),
                        ProjectItem::Empty => None,
                    };
                }
                KeyCode::Enter if active_menu_item == MenuItem::AddTask => {
                    let tasks2 = Arc::clone(&tasks);
                    let projects2 = Arc::clone(&projects);
                    let current_selected_project =
                        &projects2.lock().unwrap()[project_list_state.selected().unwrap()].id;
                    let temp_task =
                        Task::temp(task_content.clone(), current_selected_project.to_string());
                    let temp_task2 =
                        Task::temp(task_content.clone(), current_selected_project.to_string());
                    tasks.lock().unwrap().push(temp_task.clone());
                    let tasks4 = Arc::clone(&tasks);
                    tokio::spawn(async move {
                        let _ = post_task(temp_task2).await;
                        *tasks2.lock().unwrap() = get_tasks().await.unwrap();
                        tasks4
                            .lock()
                            .unwrap()
                            .sort_by(|a, b| a.project_id.cmp(&b.project_id));
                    });
                    task_content = TaskContent::default();
                    highlight = AddTaskHighlight::default();
                    active_menu_item = MenuItem::Projects;
                    active_task_item = TaskItem::Empty;
                }
                KeyCode::Enter if active_menu_item == MenuItem::AddProject => {
                    let projects2 = Arc::clone(&projects);
                    let temp_project = Project::name(&project_item.name);
                    projects2.lock().unwrap().push(temp_project.clone());
                    let projects2 = Arc::clone(&projects);
                    tokio::spawn(async move {
                        let _ = post_projects(project_item).await;
                        *projects2.lock().unwrap() = get_projects().await.unwrap();
                    });
                    project_item = PostProject::default();
                    active_menu_item = MenuItem::Projects;
                    active_project_item = ProjectItem::Empty;
                }
                KeyCode::Tab if active_menu_item == MenuItem::AddTask => match active_task_item {
                    TaskItem::Name => {
                        active_task_item = TaskItem::Desc;
                        highlight = AddTaskHighlight::default();
                        highlight.desc = Color::LightRed;
                    }
                    TaskItem::Desc => {
                        active_task_item = TaskItem::Label;
                        highlight = AddTaskHighlight::default();
                        highlight.label = Color::LightRed;
                    }
                    TaskItem::Label => {
                        active_task_item = TaskItem::Prio;
                        highlight = AddTaskHighlight::default();
                        highlight.prio = Color::LightRed;
                    }
                    TaskItem::Prio => {
                        active_task_item = TaskItem::Due;
                        highlight = AddTaskHighlight::default();
                        highlight.due = Color::LightRed;
                    }
                    TaskItem::Due => {
                        active_task_item = TaskItem::Name;
                        highlight = AddTaskHighlight::default();
                        highlight.name = Color::LightRed;
                    }
                    _ => {}
                },
                KeyCode::BackTab if active_menu_item == MenuItem::AddTask => match active_task_item
                {
                    TaskItem::Name => {
                        active_task_item = TaskItem::Due;
                        highlight = AddTaskHighlight::default();
                        highlight.due = Color::LightRed;
                    }
                    TaskItem::Desc => {
                        active_task_item = TaskItem::Name;
                        highlight = AddTaskHighlight::default();
                        highlight.name = Color::LightRed;
                    }
                    TaskItem::Label => {
                        active_task_item = TaskItem::Desc;
                        highlight = AddTaskHighlight::default();
                        highlight.desc = Color::LightRed;
                    }
                    TaskItem::Prio => {
                        active_task_item = TaskItem::Label;
                        highlight = AddTaskHighlight::default();
                        highlight.label = Color::LightRed;
                    }
                    TaskItem::Due => {
                        active_task_item = TaskItem::Prio;
                        highlight = AddTaskHighlight::default();
                        highlight.prio = Color::LightRed;
                    }
                    _ => {}
                },
                KeyCode::Char('q') => break,
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
                        active_project_item = ProjectItem::Name;
                        active_menu_item = MenuItem::AddProject;
                    }
                    _ => {}
                },
                KeyCode::Char('a') => match active_menu_item {
                    MenuItem::Projects | MenuItem::Tasks => {
                        active_menu_item = MenuItem::AddTask;
                        active_task_item = TaskItem::Name;
                        highlight.name = Color::LightRed;
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
                                    let _ = delete_task(task_at_select.to_string()).await;
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
