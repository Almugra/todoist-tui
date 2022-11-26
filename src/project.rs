use std::sync::{Arc, Mutex};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Cell, Row, Table, TableState},
    Frame,
};

use crate::{
    api::{PostProject, Task},
    handler::{create_advanced_block, create_basic_block, create_basic_paragraph},
    menu::Database,
};

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

pub fn render_projects<'a>(
    project_table_state: &TableState,
    database: Arc<Mutex<Database>>,
    color_left: Color,
    color_right: Color,
    config_color: Color,
) -> (Table<'a>, Table<'a>) {
    let projects_block = create_advanced_block("Projects", color_left, Alignment::Center);
    let task_block = create_advanced_block("Tasks", color_right, Alignment::Left);

    pub fn get_task_from_project_id(project_id: String, task_list: &mut Vec<Task>) -> String {
        let mut counter = 0;
        (0..task_list.len()).for_each(|i| {
            if project_id == task_list[i].project_id {
                counter += 1;
            }
        });
        counter.to_string()
    }

    let projects = database.lock().unwrap().projects.clone();
    let tasks = database.lock().unwrap().tasks.clone();
    let project_items: Vec<_> = projects
        .iter()
        .map(|project| {
            Row::new(vec![
                project.name.clone(),
                get_task_from_project_id(project.id.clone(), &mut tasks.clone()),
            ])
        })
        .collect();

    let selected_project = projects
        .get(
            project_table_state
                .selected()
                .expect("there is always a selected project"),
        )
        .expect("exists")
        .clone();

    let task_rows: Vec<_> = tasks
        .iter()
        .filter(|task| task.project_id == selected_project.id)
        .map(|task| {
            let style = Style::default()
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                .fg(config_color);
            let empty = Cell::from("");

            let mut updated_row = vec![];
            let mut height = 2;

            updated_row.push(Spans::from(Span::styled(task.content.clone(), style)));

            let desc_len = task.description.len();
            let mut c_desc = task.description.clone();
            if desc_len != 0 {
                if desc_len >= 38 {
                    for _ in 1..=(desc_len / 38) {
                        let end_split = c_desc.split_off(38);
                        if end_split.len() <= 3 {
                            c_desc.push_str(&end_split);
                            updated_row.push(Spans::from(c_desc.clone()));
                            c_desc.clear();
                        } else {
                            updated_row.push(Spans::from(c_desc.clone()));
                            c_desc = end_split;
                        }
                        height += 1;
                    }
                    if c_desc.len() > 0 {
                        height += 1;
                        updated_row.push(Spans::from(c_desc));
                    }
                } else {
                    height += 1;
                    updated_row.push(Spans::from(task.description.clone()));
                }
            }
            if task.labels.len() != 0 {
                height += 1;
                updated_row.push(Spans::from(task.labels.join(", ")));
            }

            match &task.due {
                Some(due) => match &due.datetime {
                    Some(datetime) => {
                        height += 1;
                        updated_row.push(Spans::from(datetime.replace("T", " ")));
                    }
                    None => {}
                },
                None => {}
            };

            Row::new(vec![empty, Cell::from(updated_row)]).height(height)
        })
        .collect();

    let task_list = Table::new(task_rows)
        .block(task_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .column_spacing(1)
        .highlight_symbol(">")
        .widths(&[Constraint::Max(2), Constraint::Percentage(100)]);

    let project_list = Table::new(project_items)
        .block(projects_block)
        .highlight_style(
            Style::default()
                .fg(config_color)
                .add_modifier(Modifier::BOLD),
        )
        .column_spacing(1)
        .highlight_symbol(">")
        .widths(&[Constraint::Percentage(89), Constraint::Percentage(5)]);

    (project_list, task_list)
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
