use tui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Tabs},
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
                        .fg(Color::Red)
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
    pet_list_state: &ListState,
    project_list: Vec<Project>,
    task_list: Vec<Task>,
) -> (List<'a>, List<'a>) {
    let projects_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Projects")
        .border_type(BorderType::Plain);

    let task_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Tasks")
        .border_type(BorderType::Plain);

    let project_items: Vec<_> = project_list
        .iter()
        .map(|project| {
            ListItem::new(Spans::from(vec![Span::styled(
                project.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_project = project_list
        .get(
            pet_list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .expect("exists")
        .clone();

    let task_items: Vec<_> = task_list
        .iter()
        .filter(|task| task.project_id == selected_project.id)
        .map(|task| {
            ListItem::new(Spans::from(vec![Span::styled(
                task.content.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let project_list = List::new(project_items)
        .block(projects_block)
        .highlight_style(
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        );

    let task_list = List::new(task_items).block(task_block).highlight_style(
        Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::BOLD),
    );

    (project_list, task_list)
}
