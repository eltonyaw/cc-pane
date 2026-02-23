use crate::services::skill_service::{SkillInfo, SkillSummary};
use crate::services::SkillService;
use crate::utils::{validate_path, AppResult};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn list_skills(
    project_path: String,
    service: State<'_, Arc<SkillService>>,
) -> AppResult<Vec<SkillSummary>> {
    validate_path(&project_path)?;
    Ok(service.list_skills(&project_path)?)
}

#[tauri::command]
pub fn get_skill(
    project_path: String,
    name: String,
    service: State<'_, Arc<SkillService>>,
) -> AppResult<Option<SkillInfo>> {
    validate_path(&project_path)?;
    Ok(service.get_skill(&project_path, &name)?)
}

#[tauri::command]
pub fn save_skill(
    project_path: String,
    name: String,
    content: String,
    service: State<'_, Arc<SkillService>>,
) -> AppResult<SkillInfo> {
    validate_path(&project_path)?;
    Ok(service.save_skill(&project_path, &name, &content)?)
}

#[tauri::command]
pub fn delete_skill(
    project_path: String,
    name: String,
    service: State<'_, Arc<SkillService>>,
) -> AppResult<bool> {
    validate_path(&project_path)?;
    Ok(service.delete_skill(&project_path, &name)?)
}

#[tauri::command]
pub fn copy_skill(
    source_project: String,
    target_project: String,
    name: String,
    service: State<'_, Arc<SkillService>>,
) -> AppResult<SkillInfo> {
    validate_path(&source_project)?;
    validate_path(&target_project)?;
    Ok(service.copy_skill(&source_project, &target_project, &name)?)
}
