use std::sync::Arc;
use std::sync::Mutex;

use tui::widgets::Paragraph;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Cell, Row, Table, TableState},
    Frame,
};

use crate::{
    api::{Task, TaskContent},
    handler::{create_advanced_block, create_basic_block, create_basic_paragraph},
    menu::Database,
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

pub fn add_buffer_if_len(str_len: usize, next_line_buffer: &mut u16, buffer_at: usize) {
    if str_len >= buffer_at {
        *next_line_buffer = 3;
    }
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
                    add_buffer_if_len(name_len, &mut next_line_buffer, 40);
                    x += name_len as u16 - ((name_len / 40) * 40) as u16 + next_line_buffer;
                }
                TaskItem::Desc => {
                    let desc_len = task_content.description.len();
                    let mut next_line_buffer = 0;
                    add_buffer_if_len(desc_len, &mut next_line_buffer, 40);
                    x += desc_len as u16 - ((desc_len / 40) * 40) as u16 + next_line_buffer;
                    y += 3;
                }
                TaskItem::Label => {
                    let label_len = task_content.labels.len();
                    let mut next_line_buffer = 0;
                    add_buffer_if_len(label_len, &mut next_line_buffer, 40);
                    x += label_len as u16 - ((label_len / 40) * 40) as u16 + next_line_buffer;
                    y += 6;
                }
                TaskItem::Due => {
                    let due_len = task_content.due_string.len();
                    let mut next_line_buffer = 0;
                    add_buffer_if_len(due_len, &mut next_line_buffer, 40);
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
                left_right_bottom,
                task_status.add_task_highlight,
                &mut task_status.task_content.clone(),
            );
        }
        _ => {}
    }
}

pub struct Blocks {
    content: Block<'static>,
    description: Block<'static>,
    labels: Block<'static>,
    prio: Block<'static>,
    due: Block<'static>,
}

impl Blocks {
    pub fn create_add_task_blocks(highlight: &AddTaskHighlight) -> Blocks {
        let name = create_basic_block("Name", highlight.name);
        let desc = create_basic_block("Description", highlight.desc);
        let label = create_basic_block("Labels", highlight.label);
        let prio = create_basic_block("Priority", highlight.prio);
        let due = create_basic_block("Due date", highlight.due);

        Blocks {
            content: name,
            description: desc,
            labels: label,
            prio,
            due,
        }
    }
}

pub fn split_if_has_len(str_len: usize, split_at: usize, current: &mut String, task: String) {
    if str_len >= split_at {
        let (_, second) = task.split_at(((str_len / split_at) * split_at) - 3);
        *current = second.to_string();
    }
}

pub fn construct_input_paragraph(task_content: String, blocks_content: Block) -> Paragraph {
    let task_name = task_content.clone();
    let name_len = task_content.len();
    let mut current_name = task_name.clone();
    split_if_has_len(name_len, 40, &mut current_name, task_name);
    create_basic_paragraph(current_name, blocks_content)
}

pub fn render_add_task_input_fields<B: Backend>(
    rect: &mut Frame<B>,
    project_chunks: Vec<Rect>,
    highlight: AddTaskHighlight,
    task_content: &mut TaskContent,
) {
    let task_width_full = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(project_chunks[1]);

    let constraints = [Constraint::Length(3); 6];
    let desc_width = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(task_width_full[0]);

    let blocks: Blocks = Blocks::create_add_task_blocks(&highlight);

    let name = construct_input_paragraph(task_content.content.clone(), blocks.content);

    let desc = construct_input_paragraph(task_content.description.clone(), blocks.description);

    let label = construct_input_paragraph(task_content.labels.clone(), blocks.labels);

    let due = construct_input_paragraph(task_content.due_string.clone(), blocks.due);

    let prio = create_basic_paragraph(task_content.priority.clone(), blocks.prio);

    rect.render_widget(name, desc_width[0]);
    rect.render_widget(desc, desc_width[1]);
    rect.render_widget(label, desc_width[2]);
    rect.render_widget(due, desc_width[3]);
    rect.render_widget(prio, desc_width[4]);
}

pub fn get_task_from_project_id(project_id: String, task_list: &mut Vec<Task>) -> String {
    let mut counter = 0;
    (0..task_list.len()).for_each(|i| {
        if project_id == task_list[i].project_id {
            counter += 1;
        }
    });
    counter.to_string()
}

pub fn get_task_table_list(
    project_table_state: &TableState,
    database: Arc<Mutex<Database>>,
    selection_color: Color,
    highlight_color: Color,
) -> Table<'static> {
    let task_block = create_advanced_block("Tasks", selection_color, Alignment::Left);

    let projects = database.lock().unwrap().projects.clone();
    let tasks = database.lock().unwrap().tasks.clone();

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
                .fg(highlight_color);
            let empty = Cell::from("");

            let mut updated_row = vec![];
            let mut height = 2;

            updated_row.push(Spans::from(Span::styled(task.content.clone(), style)));
            
            let desc_len = task.description.len();
            let mut c_desc = task.description.clone();
            if desc_len > 0 && desc_len < 38 {
                height += 1;
                updated_row.push(Spans::from(task.description.clone()));
            }
            if desc_len != 0 && desc_len >= 38 {
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
                if !c_desc.is_empty() {
                    height += 1;
                    updated_row.push(Spans::from(c_desc));
                }
            }

            if !task.labels.is_empty() {
                height += 1;
                updated_row.push(Spans::from(task.labels.join(", ")));
            }

            match &task.due {
                Some(due) => match &due.datetime {
                    Some(datetime) => {
                        height += 1;
                        if datetime.chars().into_iter().nth(10) == Some('T') {
                            updated_row.push(Spans::from(datetime.replace('T', " ")));
                        } else {
                            updated_row.push(Spans::from(datetime.to_owned()));
                        }
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

    task_list
}
