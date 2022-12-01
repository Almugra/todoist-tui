use crate::{project::ProjectStatus, task::{TaskStatus, TaskItem}, ProjectItem};

pub fn push_char_to_field(
    e: char,
    task_status: &mut TaskStatus,
    project_status: &mut ProjectStatus,
) {
    let task_content = &mut task_status.task_content;
    match task_status.active_task_item {
        TaskItem::Name => task_content.content.push(e),
        TaskItem::Desc => task_content.description.push(e),
        TaskItem::Label => task_content.labels.push(e),
        TaskItem::Due => task_content.due_string.push(e),
        TaskItem::Prio => {
            task_content.priority.push(e);
            match task_content.priority.parse::<usize>() {
                Ok(x) if (1..=4).contains(&x) => {}
                _ => task_content.priority.clear(),
            };
        }
        _ => {}
    }
    match project_status.active_project_item {
        ProjectItem::Name => project_status.project_item.name.push(e),
        ProjectItem::Empty => {}
    };
}

pub fn remove_char_from_field(task_status: &mut TaskStatus, project_status: &mut ProjectStatus) {
    let task_content = &mut task_status.task_content;
    match task_status.active_task_item {
        TaskItem::Name => task_content.content.pop(),
        TaskItem::Desc => task_content.description.pop(),
        TaskItem::Label => task_content.labels.pop(),
        TaskItem::Due => task_content.due_string.pop(),
        TaskItem::Prio => task_content.priority.pop(),
        _ => None,
    };
    match project_status.active_project_item {
        ProjectItem::Name => project_status.project_item.name.pop(),
        ProjectItem::Empty => None,
    };
}
