use tui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, ListItem, Paragraph, Row, Table, TableState, Tabs},
};

use crate::{
    api::{Project, Task},
    MenuItem,
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
        .block(Block::default().title("Menu").borders(Borders::ALL))
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
    let key_titles = vec!["Add", "Delete", "Quit"];
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

pub fn render_projects<'a>(
    project_table_state: &TableState,
    project_list: Vec<Project>,
    task_list: &mut Vec<Task>,
) -> (Table<'a>, Table<'a>) {
    let projects_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Projects")
        .border_type(BorderType::Plain);

    let task_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .border_type(BorderType::Plain);

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
                task.priority.to_string(),
                task.content.to_string(),
                task.description.to_string(),
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
            Row::new(vec!["Priority", "Name", "Description"]).style(
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
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(40),
        ]);

    (project_list, task_list)
}
