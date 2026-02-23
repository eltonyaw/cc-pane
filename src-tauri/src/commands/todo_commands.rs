use crate::models::todo::*;
use crate::services::TodoService;
use crate::utils::AppResult;
use std::sync::Arc;
use tauri::State;

// ============ TodoItem 命令 (8 个) ============

#[tauri::command]
pub fn create_todo(
    service: State<'_, Arc<TodoService>>,
    request: CreateTodoRequest,
) -> AppResult<TodoItem> {
    Ok(service.create_todo(request)?)
}

#[tauri::command]
pub fn get_todo(
    service: State<'_, Arc<TodoService>>,
    id: String,
) -> AppResult<Option<TodoItem>> {
    Ok(service.get_todo(&id)?)
}

#[tauri::command]
pub fn update_todo(
    service: State<'_, Arc<TodoService>>,
    id: String,
    request: UpdateTodoRequest,
) -> AppResult<TodoItem> {
    Ok(service.update_todo(&id, request)?)
}

#[tauri::command]
pub fn delete_todo(
    service: State<'_, Arc<TodoService>>,
    id: String,
) -> AppResult<()> {
    Ok(service.delete_todo(&id)?)
}

#[tauri::command]
pub fn query_todos(
    service: State<'_, Arc<TodoService>>,
    query: TodoQuery,
) -> AppResult<TodoQueryResult> {
    Ok(service.query_todos(query)?)
}

#[tauri::command]
pub fn reorder_todos(
    service: State<'_, Arc<TodoService>>,
    todo_ids: Vec<String>,
) -> AppResult<()> {
    Ok(service.reorder_todos(todo_ids)?)
}

#[tauri::command]
pub fn batch_update_todo_status(
    service: State<'_, Arc<TodoService>>,
    ids: Vec<String>,
    status: TodoStatus,
) -> AppResult<u32> {
    Ok(service.batch_update_status(ids, status)?)
}

#[tauri::command]
pub fn get_todo_stats(
    service: State<'_, Arc<TodoService>>,
    scope: Option<TodoScope>,
    scope_ref: Option<String>,
) -> AppResult<TodoStats> {
    Ok(service.get_stats(scope, scope_ref)?)
}

// ============ Subtask 命令 (5 个) ============

#[tauri::command]
pub fn add_todo_subtask(
    service: State<'_, Arc<TodoService>>,
    todo_id: String,
    title: String,
) -> AppResult<TodoSubtask> {
    Ok(service.add_subtask(&todo_id, &title)?)
}

#[tauri::command]
pub fn update_todo_subtask(
    service: State<'_, Arc<TodoService>>,
    id: String,
    title: Option<String>,
    completed: Option<bool>,
) -> AppResult<bool> {
    Ok(service.update_subtask(&id, title, completed)?)
}

#[tauri::command]
pub fn delete_todo_subtask(
    service: State<'_, Arc<TodoService>>,
    id: String,
) -> AppResult<()> {
    Ok(service.delete_subtask(&id)?)
}

#[tauri::command]
pub fn toggle_todo_subtask(
    service: State<'_, Arc<TodoService>>,
    id: String,
) -> AppResult<bool> {
    Ok(service.toggle_subtask(&id)?)
}

#[tauri::command]
pub fn reorder_todo_subtasks(
    service: State<'_, Arc<TodoService>>,
    subtask_ids: Vec<String>,
) -> AppResult<()> {
    Ok(service.reorder_subtasks(subtask_ids)?)
}
