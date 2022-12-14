use std::{sync::{Arc, Mutex}, io};

use tui::{
    backend::Backend,
    layout::Alignment,
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::{
    api::{PostProject, Project, Task, TaskContent},
    chunks::Chunks,
    config::Config,
    home::render_home,
    project::{get_project_table_list, ProjectItem, ProjectStatus},
    task::{get_task_table_list, AddTaskHighlight, TaskItem, TaskStatus},
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

pub fn render_active_menu_widget<B: Backend> (
    rect: &mut Frame<B>,
    active_menu_item: MenuItem,
    database: Arc<Mutex<Database>>,
    project_status: &mut ProjectStatus,
    task_status: &mut TaskStatus,
    config: &Config,
    chunks: &Chunks,
) {
    let highlight_color = config.color;

    match active_menu_item {
        MenuItem::Home => rect.render_widget(render_home(), chunks.bottom_fullscreen[0]),
        MenuItem::Projects => {
            let project_table = get_project_table_list(
                &database,
                highlight_color,
                highlight_color,
            );

            let task_table = get_task_table_list(
                &project_status.project_table_state,
                Arc::clone(&database),
                Color::White,
                highlight_color,
            );
            rect.render_stateful_widget(
                project_table,
                chunks.projects_or_tasks[0],
                &mut project_status.project_table_state,
            );
            rect.render_widget(task_table, chunks.projects_or_tasks[1]);
        }
        MenuItem::Tasks => {
            let project_table = get_project_table_list(
                &database,
                Color::White,
                config.color,
            );
            let task_table = get_task_table_list(
                &project_status.project_table_state,
                Arc::clone(&database),
                Color::White,
                highlight_color,
            );
            rect.render_stateful_widget(
                project_table,
                chunks.projects_or_tasks[0],
                &mut project_status.project_table_state,
            );
            rect.render_stateful_widget(
                task_table,
                chunks.projects_or_tasks[1],
                &mut task_status.task_table_state,
            );
        }
        MenuItem::AddTask => {
            let project_table = get_project_table_list(
                &database,
                Color::White,
                config.color,
            );
            let task_table = get_task_table_list(
                &project_status.project_table_state,
                Arc::clone(&database),
                Color::White,
                highlight_color,
            );
            rect.render_stateful_widget(
                project_table,
                chunks.projects_or_tasks[0],
                &mut project_status.project_table_state,
            );
            rect.render_widget(task_table, chunks.tasks_with_add_task[1]);
        }
        MenuItem::AddProject => {
            let project_table = get_project_table_list(
                &database,
                Color::White,
                config.color,
            );
            let task_table = get_task_table_list(
                &project_status.project_table_state,
                Arc::clone(&database),
                Color::White,
                highlight_color,
            );
            let name_len = project_status.project_item.name.len();
            let mut next_line_buffer = 0;
            if name_len >= 25 {
                next_line_buffer = 3;
            }
            rect.set_cursor(
                chunks.add_project_with_projects[0].x + 1 + name_len as u16
                    - ((name_len / 25) * 25) as u16
                    + next_line_buffer,
                chunks.add_project_with_projects[0].y + 1,
            );
            rect.render_widget(project_table, chunks.project_with_add_project[0]);
            rect.render_widget(task_table, chunks.projects_or_tasks[1]);
        }
    }
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

pub fn render_menu_tabs(active_menu_item: MenuItem, config_color: Color) -> Tabs<'static> {
    let menu_titles = vec!["Home", "Projects", "Tasks"];

    let menu: Vec<_> = menu_titles
        .iter()
        .map(|t| {
            Spans::from(Span::styled(
                t.to_owned(),
                Style::default().fg(Color::White),
            ))
        })
        .collect();

    let menu_tabs = Tabs::new(menu)
        .select(active_menu_item.into())
        .block(
            Block::default()
                .title("Menu")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(config_color)
                .add_modifier(Modifier::BOLD),
        )
        .divider(symbols::DOT);

    menu_tabs
}

pub fn render_key_tabs(config_color: Color) -> Tabs<'static> {
    let key_titles = vec!["Add Task", "Post Project", "Delete", "Quit"];
    let keybinds: Vec<_> = key_titles
        .iter()
        .map(|t| {
            let (left, right) = t.split_at(1);
            Spans::from(vec![
                Span::styled(
                    left,
                    Style::default()
                        .fg(config_color)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ),
                Span::styled(right, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let key_tabs = Tabs::new(keybinds)
        .block(Block::default().title("Keybinds").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .divider(symbols::DOT);

    key_tabs
}
