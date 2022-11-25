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
                let database1 = Arc::clone(&database);
                let current_selected_project =
                    &database1.lock().unwrap().projects[selected].id.clone();
                let temp_task = Task::temp(
                    task_status.task_content.clone(),
                    current_selected_project.to_string(),
                );
                database1.lock().unwrap().tasks.push(temp_task.clone());
                let database2 = Arc::clone(&database);
                let token2 = config.token.clone();
                tokio::spawn(async move {
                    let _ = post_task(token2.clone(), temp_task).await;
                    database2.lock().unwrap().tasks = get_tasks(token2).await.unwrap();
                    database2
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
            let database2 = Arc::clone(&database);
            database2
                .lock()
                .unwrap()
                .projects
                .push(temp_project.clone());
            let token = config.token.clone();
            let token2 = config.token.clone();
            let project_item = project_status.project_item.clone();
            tokio::spawn(async move {
                let _ = post_projects(token, project_item).await;
                database2.lock().unwrap().projects = get_projects(token2).await.unwrap();
            });
            project_status.project_item = PostProject::default();
            *active_menu_item = MenuItem::Projects;
            project_status.active_project_item = ProjectItem::Empty;
        }
        KeyCode::Tab if *active_menu_item == MenuItem::AddTask => {
            change_active_add_task_input_field(task_status, config.color.clone());
        }
        KeyCode::BackTab if *active_menu_item == MenuItem::AddTask => {
            change_active_add_task_input_field(task_status, config.color.clone());
            change_active_add_task_input_field(task_status, config.color.clone());
            change_active_add_task_input_field(task_status, config.color.clone());
            change_active_add_task_input_field(task_status, config.color.clone());
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
                task_status.task_table_state.select(Some(0));
                let database2 = Arc::clone(&database);
                let project_id = &database2.lock().unwrap().projects
                    [project_status.project_table_state.selected().unwrap()]
                .id
                .clone();
                let tasks2 = database2.lock().unwrap();
                let tasks_from_project: Vec<_> = tasks2
                    .tasks
                    .iter()
                    .filter(|x| x.project_id == project_id.clone())
                    .collect();

                if tasks_from_project.len() != 0 {
                    *active_menu_item = MenuItem::Tasks;
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
                task_status.add_task_highlight.name = config.color.clone();
            }
        }
        KeyCode::Char('d') => match active_menu_item {
            MenuItem::Tasks => {
                if let Some(selected) = task_status.task_table_state.selected() {
                    if let Some(selected_project) = project_status.project_table_state.selected() {
                        let mut task_count = 0;

                        let database2 = Arc::clone(&database);
                        let mut tasks_above_current = vec![];
                        for i in 0..selected_project {
                            let task_id_at_i = database2.lock().unwrap().projects[i].id.clone();
                            tasks_above_current.push(task_id_at_i);
                        }
                        database2.lock().unwrap().tasks.iter().for_each(|task| {
                            if tasks_above_current
                                .iter()
                                .any(|s| s.to_string() == task.project_id)
                            {
                                task_count += 1;
                            }
                        });

                        let task_at_select = database2.lock().unwrap().tasks[task_count + selected]
                            .id
                            .clone();
                        let token2 = config.token.clone();
                        let database3 = Arc::clone(&database);
                        tokio::spawn(async move {
                            database2
                                .lock()
                                .unwrap()
                                .tasks
                                .remove(selected + task_count);
                            let _ = delete_task(token2.clone(), task_at_select.to_string()).await;
                            database3.lock().unwrap().tasks = get_tasks(token2).await.unwrap();
                            database3
                                .lock()
                                .unwrap()
                                .tasks
                                .sort_by(|a, b| a.project_id.cmp(&b.project_id));
                        });

                        let mut task_count = 0;
                        let database2 = Arc::clone(&database);
                        let tasks_len = database2.lock().unwrap().tasks.len().clone();
                        (0..tasks_len).for_each(|i| {
                            let selected_project_id = database2.lock().unwrap().projects
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
                    let database2 = Arc::clone(&database);
                    let id = database2.lock().unwrap().projects[selected].id.clone();
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
