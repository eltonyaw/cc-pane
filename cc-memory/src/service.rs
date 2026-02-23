use crate::models::*;
use crate::repository::MemoryRepository;

pub struct MemoryService {
    repo: MemoryRepository,
}

impl MemoryService {
    pub fn new(repo: MemoryRepository) -> Self {
        Self { repo }
    }

    /// 存储新 Memory
    pub fn store(&self, req: StoreMemoryRequest) -> Result<Memory, String> {
        // 验证
        if req.title.trim().is_empty() {
            return Err("标题不能为空".to_string());
        }
        if req.title.len() > 200 {
            return Err("标题不能超过 200 字符".to_string());
        }
        if req.content.trim().is_empty() {
            return Err("内容不能为空".to_string());
        }

        let scope = req.scope.unwrap_or(MemoryScope::Project);
        let importance = req.importance.unwrap_or(3).clamp(1, 5);

        // scope 约束检查
        match scope {
            MemoryScope::Workspace => {
                if req.workspace_name.is_none() {
                    return Err("Workspace 作用域需要 workspace_name".to_string());
                }
            }
            MemoryScope::Project => {
                if req.project_path.is_none() {
                    return Err("Project 作用域需要 project_path".to_string());
                }
            }
            MemoryScope::Session => {
                if req.session_id.is_none() {
                    return Err("Session 作用域需要 session_id".to_string());
                }
            }
            MemoryScope::Global => {}
        }

        let now = chrono::Utc::now().to_rfc3339();
        let memory = Memory {
            id: uuid::Uuid::new_v4().to_string(),
            title: req.title.trim().to_string(),
            content: req.content,
            scope,
            category: req.category.unwrap_or(MemoryCategory::Fact),
            importance,
            workspace_name: req.workspace_name,
            project_path: req.project_path,
            session_id: req.session_id,
            tags: req.tags.unwrap_or_default(),
            source: req.source.unwrap_or_else(|| "user".to_string()),
            created_at: now.clone(),
            updated_at: now.clone(),
            accessed_at: now,
            access_count: 0,
            user_id: None,
            sync_status: "local_only".to_string(),
            sync_version: 0,
            is_deleted: false,
        };

        self.repo.store(&memory)?;
        Ok(memory)
    }

    /// 搜索
    pub fn search(&self, query: MemoryQuery) -> Result<MemoryQueryResult, String> {
        self.repo.search(&query)
    }

    /// 列表（无搜索词）
    pub fn list(
        &self,
        scope: Option<MemoryScope>,
        workspace_name: Option<&str>,
        project_path: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<MemoryQueryResult, String> {
        let query = MemoryQuery {
            scope,
            workspace_name: workspace_name.map(String::from),
            project_path: project_path.map(String::from),
            limit,
            offset,
            ..Default::default()
        };
        self.repo.search(&query)
    }

    /// 按 ID 获取
    pub fn get(&self, id: &str) -> Result<Option<Memory>, String> {
        let result = self.repo.get_by_id(id)?;
        if result.is_some() {
            let _ = self.repo.touch(id);
        }
        Ok(result)
    }

    /// 更新
    pub fn update(&self, id: &str, req: UpdateMemoryRequest) -> Result<bool, String> {
        if let Some(ref title) = req.title {
            if title.trim().is_empty() {
                return Err("标题不能为空".to_string());
            }
            if title.len() > 200 {
                return Err("标题不能超过 200 字符".to_string());
            }
        }
        if let Some(importance) = req.importance {
            if !(1..=5).contains(&importance) {
                return Err("重要度必须在 1-5 之间".to_string());
            }
        }
        self.repo.update(id, &req)
    }

    /// 删除
    pub fn delete(&self, id: &str) -> Result<bool, String> {
        self.repo.delete(id)
    }

    /// 统计
    pub fn stats(
        &self,
        workspace_name: Option<&str>,
        project_path: Option<&str>,
    ) -> Result<MemoryStats, String> {
        self.repo.stats(workspace_name, project_path)
    }

    /// 格式化 Memory 为 Markdown（用于注入）
    pub fn format_for_injection(&self, ids: &[String]) -> Result<String, String> {
        let mut sections = Vec::new();

        for id in ids {
            if let Some(memory) = self.repo.get_by_id(id)? {
                let _ = self.repo.touch(id);
                sections.push(format!(
                    "## {} [{}]\n\n{}\n",
                    memory.title, memory.category, memory.content
                ));
            }
        }

        if sections.is_empty() {
            return Ok(String::new());
        }

        Ok(format!(
            "# Memory Context\n\n{}",
            sections.join("\n---\n\n")
        ))
    }

    /// 获取项目摘要（高重要度 Memory）
    pub fn get_project_summary(
        &self,
        project_path: &str,
        min_importance: u8,
        limit: u32,
    ) -> Result<Vec<Memory>, String> {
        let query = MemoryQuery {
            project_path: Some(project_path.to_string()),
            min_importance: Some(min_importance),
            sort_by: Some("importance".to_string()),
            limit: Some(limit),
            ..Default::default()
        };
        let result = self.repo.search(&query)?;
        Ok(result.items)
    }
}
