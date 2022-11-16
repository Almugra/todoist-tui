use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Error as RError;

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::Config;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub comment_count: usize,
    pub order: usize,
    pub color: String,
    pub is_shared: bool,
    pub is_favorite: bool,
    pub parent_id: Option<String>,
    pub is_inbox_project: bool,
    pub is_team_inbox: bool,
    pub view_style: String,
    pub url: String,
}

impl Project {
    pub fn name(name: &str) -> Project {
        Project {
            id: "".to_owned(),
            name: name.to_owned(),
            comment_count: 0,
            order: 0,
            color: String::new(),
            is_shared: false,
            is_favorite: false,
            parent_id: Some(String::new()),
            is_inbox_project: false,
            is_team_inbox: false,
            view_style: String::new(),
            url: String::new(),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Task {
    pub creator_id: String,
    pub created_at: String,
    pub assignee_id: Option<String>,
    pub assigner_id: Option<String>,
    pub comment_count: usize,
    pub is_completed: bool,
    pub content: String,
    pub description: String,
    pub id: String,
    pub labels: Vec<String>,
    pub order: usize,
    pub priority: usize,
    pub project_id: String,
    pub section_id: Option<String>,
    pub parent_id: Option<String>,
    pub due_string: Option<String>,
    pub url: String,
    pub date: Option<String>,
    pub is_recurring: Option<bool>,
    pub datetime: Option<String>,
    pub timezone: Option<String>,
}

impl Task {
    pub fn temp(task_content: TaskContent, project_id: String) -> Task {
        let labels: Vec<String> = task_content
            .labels
            .replace(" ", "")
            .split(',')
            .map(|s| s.to_owned())
            .collect();

        Task {
            creator_id: String::new(),
            created_at: String::new(),
            assignee_id: None,
            assigner_id: None,
            comment_count: 0,
            is_completed: false,
            content: task_content.content,
            description: task_content.description,
            id: String::new(),
            labels,
            order: 0,
            priority: task_content.priority.parse::<usize>().unwrap(),
            project_id: project_id.to_string(),
            section_id: None,
            parent_id: None,
            due_string: Some(task_content.due_string),
            url: String::new(),
            date: None,
            is_recurring: None,
            datetime: None,
            timezone: None,
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct PostProject {
    pub name: String,
}

impl Default for PostProject {
    fn default() -> Self {
        Self {
            name: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct TaskContent {
    pub content: String,
    pub description: String,
    pub labels: String,
    pub priority: String,
    pub due_string: String,
}

impl Default for TaskContent {
    fn default() -> Self {
        Self {
            content: String::new(),
            description: String::new(),
            labels: String::new(),
            priority: String::from("1"),
            due_string: String::new(),
        }
    }
}

#[allow(dead_code)]
pub async fn get_projects() -> Result<Vec<Project>, RError> {
    let token = Config::get_token();
    let url = format!("Bearer {}", token.token);
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.todoist.com/rest/v2/projects")
        .header(AUTHORIZATION, url)
        .send()
        .await
        .unwrap();
    let projects: Vec<Project> = response.json().await?;

    Ok(projects)
}

#[allow(dead_code)]
pub async fn delete_project(project_id: String) -> Result<(), RError> {
    let token = Config::get_token();
    let url = format!("Bearer {}", token.token);
    let client = reqwest::Client::new();
    let _ = client
        .delete(format!(
            "https://api.todoist.com/rest/v2/projects/{}",
            project_id
        ))
        .header(AUTHORIZATION, url)
        .send()
        .await
        .unwrap();

    Ok(())
}

#[allow(dead_code)]
pub async fn delete_task(task_id: String) -> Result<(), RError> {
    let token = Config::get_token();
    let url = format!("Bearer {}", token.token);
    let client = reqwest::Client::new();
    let _ = client
        .delete(format!("https://api.todoist.com/rest/v2/tasks/{}", task_id))
        .header(AUTHORIZATION, url)
        .send()
        .await
        .unwrap();

    Ok(())
}

#[allow(dead_code)]
pub async fn post_projects(project: PostProject) -> Result<Project, RError> {
    let token = Config::get_token();
    let autherization = format!("Bearer {}", token.token);
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.todoist.com/rest/v2/projects")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, autherization)
        .json(&project)
        .send()
        .await
        .unwrap();
    let project: Project = response.json().await?;

    Ok(project)
}

#[allow(dead_code)]
pub async fn get_tasks() -> Result<Vec<Task>, RError> {
    let token = Config::get_token();
    let url = format!("Bearer {}", token.token);
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.todoist.com/rest/v2/tasks")
        .header(AUTHORIZATION, url)
        .send()
        .await
        .unwrap();
    let tasks: Vec<Task> = response.json().await?;

    Ok(tasks)
}

#[allow(dead_code)]
pub async fn post_task(task: Task) -> Result<Task, RError> {
    let token = Config::get_token();
    let autherization = format!("Bearer {}", token.token);
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.todoist.com/rest/v2/tasks")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, autherization)
        .json(&task)
        .send()
        .await
        .unwrap();
    let tasks: Task = response.json().await?;

    Ok(tasks)
}
