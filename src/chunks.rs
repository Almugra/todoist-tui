use tui::layout::{Rect, Layout, Direction, Constraint};

pub struct Chunks {
        pub menu_or_keybinds: Vec<Rect>,
        pub projects_or_tasks: Vec<Rect>,
        pub bottom_fullscreen: Vec<Rect>,
        pub tasks_with_add_task: Vec<Rect>,
        pub project_with_add_project: Vec<Rect>,
        pub add_project_with_projects: Vec<Rect>,
}

impl Chunks {
    pub fn create_chunks(size: Rect) -> Chunks {
    let top_bottom_split = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
        .split(size);

    let bottom_fullscreen = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(75), Constraint::Length(40)])
        .split(top_bottom_split[1]);

    let projects_section_with_add_project_widget = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(6), Constraint::Min(50)].as_ref())
        .split(size);

    let task_selection_with_add_task_widget = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(18), Constraint::Min(50)].as_ref())
        .split(size);

    let add_project_section_with_projects_widget = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(50),
            ]
            .as_ref(),
        )
        .split(size);

    let constraints = [
        Constraint::Length(30),
        Constraint::Length(45),
        Constraint::Length(10),
    ];

    let menu_or_keybinds = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(top_bottom_split[0]);

    let projects_or_tasks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(top_bottom_split[1]);

    let project_with_add_project = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(projects_section_with_add_project_widget[1]);

    let add_project_with_projects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(add_project_section_with_projects_widget[1]);

    let tasks_with_add_task = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(task_selection_with_add_task_widget[1]);

    Chunks {
        menu_or_keybinds,
        projects_or_tasks,
        bottom_fullscreen,
        tasks_with_add_task,
        project_with_add_project,
        add_project_with_projects,
    }
    }
}
