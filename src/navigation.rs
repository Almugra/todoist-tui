use std::sync::{Arc, Mutex};

use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{ListItem, TableState},
};

use crate::{menu::Database, task::TaskStatus, AddTaskHighlight, TaskItem};

pub fn navigate_down_projects(project_list_state: &mut TableState, project_amount: usize) {
    if let Some(selected) = project_list_state.selected() {
        if selected >= project_amount - 1 {
            project_list_state.select(Some(0));
        } else {
            project_list_state.select(Some(selected + 1));
        }
    }
}

pub fn navigate_up_projects(project_list_state: &mut TableState, project_amount: usize) {
    if let Some(selected) = project_list_state.selected() {
        if selected > 0 {
            project_list_state.select(Some(selected - 1));
        } else {
            project_list_state.select(Some(project_amount - 1));
        }
    }
}

pub fn navigate_down_tasks(
    database: Arc<Mutex<Database>>,
    task_list_state: &mut TableState,
    project_list_state: &TableState,
) {
    if let Some(selected) = task_list_state.selected() {
        let selected_project = database
            .lock()
            .unwrap()
            .projects
            .clone()
            .get(
                project_list_state
                    .selected()
                    .expect("there is always a selected project"),
            )
            .expect("exists")
            .clone();

        let task_items: Vec<_> = database
            .lock()
            .unwrap()
            .tasks
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

pub fn navigate_up_tasks(
    database: Arc<Mutex<Database>>,
    task_list_state: &mut TableState,
    project_list_state: &TableState,
) {
    if let Some(selected) = task_list_state.selected() {
        if let Some(selected_p) = project_list_state.selected() {
            let selected_project = database
                .lock()
                .unwrap()
                .projects
                .clone()
                .get(selected_p)
                .unwrap()
                .clone();

            let task_items: Vec<ListItem> = database
                .lock()
                .unwrap()
                .tasks
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
}

pub fn change_active_add_task_input_field(task_status: &mut TaskStatus, config_color: Color) {
    match task_status.active_task_item {
        TaskItem::Name => {
            task_status.active_task_item = TaskItem::Desc;
            task_status.add_task_highlight = AddTaskHighlight::default();
            task_status.add_task_highlight.desc = config_color.clone();
        }
        TaskItem::Desc => {
            task_status.active_task_item = TaskItem::Label;
            task_status.add_task_highlight = AddTaskHighlight::default();
            task_status.add_task_highlight.label = config_color.clone();
        }
        TaskItem::Label => {
            task_status.active_task_item = TaskItem::Due;
            task_status.add_task_highlight = AddTaskHighlight::default();
            task_status.add_task_highlight.due = config_color.clone();
        }
        TaskItem::Due => {
            task_status.active_task_item = TaskItem::Prio;
            task_status.add_task_highlight = AddTaskHighlight::default();
            task_status.add_task_highlight.prio = config_color.clone();
        }
        TaskItem::Prio => {
            task_status.active_task_item = TaskItem::Name;
            task_status.add_task_highlight = AddTaskHighlight::default();
            task_status.add_task_highlight.name = config_color.clone();
        }
        _ => {}
    }
}
