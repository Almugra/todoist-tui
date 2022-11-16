use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, TableState, Tabs},
    Frame,
};

use crate::{
    api::{PostProject, Project, Task, TaskContent},
    AddTaskHighlight, MenuItem, TaskItem,
};

pub fn render_menu_tabs(active_menu_item: MenuItem) -> Tabs<'static> {
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
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )
        .divider(symbols::DOT);

    menu_tabs
}

pub fn render_key_tabs() -> Tabs<'static> {
    let key_titles = vec!["Add Task", "Post Project", "Delete", "Quit"];
    let keybinds: Vec<_> = key_titles
        .iter()
        .map(|t| {
            let (left, right) = t.split_at(1);
            Spans::from(vec![
                Span::styled(
                    left,
                    Style::default()
                        .fg(Color::LightRed)
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

pub fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("todoist-tui")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

pub fn render_add_tasks<'a>(
    highlight: &AddTaskHighlight,
) -> (
    Block<'a>,
    Block<'a>,
    Block<'a>,
    Block<'a>,
    Block<'a>,
    Block<'a>,
) {
    let outer = Block::default()
        .title("Add Task")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .border_type(BorderType::Plain);

    let name = Block::default()
        .title("Name")
        .borders(Borders::ALL)
        .style(Style::default().fg(highlight.name))
        .border_type(BorderType::Plain);

    let desc = Block::default()
        .title("Description")
        .borders(Borders::ALL)
        .style(Style::default().fg(highlight.desc))
        .border_type(BorderType::Plain);

    let label = Block::default()
        .title("Labels • Separate with comma")
        .borders(Borders::ALL)
        .style(Style::default().fg(highlight.label))
        .border_type(BorderType::Plain);

    let prio = Block::default()
        .title("Priority • Highest = 4 and Lowest = 1")
        .borders(Borders::ALL)
        .style(Style::default().fg(highlight.prio))
        .border_type(BorderType::Plain);

    let due = Block::default()
        .title("Due date • e.g. 'Next week friday at 12:00' ")
        .borders(Borders::ALL)
        .style(Style::default().fg(highlight.due))
        .border_type(BorderType::Plain);

    (outer, name, desc, prio, label, due)
}

pub fn render_projects<'a>(
    project_table_state: &TableState,
    project_list: Vec<Project>,
    task_list: &mut Vec<Task>,
) -> (Table<'a>, Table<'a>) {
    let projects_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Projects")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Plain);

    let task_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .border_type(BorderType::Plain)
        .title("Tasks");

    pub fn get_task_from_project_id(project_id: String, task_list: &mut Vec<Task>) -> String {
        let mut counter = 0;
        (0..task_list.len()).for_each(|i| {
            if project_id == task_list[i].project_id {
                counter += 1;
            }
        });
        counter.to_string()
    }

    let project_items: Vec<_> = project_list
        .iter()
        .map(|project| {
            Row::new(vec![
                project.name.clone(),
                get_task_from_project_id(project.id.clone(), &mut task_list.clone()),
            ])
        })
        .collect();

    let selected_project = project_list
        .get(
            project_table_state
                .selected()
                .expect("there is always a selected project"),
        )
        .expect("exists")
        .clone();

    let task_rows: Vec<_> = task_list
        .iter()
        .filter(|task| task.project_id == selected_project.id)
        .map(|task| {
            Row::new(vec![
                "".to_owned(),
                task.priority.to_string(),
                task.content.to_string(),
                task.description.to_string(),
                task.labels.join(", "),
            ])
        })
        .collect();

    let project_list = Table::new(project_items)
        .block(projects_block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )
        .column_spacing(1)
        .widths(&[Constraint::Length(35), Constraint::Length(5)]);

    let task_list = Table::new(task_rows)
        .block(task_block)
        .header(
            Row::new(vec!["", "Prio", "Name", "Description", "Labels"]).style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::LightRed),
            ),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )
        .column_spacing(1)
        .widths(&[
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(25),
            Constraint::Length(25),
            Constraint::Length(25),
            Constraint::Length(60),
        ]);

    (project_list, task_list)
}

pub fn render_project_item<B: Backend>(
    rect: &mut Frame<B>,
    project_chunks: Vec<Rect>,
    project_item: PostProject,
) {
    let name = Block::default()
        .title("Add Project")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightRed))
        .border_type(BorderType::Plain);

    let name = Paragraph::new(project_item.name.as_ref())
        .style(Style::default().fg(Color::White))
        .block(name);
    rect.render_widget(name, project_chunks[0]);
}

pub fn render_task_item<'a, B: Backend>(
    rect: &mut Frame<B>,
    project_chunks: Vec<Rect>,
    highlight: &AddTaskHighlight,
    task_content: TaskContent,
    task_width_33: Vec<Rect>,
) -> () {
    let task_width_full = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(project_chunks[1]);

    let add_task_chunks_left = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
        ])
        .split(task_width_33[0]);

    let add_task_chunks_mid = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
        ])
        .split(task_width_33[1]);

    let desc_width = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
        ])
        .split(task_width_full[0]);

    let (outer, name, desc, prio, label, due) = render_add_tasks(&highlight);
    let name = Paragraph::new(task_content.content.as_ref())
        .style(Style::default().fg(Color::White))
        .block(name);
    let desc = Paragraph::new(task_content.description.as_ref())
        .style(Style::default().fg(Color::White))
        .block(desc);
    let label = Paragraph::new(task_content.labels.as_ref())
        .style(Style::default().fg(Color::White))
        .block(label);
    let prio = Paragraph::new(task_content.priority.as_ref())
        .style(Style::default().fg(Color::White))
        .block(prio);
    let due = Paragraph::new(task_content.due_string.as_ref())
        .style(Style::default().fg(Color::White))
        .block(due);
    rect.render_widget(outer, project_chunks[1]);
    rect.render_widget(name, desc_width[0]);
    rect.render_widget(desc, desc_width[1]);
    rect.render_widget(label, desc_width[2]);
    rect.render_widget(prio, add_task_chunks_left[3]);
    rect.render_widget(due, add_task_chunks_mid[3]);
}
