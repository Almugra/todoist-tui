use tui::{backend::Backend, widgets::TableState, Frame};

use crate::{api::PostProject, config::Config, ui::render_project_item};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProjectItem {
    Empty,
    Name,
}

pub struct ProjectStatus {
    pub project_table_state: TableState,
    pub active_project_item: ProjectItem,
    pub project_item: PostProject,
}

impl Default for ProjectStatus {
    fn default() -> Self {
        Self {
            project_table_state: Default::default(),
            active_project_item: ProjectItem::Empty,
            project_item: Default::default(),
        }
    }
}

pub fn render_active_project_input_widget<B: Backend>(
    rect: &mut Frame<B>,
    project_status: &ProjectStatus,
    project_chunks_add: Vec<tui::layout::Rect>,
    config: &Config,
) {
    if project_status.active_project_item == ProjectItem::Name {
        render_project_item(
            rect,
            project_chunks_add.clone(),
            &project_status.project_item,
            config.color.clone(),
        );
    }
}
