use std::sync::{Arc, Mutex};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    Frame,
};

use crate::{
    api::{PostProject, Project, Task, TaskContent},
    config::Config,
    project::{ProjectItem, ProjectStatus},
    task::{AddTaskHighlight, TaskItem, TaskStatus},
    ui::{render_home, render_projects},
};

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

#[derive(Clone, Debug)]
pub struct Database {
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            projects: vec![Project::name("Loading...")],
            tasks: vec![],
        }
    }
}

pub fn render_active_menu_widget<B: Backend>(
    rect: &mut Frame<B>,
    active_menu_item: MenuItem,
    database: Arc<Mutex<Database>>,
    project_status: &mut ProjectStatus,
    task_status: &mut TaskStatus,
    config: &Config,
    left_right_bottom: Vec<Rect>,
    bottom_fullscreen: Vec<Rect>,
    task_chunks_add: Vec<Rect>,
    project_chunks_2: Vec<Rect>,
    project_chunks_add: Vec<Rect>,
) {
    let highlight_color = config.color.clone();

    match active_menu_item {
        MenuItem::Home => rect.render_widget(render_home(), bottom_fullscreen[0]),
        MenuItem::Projects => {
            let (left, right) = render_projects(
                &project_status.project_table_state,
                database,
                highlight_color,
                Color::White,
                config.color.clone(),
            );
            rect.render_stateful_widget(
                left,
                left_right_bottom[0],
                &mut project_status.project_table_state,
            );
            rect.render_widget(right, left_right_bottom[1]);
        }
        MenuItem::Tasks => {
            let (left, right) = render_projects(
                &project_status.project_table_state,
                database,
                Color::White,
                highlight_color,
                config.color.clone(),
            );
            rect.render_stateful_widget(
                left,
                left_right_bottom[0],
                &mut project_status.project_table_state,
            );
            rect.render_stateful_widget(
                right,
                left_right_bottom[1],
                &mut task_status.task_table_state,
            );
        }
        MenuItem::AddTask => {
            let (left, right) = render_projects(
                &project_status.project_table_state,
                database,
                Color::White,
                Color::White,
                config.color.clone(),
            );
            rect.render_stateful_widget(
                left,
                left_right_bottom[0],
                &mut project_status.project_table_state,
            );
            rect.render_widget(right, task_chunks_add[1]);
        }
        MenuItem::AddProject => {
            let (left, right) = render_projects(
                &project_status.project_table_state,
                Arc::clone(&database),
                Color::White,
                Color::White,
                config.color.clone(),
            );
            rect.set_cursor(
                project_chunks_add[0].x + project_status.project_item.name.len() as u16 + 1,
                project_chunks_add[0].y + 1,
            );
            rect.render_widget(left, project_chunks_2[0]);
            rect.render_widget(right, left_right_bottom[1]);
        }
    }
}

pub fn create_chunks(
    size: Rect,
) -> (
    Vec<Rect>,
    Vec<Rect>,
    Vec<Rect>,
    Vec<Rect>,
    Vec<Rect>,
    Vec<Rect>,
) {
    let top_bottom_fullscreen = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
        .split(size);

    let bottom_fullscreen = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(75), Constraint::Length(40)])
        .split(top_bottom_fullscreen[1]);

    let project_add_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(6), Constraint::Min(50)].as_ref())
        .split(size);

    let task_add_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(18), Constraint::Min(50)].as_ref())
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

    let constraints = [
        Constraint::Length(30),
        Constraint::Length(45),
        Constraint::Length(10),
    ];

    let left_right_top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(top_bottom_fullscreen[0]);

    let left_right_bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(top_bottom_fullscreen[1]);

    let project_chunks_2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(project_add_chunk[1]);

    let project_chunks_add = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(project_add_chunk2[1]);

    let task_chunks_add = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(task_add_chunk[1]);

    (
        left_right_top,
        left_right_bottom,
        bottom_fullscreen,
        task_chunks_add,
        project_chunks_2,
        project_chunks_add,
    )
}

pub fn cleanup(
    active_menu_item: &mut MenuItem,
    task_status: &mut TaskStatus,
    project_status: &mut ProjectStatus,
) {
    *active_menu_item = MenuItem::Projects;

    task_status.task_content = TaskContent::default();
    task_status.add_task_highlight = AddTaskHighlight::default();
    task_status.active_task_item = TaskItem::Empty;

    project_status.project_item = PostProject::default();
    project_status.active_project_item = ProjectItem::Empty;
}
