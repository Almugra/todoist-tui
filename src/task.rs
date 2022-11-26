use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    widgets::{Block, TableState},
    Frame,
};

use crate::{
    api::TaskContent,
    handler::{create_basic_block, create_basic_paragraph},
};

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

pub struct TaskStatus {
    pub task_table_state: TableState,
    pub active_task_item: TaskItem,
    pub add_task_highlight: AddTaskHighlight,
    pub task_content: TaskContent,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self {
            task_table_state: TableState::default(),
            active_task_item: TaskItem::Empty,
            add_task_highlight: AddTaskHighlight::default(),
            task_content: TaskContent::default(),
        }
    }
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

pub fn render_active_task_input_widget<B: Backend>(
    rect: &mut Frame<B>,
    task_status: &TaskStatus,
    left_right_bottom: Vec<Rect>,
) {
    match task_status.active_task_item {
        _ if task_status.active_task_item != TaskItem::Empty => {
            let mut x = left_right_bottom[1].x + 1;
            let mut y = left_right_bottom[1].y + 1;
            let task_content = &task_status.task_content;
            match task_status.active_task_item {
                TaskItem::Name => {
                    let name_len = task_content.content.len();
                    let mut next_line_buffer = 0;
                    if name_len >= 40 {
                        next_line_buffer = 3;
                    }
                    x += name_len as u16 - ((name_len / 40) * 40) as u16 + next_line_buffer;
                }
                TaskItem::Desc => {
                    let desc_len = task_content.description.len();
                    let mut next_line_buffer = 0;
                    if desc_len >= 40 {
                        next_line_buffer = 3;
                    }
                    x += desc_len as u16 - ((desc_len / 40) * 40) as u16 + next_line_buffer;
                    y += 3;
                }
                TaskItem::Label => {
                    let label_len = task_content.labels.len();
                    let mut next_line_buffer = 0;
                    if label_len > 40 {
                        next_line_buffer = 3;
                    }
                    x += label_len as u16 - ((label_len / 40) * 40) as u16 + next_line_buffer;
                    y += 6;
                }
                TaskItem::Due => {
                    let due_len = task_content.due_string.len();
                    let mut next_line_buffer = 0;
                    if due_len > 40 {
                        next_line_buffer = 3;
                    }
                    x += due_len as u16 - ((due_len / 40) * 40) as u16 + next_line_buffer;
                    y += 9;
                }
                TaskItem::Prio => {
                    x += task_content.priority.len() as u16;
                    y += 12;
                }
                _ => {}
            }
            rect.set_cursor(x, y);
            render_add_task_input_fields(
                rect,
                left_right_bottom.clone(),
                task_status.add_task_highlight,
                &mut task_status.task_content.clone(),
            );
        }
        _ => {}
    }
}

pub fn create_add_task_blocks<'a>(
    highlight: &AddTaskHighlight,
) -> (Block<'a>, Block<'a>, Block<'a>, Block<'a>, Block<'a>) {
    let name = create_basic_block("Name", highlight.name);
    let desc = create_basic_block("Description", highlight.desc);
    let label = create_basic_block("Labels", highlight.label);
    let prio = create_basic_block("Priority", highlight.prio);
    let due = create_basic_block("Due date", highlight.due);

    (name, desc, prio, label, due)
}

pub fn render_add_task_input_fields<'a, B: Backend>(
    rect: &mut Frame<B>,
    project_chunks: Vec<Rect>,
    highlight: AddTaskHighlight,
    task_content: &mut TaskContent,
) -> () {
    let task_width_full = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(project_chunks[1]);

    let constraints = [Constraint::Length(3); 6];
    let desc_width = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(task_width_full[0]);

    let (name, desc, prio, label, due) = create_add_task_blocks(&highlight);

    let task_name = task_content.content.clone();
    let name_len = task_content.content.len();
    let mut current_name = task_name.clone();
    if name_len >= 40 {
        let (_, second) = task_name.split_at(((name_len / 40) * 40) - 3);
        current_name = second.to_string();
    }
    let name = create_basic_paragraph(current_name, name);

    let task_desc = task_content.description.clone();
    let desc_len = task_content.description.len();
    let mut current_desc = task_desc.clone();
    if desc_len >= 40 {
        let (_, second) = task_desc.split_at(((desc_len / 40) * 40) - 3);
        current_desc = second.to_string();
    }
    let desc = create_basic_paragraph(current_desc, desc);

    let task_label = task_content.labels.clone();
    let label_len = task_content.labels.len();
    let mut current_label = task_label.clone();
    if label_len >= 40 {
        let (_, second) = task_label.split_at(((label_len / 40) * 40) - 3);
        current_label = second.to_string();
    }
    let label = create_basic_paragraph(current_label, label);

    let task_due = task_content.due_string.clone();
    let due_len = task_content.due_string.len();
    let mut current_due = task_due.clone();
    if due_len >= 40 {
        let (_, second) = task_due.split_at(((due_len / 40) * 40) - 3);
        current_due = second.to_string();
    }
    let due = create_basic_paragraph(current_due, due);

    let prio = create_basic_paragraph(task_content.priority.clone(), prio);

    rect.render_widget(name, desc_width[0]);
    rect.render_widget(desc, desc_width[1]);
    rect.render_widget(label, desc_width[2]);
    rect.render_widget(due, desc_width[3]);
    rect.render_widget(prio, desc_width[4]);
}
