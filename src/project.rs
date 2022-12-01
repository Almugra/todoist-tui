use std::sync::{Arc, Mutex};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::{
    api::PostProject,
    handler::{create_advanced_block, create_basic_block, create_basic_paragraph},
    menu::Database,
};

use super::task::get_task_from_project_id;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProjectItem {
    Empty,
    Name,
}

pub struct ProjectStatus {
    pub project_table_state: TableState,
    pub active_project_item: ProjectItem,
    pub project_item: PostProject,
}

impl Default for ProjectStatus {
    fn default() -> Self {
        Self {
            project_table_state: Default::default(),
            active_project_item: ProjectItem::Empty,
            project_item: Default::default(),
        }
    }
}

pub fn get_project_table_list(
    database: &Arc<Mutex<Database>>,
    selection_color: Color,
    highlight_color: Color,
) -> Table {
    let projects_block = create_advanced_block("Projects", selection_color, Alignment::Center);

    let tasks = database.lock().unwrap().tasks.clone();
    let projects = database.lock().unwrap().projects.clone();

    let project_items: Vec<_> = projects
        .iter()
        .map(|project| {
            Row::new(vec![
                project.name.clone(),
                get_task_from_project_id(project.id.clone(), &mut tasks.clone()),
            ])
        })
        .collect();

    let project_table_list = Table::new(project_items)
        .block(projects_block)
        .highlight_style(
            Style::default()
                .fg(highlight_color)
                .add_modifier(Modifier::BOLD),
        )
        .column_spacing(1)
        .highlight_symbol(">")
        .widths(&[Constraint::Percentage(89), Constraint::Percentage(5)]);

    project_table_list
}

pub fn render_project_item<B: Backend>(
    rect: &mut Frame<B>,
    project_chunks: Vec<Rect>,
    project_item: &PostProject,
    config_color: Color,
) {
    let name = create_basic_block("Add Project", config_color);

    let task_name = project_item.name.clone();
    let name_len = project_item.name.len();
    let mut current_name = task_name.clone();
    if name_len >= 25 {
        let (_, second) = task_name.split_at(((name_len / 25) * 25) - 3);
        current_name = second.to_string();
    }
    let name = create_basic_paragraph(current_name, name);

    rect.render_widget(name, project_chunks[0]);
}
