use tui::{style::Color, widgets::TableState, layout::Rect, Frame, backend::Backend};

use crate::{api::TaskContent, ui::render_task_item};

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
                    x += task_content.content.len() as u16;
                }
                TaskItem::Desc => {
                    x += task_content.description.len() as u16;
                    y += 3;
                }
                TaskItem::Label => {
                    x += task_content.labels.len() as u16;
                    y += 6;
                }
                TaskItem::Due => {
                    x += task_content.due_string.len() as u16;
                    y += 9;
                }
                TaskItem::Prio => {
                    x += task_content.priority.len() as u16;
                    y += 12;
                }
                _ => {}
            }
            rect.set_cursor(x, y);
            render_task_item(
                rect,
                left_right_bottom.clone(),
                task_status.add_task_highlight,
                task_status.task_content.clone(),
            );
        }
        _ => {}
    }
}
