use crate::models::todo::*;
use crate::repository::TodoRepository;
use crate::utils::error_codes as EC;
use std::sync::Arc;

/// Todo 业务逻辑层
pub struct TodoService {
    repo: Arc<TodoRepository>,
}

impl TodoService {
    pub fn new(repo: Arc<TodoRepository>) -> Self {
        Self { repo }
    }

    // ============ TodoItem 操作 ============

    /// 创建 Todo
    pub fn create_todo(&self, req: CreateTodoRequest) -> Result<TodoItem, String> {
        let title = req.title.trim().to_string();
        if title.is_empty() {
            return Err(serde_json::json!({"code": EC::TODO_TITLE_EMPTY, "message": "Title cannot be empty"}).to_string());
        }

        let scope = req.scope.unwrap_or(TodoScope::Global);
        // Validate: workspace/project scope requires scope_ref
        if matches!(scope, TodoScope::Workspace | TodoScope::Project) && req.scope_ref.is_none() {
            return Err(serde_json::json!({"code": EC::TODO_SCOPE_REF_REQUIRED, "message": "workspace/project scope requires scopeRef"}).to_string());
        }

        let now = chrono::Utc::now().to_rfc3339();
        let sort_order = self.repo.max_sort_order()? + 1;

        let todo = TodoItem {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description: req.description,
            status: req.status.unwrap_or(TodoStatus::Todo),
            priority: req.priority.unwrap_or(TodoPriority::Medium),
            scope,
            scope_ref: req.scope_ref,
            tags: req.tags.unwrap_or_default(),
            due_date: req.due_date,
            sort_order,
            created_at: now.clone(),
            updated_at: now,
            subtasks: vec![],
        };

        self.repo.insert(&todo)?;
        Ok(todo)
    }

    /// 获取 Todo
    pub fn get_todo(&self, id: &str) -> Result<Option<TodoItem>, String> {
        self.repo.get(id)
    }

    /// 更新 Todo
    pub fn update_todo(&self, id: &str, req: UpdateTodoRequest) -> Result<TodoItem, String> {
        // Validate title is not empty (if provided)
        if let Some(ref title) = req.title {
            if title.trim().is_empty() {
                return Err(serde_json::json!({"code": EC::TODO_TITLE_EMPTY, "message": "Title cannot be empty"}).to_string());
            }
        }

        let updated = self.repo.update(id, &req)?;
        if !updated {
            return Err(serde_json::json!({"code": EC::TODO_NOT_FOUND, "message": format!("Todo {} not found", id), "params": {"id": id}}).to_string());
        }

        self.repo
            .get(id)?
            .ok_or_else(|| serde_json::json!({"code": EC::TODO_NOT_FOUND, "message": format!("Todo {} not found", id), "params": {"id": id}}).to_string())
    }

    /// 删除 Todo
    pub fn delete_todo(&self, id: &str) -> Result<(), String> {
        let deleted = self.repo.delete(id)?;
        if !deleted {
            return Err(serde_json::json!({"code": EC::TODO_NOT_FOUND, "message": format!("Todo {} not found", id), "params": {"id": id}}).to_string());
        }
        Ok(())
    }

    /// 查询 Todo 列表
    pub fn query_todos(&self, query: TodoQuery) -> Result<TodoQueryResult, String> {
        self.repo.query(&query)
    }

    /// 重排 Todo
    pub fn reorder_todos(&self, todo_ids: Vec<String>) -> Result<(), String> {
        self.repo.reorder(&todo_ids)
    }

    /// 批量更新状态
    pub fn batch_update_status(
        &self,
        ids: Vec<String>,
        status: TodoStatus,
    ) -> Result<u32, String> {
        self.repo.batch_update_status(&ids, &status)
    }

    /// 获取统计
    pub fn get_stats(
        &self,
        scope: Option<TodoScope>,
        scope_ref: Option<String>,
    ) -> Result<TodoStats, String> {
        self.repo
            .stats(scope.as_ref(), scope_ref.as_deref())
    }

    // ============ 子任务操作 ============

