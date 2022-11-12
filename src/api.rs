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
pub struct Due {
    pub date: String,
    pub is_recurring: bool,
    pub datetime: String,
    pub string: String,
    pub timezone: String,
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
    pub due: Option<Due>,
    pub id: String,
    pub labels: Vec<String>,
    pub order: usize,
    pub priority: usize,
    pub project_id: String,
    pub section_id: Option<String>,
    pub parent_id: Option<String>,
    pub url: String,
}

impl Task {
    pub fn new(content: String, project_id: String) -> Task {
        Task {
            creator_id: "asd".to_owned(),
            created_at: "asda".to_owned(),
            assignee_id: None,
            assigner_id: None,
            comment_count: 0,
            is_completed: false,
            content,
            description: "asda".to_owned(),
            due: None,
            id: "asda".to_owned(),
            labels: vec![],
            order: 0,
            priority: 0,
            project_id,
            section_id: None,
            parent_id: None,
            url: "asda".to_owned(),
        }
    }
}

#[allow(dead_code)]
pub async fn get_projects(token: &str) -> Result<Vec<Project>, RError> {
    let token = token;
    let url = format!("Bearer {}", token);
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
pub async fn delete_project(token: &str, project_id: String) -> Result<(), RError> {
    let token = token;
    let url = format!("Bearer {}", token);
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
pub async fn delete_task(token: &str, task_id: String) -> Result<(), RError> {
    let token = token;
    let url = format!("Bearer {}", token);
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
pub async fn post_projects(token: &str, name: &str) -> Result<Project, RError> {
    let mut map = HashMap::new();
    map.insert("name", name);
    let token = token;
    let autherization = format!("Bearer {}", token);
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.todoist.com/rest/v2/projects")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, autherization)
        .json(&map)
        .send()
        .await
        .unwrap();
    let project: Project = response.json().await?;

    Ok(project)
}

#[allow(dead_code)]
pub async fn get_tasks(token: &str) -> Result<Vec<Task>, RError> {
    let token = token;
    let url = format!("Bearer {}", token);
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
pub async fn post_task<'a>(
    content: &'a str,
    map: &mut HashMap<&str, &'a str>,
) -> Result<Task, RError> {
    map.insert("content", content);
    let token = Config::get_token();
    let autherization = format!("Bearer {}", token.token);
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.todoist.com/rest/v2/tasks")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, autherization)
        .json(&map)
        .send()
        .await
        .unwrap();
    let tasks: Task = response.json().await?;

    Ok(tasks)
}
