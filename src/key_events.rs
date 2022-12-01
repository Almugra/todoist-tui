use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    api::{
        delete_project, delete_task, get_projects, get_tasks, post_projects, post_task,
        PostProject, Project, Task, TaskContent,
    },
    config::Config,
    input::{push_char_to_field, remove_char_from_field},
    menu::{cleanup, Database, MenuItem},
    navigation::{
        change_active_add_task_input_field, navigate_down_projects, navigate_down_tasks,
        navigate_up_projects, navigate_up_tasks,
    },
    project::{ProjectItem, ProjectStatus},
    task::{AddTaskHighlight, TaskItem, TaskStatus},
};

pub enum EventExit {
    Break,
    Continue,
}

pub fn get_key_event(
    event: KeyEvent,
    active_menu_item: &mut MenuItem,
    task_status: &mut TaskStatus,
    project_status: &mut ProjectStatus,
    config: &Config,
    database: Arc<Mutex<Database>>,
) -> EventExit {
    match event.code {
        KeyCode::Esc
            if *active_menu_item == MenuItem::AddTask
                || *active_menu_item == MenuItem::AddProject =>
        {
            cleanup(active_menu_item, task_status, project_status)
        }
        KeyCode::Char(e)
            if *active_menu_item == MenuItem::AddTask
                || *active_menu_item == MenuItem::AddProject =>
        {
            push_char_to_field(e, task_status, project_status)
        }
        KeyCode::Backspace
            if *active_menu_item == MenuItem::AddTask
                || *active_menu_item == MenuItem::AddProject =>
        {
            remove_char_from_field(task_status, project_status)
        }
        KeyCode::Enter if *active_menu_item == MenuItem::AddTask => {
            if let Some(selected) = project_status.project_table_state.selected() {
                let current_selected_project =
                    &database.lock().unwrap().projects[selected].id.clone();
                let temp_task = Task::temp(
                    task_status.task_content.clone(),
                    current_selected_project.to_string(),
                );
                database.lock().unwrap().tasks.push(temp_task.clone());
                let database_mutex = Arc::clone(&database);
                let token2 = config.token.clone();
                tokio::spawn(async move {
                    let _ = post_task(token2.clone(), temp_task).await;
                    database_mutex.lock().unwrap().tasks = get_tasks(token2).await.unwrap();
                    database_mutex
                        .lock()
                        .unwrap()
                        .tasks
                        .sort_by(|a, b| a.project_id.cmp(&b.project_id));
                });
                task_status.task_content = TaskContent::default();
                *active_menu_item = MenuItem::Projects;
                task_status.add_task_highlight = AddTaskHighlight::default();
                task_status.active_task_item = TaskItem::Empty;
            }
        }
        KeyCode::Enter if *active_menu_item == MenuItem::AddProject => {
            let temp_project = Project::name(&project_status.project_item.name);
            database.lock().unwrap().projects.push(temp_project);
            let token = config.token.clone();
            let project_item = project_status.project_item.clone();
            let database2 = Arc::clone(&database);
            tokio::spawn(async move {
                let _ = post_projects(token.clone(), project_item).await;
                database2.lock().unwrap().projects = get_projects(token).await.unwrap();
            });
            project_status.project_item = PostProject::default();
            *active_menu_item = MenuItem::Projects;
            project_status.active_project_item = ProjectItem::Empty;
        }
        KeyCode::Tab if *active_menu_item == MenuItem::AddTask => {
            change_active_add_task_input_field(task_status, config.color);
        }
        KeyCode::BackTab if *active_menu_item == MenuItem::AddTask => {
            change_active_add_task_input_field(task_status, config.color);
            change_active_add_task_input_field(task_status, config.color);
            change_active_add_task_input_field(task_status, config.color);
            change_active_add_task_input_field(task_status, config.color);
        }
        KeyCode::Char('q') => return EventExit::Break,
        KeyCode::Char('h') => match active_menu_item {
            MenuItem::Home => *active_menu_item = MenuItem::Projects,
            MenuItem::Projects => {
                *active_menu_item = MenuItem::Home;
                project_status.project_table_state.select(Some(0));
            }
            MenuItem::Tasks => {
                *active_menu_item = MenuItem::Projects;
            }
            _ => {}
        },
        KeyCode::Char('l') => match active_menu_item {
            MenuItem::Home => *active_menu_item = MenuItem::Projects,
            MenuItem::Projects => {
                if let Some(selected) = project_status.project_table_state.selected() {
                    task_status.task_table_state.select(Some(0));
                    let project_id = &database.lock().unwrap().projects[selected].id.clone();
                    let tasks2 = database.lock().unwrap();
                    let tasks_from_project: Vec<_> = tasks2
                        .tasks
                        .iter()
                        .filter(|x| x.project_id == project_id.clone())
                        .collect();

                    if !tasks_from_project.is_empty() {
                        *active_menu_item = MenuItem::Tasks;
                    }
                }
            }
            MenuItem::Tasks => {
                *active_menu_item = MenuItem::Projects;
            }
            _ => {}
        },
        KeyCode::Char('p') => {
            if let MenuItem::Projects | MenuItem::Tasks = active_menu_item {
                project_status.active_project_item = ProjectItem::Name;
                *active_menu_item = MenuItem::AddProject;
            }
        }
        KeyCode::Char('a') => {
            if let MenuItem::Projects | MenuItem::Tasks = active_menu_item {
                *active_menu_item = MenuItem::AddTask;
                task_status.active_task_item = TaskItem::Name;
                task_status.add_task_highlight.name = config.color;
            }
        }
        KeyCode::Char('d') => match active_menu_item {
            MenuItem::Tasks => {
                if let Some(selected) = task_status.task_table_state.selected() {
                    if let Some(selected_project) = project_status.project_table_state.selected() {
                        let mut task_count = 0;

                        let mut tasks_above_current = vec![];
                        for i in 0..selected_project {
                            let task_id_at_i = database.lock().unwrap().projects[i].id.clone();
                            tasks_above_current.push(task_id_at_i);
                        }
                        database.lock().unwrap().tasks.iter().for_each(|task| {
                            if tasks_above_current.iter().any(|s| *s == task.project_id) {
                                task_count += 1;
                            }
                        });

                        let task_at_select = database.lock().unwrap().tasks[task_count + selected]
                            .id
                            .clone();
                        let token_c = config.token.clone();
                        let database_c = Arc::clone(&database);
                        tokio::spawn(async move {
                            database_c
                                .lock()
                                .unwrap()
                                .tasks
                                .remove(selected + task_count);
                            let _ = delete_task(token_c.clone(), task_at_select.to_string()).await;
                            database_c.lock().unwrap().tasks = get_tasks(token_c).await.unwrap();
                            database_c
                                .lock()
                                .unwrap()
                                .tasks
                                .sort_by(|a, b| a.project_id.cmp(&b.project_id));
                        });

                        let mut task_count = 0;
                        let tasks_len = database.lock().unwrap().tasks.len();
                        (0..tasks_len).for_each(|i| {
                            let selected_project_id = database.lock().unwrap().projects
                                [selected_project]
                                .id
                                .clone();
                            let project_id_from_task =
                                database.lock().unwrap().tasks[i].project_id.clone();
                            if selected_project_id == project_id_from_task {
                                task_count += 1;
                            }
                        });
                        let task_table_state = &mut task_status.task_table_state;
                        if selected == 0 && task_count == 1 {
                            *active_menu_item = MenuItem::Projects;
                            task_table_state.select(Some(0));
                        } else if selected > 0 {
                            task_table_state.select(Some(selected - 1));
                        }
                    }
                }
            }
            MenuItem::Projects => {
                let project_table_state = &mut project_status.project_table_state;
                if let Some(selected) = project_table_state.selected() {
                    if selected == 0 {
                        return EventExit::Continue;
                    }
                    let id = database.lock().unwrap().projects[selected].id.clone();
                    let token = config.token.clone();
                    tokio::spawn(async move { delete_project(token, id).await });
                    database.lock().unwrap().projects.remove(selected);
                    if selected > 0 {
                        project_status
                            .project_table_state
                            .select(Some(selected - 1));
                    } else {
                        project_status.project_table_state.select(Some(0));
                    }
                }
            }
            _ => {}
        },
        KeyCode::Char('j') => match active_menu_item {
            MenuItem::Projects => {
                navigate_down_projects(
                    &mut project_status.project_table_state,
                    Arc::clone(&database).lock().unwrap().projects.len(),
                );
            }
            MenuItem::Tasks => navigate_down_tasks(
                Arc::clone(&database),
                &mut task_status.task_table_state,
                &project_status.project_table_state,
            ),

            _ => {}
        },
        KeyCode::Char('k') => match active_menu_item {
            MenuItem::Projects => {
                navigate_up_projects(
                    &mut project_status.project_table_state,
                    Arc::clone(&database).lock().unwrap().projects.len(),
                );
            }
            MenuItem::Tasks => navigate_up_tasks(
                Arc::clone(&database),
                &mut task_status.task_table_state,
                &project_status.project_table_state,
            ),
            _ => {}
        },
        _ => {}
    }
    EventExit::Continue
}