    /// 添加子任务
    pub fn add_subtask(&self, todo_id: &str, title: &str) -> Result<TodoSubtask, String> {
        let title = title.trim().to_string();
        if title.is_empty() {
            return Err(serde_json::json!({"code": EC::SUBTASK_TITLE_EMPTY, "message": "Subtask title cannot be empty"}).to_string());
        }

        // Validate parent Todo exists
        if self.repo.get(todo_id)?.is_none() {
            return Err(serde_json::json!({"code": EC::TODO_NOT_FOUND, "message": format!("Todo {} not found", todo_id), "params": {"id": todo_id}}).to_string());
        }

        let sort_order = self.repo.max_subtask_sort_order(todo_id)? + 1;

        let subtask = TodoSubtask {
            id: uuid::Uuid::new_v4().to_string(),
            todo_id: todo_id.to_string(),
            title,
            completed: false,
            sort_order,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.repo.insert_subtask(&subtask)?;
        Ok(subtask)
    }

    /// 更新子任务
    pub fn update_subtask(
        &self,
        id: &str,
        title: Option<String>,
        completed: Option<bool>,
    ) -> Result<bool, String> {
        self.repo
            .update_subtask(id, title.as_deref(), completed)
    }

    /// 删除子任务
    pub fn delete_subtask(&self, id: &str) -> Result<(), String> {
        let deleted = self.repo.delete_subtask(id)?;
        if !deleted {
            return Err(serde_json::json!({"code": EC::SUBTASK_NOT_FOUND, "message": format!("Subtask {} not found", id), "params": {"id": id}}).to_string());
        }
        Ok(())
    }

    /// 切换子任务完成状态
    pub fn toggle_subtask(&self, id: &str) -> Result<bool, String> {
        let subtask = self
            .repo
            .get_subtask(id)?
            .ok_or_else(|| serde_json::json!({"code": EC::SUBTASK_NOT_FOUND, "message": format!("Subtask {} not found", id), "params": {"id": id}}).to_string())?;

        let new_completed = !subtask.completed;
        self.repo
            .update_subtask(id, None, Some(new_completed))?;
        Ok(new_completed)
    }

    /// 重排子任务
    pub fn reorder_subtasks(&self, subtask_ids: Vec<String>) -> Result<(), String> {
        self.repo.reorder_subtasks(&subtask_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Database;

    fn setup() -> TodoService {
        let db = Arc::new(Database::new_in_memory().expect("创建内存数据库失败"));
        let repo = Arc::new(TodoRepository::new(db));
        TodoService::new(repo)
    }

    #[test]
    fn test_create_todo_basic() {
        let service = setup();
        let req = CreateTodoRequest {
            title: "测试任务".to_string(),
            description: None,
            status: None,
            priority: None,
            scope: None,
            scope_ref: None,
            tags: None,
            due_date: None,
        };
        let todo = service.create_todo(req).unwrap();
        assert_eq!(todo.title, "测试任务");
        assert_eq!(todo.status, TodoStatus::Todo);
        assert_eq!(todo.priority, TodoPriority::Medium);
        assert_eq!(todo.scope, TodoScope::Global);
    }

    #[test]
    fn test_create_todo_empty_title_fails() {
        let service = setup();
        let req = CreateTodoRequest {
            title: "  ".to_string(),
            ..Default::default()
        };
        assert!(service.create_todo(req).is_err());
    }

    #[test]
    fn test_create_todo_workspace_without_ref_fails() {
        let service = setup();
        let req = CreateTodoRequest {
            title: "任务".to_string(),
            scope: Some(TodoScope::Workspace),
            scope_ref: None,
            ..Default::default()
        };
        assert!(service.create_todo(req).is_err());
    }

    #[test]
    fn test_update_todo() {
        let service = setup();
        let todo = service
            .create_todo(CreateTodoRequest {
                title: "原标题".to_string(),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .update_todo(
                &todo.id,
                UpdateTodoRequest {
                    title: Some("新标题".to_string()),
                    status: Some(TodoStatus::InProgress),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "新标题");
        assert_eq!(updated.status, TodoStatus::InProgress);
    }

    #[test]
    fn test_delete_todo() {
        let service = setup();
        let todo = service
            .create_todo(CreateTodoRequest {
                title: "将删除".to_string(),
                ..Default::default()
            })
            .unwrap();

        service.delete_todo(&todo.id).unwrap();
        assert!(service.get_todo(&todo.id).unwrap().is_none());
    }

    #[test]
    fn test_query_todos() {
        let service = setup();
        for i in 0..5 {
            service
                .create_todo(CreateTodoRequest {
                    title: format!("任务 {}", i),
                    ..Default::default()
                })
                .unwrap();
        }

        let result = service.query_todos(TodoQuery::default()).unwrap();
        assert_eq!(result.total, 5);
    }

    #[test]
    fn test_subtask_lifecycle() {
        let service = setup();
        let todo = service
            .create_todo(CreateTodoRequest {
                title: "父任务".to_string(),
                ..Default::default()
            })
            .unwrap();

        // 添加
        let sub = service.add_subtask(&todo.id, "子任务1").unwrap();
        assert_eq!(sub.title, "子任务1");
        assert!(!sub.completed);

        // 切换
        let toggled = service.toggle_subtask(&sub.id).unwrap();
        assert!(toggled);

        // 再次切换
        let toggled = service.toggle_subtask(&sub.id).unwrap();
        assert!(!toggled);

        // 删除
        service.delete_subtask(&sub.id).unwrap();
    }

    #[test]
    fn test_add_subtask_to_nonexistent_todo_fails() {
        let service = setup();
        assert!(service.add_subtask("nonexistent", "子任务").is_err());
    }

    #[test]
    fn test_batch_update_status() {
        let service = setup();
        let t1 = service
            .create_todo(CreateTodoRequest {
                title: "A".to_string(),
                ..Default::default()
            })
            .unwrap();
        let t2 = service
            .create_todo(CreateTodoRequest {
                title: "B".to_string(),
                ..Default::default()
            })
            .unwrap();

        let count = service
            .batch_update_status(vec![t1.id.clone(), t2.id.clone()], TodoStatus::Done)
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_stats() {
        let service = setup();
        service
            .create_todo(CreateTodoRequest {
                title: "A".to_string(),
                priority: Some(TodoPriority::High),
                ..Default::default()
            })
            .unwrap();
        service
            .create_todo(CreateTodoRequest {
                title: "B".to_string(),
                priority: Some(TodoPriority::Low),
                ..Default::default()
            })
            .unwrap();

        let stats = service.get_stats(None, None).unwrap();
        assert_eq!(stats.total, 2);
    }

    #[test]
    fn test_sort_order_auto_increment() {
        let service = setup();
        let t1 = service
            .create_todo(CreateTodoRequest {
                title: "First".to_string(),
                ..Default::default()
            })
            .unwrap();
        let t2 = service
            .create_todo(CreateTodoRequest {
                title: "Second".to_string(),
                ..Default::default()
            })
            .unwrap();
        assert!(t2.sort_order > t1.sort_order);
    }
}

