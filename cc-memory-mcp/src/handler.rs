use cc_memory::models::*;
use cc_memory::report::ReportGenerator;
use cc_memory::repository::MemoryRepository;
use cc_memory::service::MemoryService;

use crate::protocol::*;

pub struct ToolHandler {
    service: MemoryService,
    repo: MemoryRepository,
    default_project_path: Option<String>,
    default_workspace_name: Option<String>,
}

impl ToolHandler {
    pub fn new(
        service: MemoryService,
        repo: MemoryRepository,
        default_project_path: Option<String>,
        default_workspace_name: Option<String>,
    ) -> Self {
        Self {
            service,
            repo,
            default_project_path,
            default_workspace_name,
        }
    }

    /// 返回所有可用工具的定义
    pub fn get_tools(&self) -> Vec<Tool> {
        vec![
            self.tool_memory_add(),
            self.tool_memory_search(),
            self.tool_memory_update(),
            self.tool_memory_delete(),
            self.tool_memory_daily_report(),
        ]
    }

    /// 分发工具调用
    pub fn handle_tool_call(&self, name: &str, arguments: &serde_json::Value) -> ToolResult {
        match name {
            "memory_add" => self.handle_memory_add(arguments),
            "memory_search" => self.handle_memory_search(arguments),
            "memory_update" => self.handle_memory_update(arguments),
            "memory_delete" => self.handle_memory_delete(arguments),
            "memory_daily_report" => self.handle_memory_daily_report(arguments),
            _ => ToolResult::error(format!("Unknown tool: {}", name)),
        }
    }

    // ---- Tool Definitions ----

    fn tool_memory_add(&self) -> Tool {
        Tool {
            name: "memory_add".to_string(),
            description: "Store a new memory entry. Use this to save decisions, lessons learned, preferences, patterns, facts, or plans for future reference.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Short title for the memory (max 200 chars)"
                    },
                    "content": {
                        "type": "string",
                        "description": "Detailed content of the memory"
                    },
                    "category": {
                        "type": "string",
                        "enum": ["decision", "lesson", "preference", "pattern", "fact", "plan"],
                        "description": "Category of the memory. Default: fact"
                    },
                    "importance": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "Importance level 1-5. Default: 3"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tags for categorization"
                    },
                    "scope": {
                        "type": "string",
                        "enum": ["global", "workspace", "project", "session"],
                        "description": "Scope of the memory. Default: project"
                    }
                },
                "required": ["title", "content"]
            }),
        }
    }

    fn tool_memory_search(&self) -> Tool {
        Tool {
            name: "memory_search".to_string(),
            description: "Search memories using full-text search. Returns relevant memories from the current project/workspace context.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (full-text search)"
                    },
                    "scope": {
                        "type": "string",
                        "enum": ["global", "workspace", "project", "session"],
                        "description": "Filter by scope"
                    },
                    "category": {
                        "type": "string",
                        "enum": ["decision", "lesson", "preference", "pattern", "fact", "plan"],
                        "description": "Filter by category"
                    },
                    "min_importance": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "Minimum importance level"
                    },
                    "from_date": {
                        "type": "string",
                        "description": "Start date filter (RFC3339)"
                    },
                    "to_date": {
                        "type": "string",
                        "description": "End date filter (RFC3339)"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "description": "Max results to return. Default: 20"
                    }
                }
            }),
        }
    }

    fn tool_memory_update(&self) -> Tool {
        Tool {
            name: "memory_update".to_string(),
            description: "Update an existing memory entry by ID.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Memory ID to update"
                    },
                    "title": {
                        "type": "string",
                        "description": "New title (max 200 chars)"
                    },
                    "content": {
                        "type": "string",
                        "description": "New content"
                    },
                    "importance": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "New importance level"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "New tags (replaces existing)"
                    },
                    "category": {
                        "type": "string",
                        "enum": ["decision", "lesson", "preference", "pattern", "fact", "plan"],
                        "description": "New category"
                    }
                },
                "required": ["id"]
            }),
        }
    }

    fn tool_memory_delete(&self) -> Tool {
        Tool {
            name: "memory_delete".to_string(),
            description: "Delete a memory entry by ID (soft delete).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Memory ID to delete"
                    }
                },
                "required": ["id"]
            }),
        }
    }

    fn tool_memory_daily_report(&self) -> Tool {
        Tool {
            name: "memory_daily_report".to_string(),
            description: "Generate a daily report of memories created on a specific date."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "date": {
                        "type": "string",
                        "description": "Date in YYYY-MM-DD format"
                    },
                    "workspace_name": {
                        "type": "string",
                        "description": "Filter by workspace name"
                    },
                    "project_path": {
                        "type": "string",
                        "description": "Filter by project path"
                    }
                },
                "required": ["date"]
            }),
        }
    }

    // ---- Tool Handlers ----

    fn handle_memory_add(&self, args: &serde_json::Value) -> ToolResult {
        let title = match args.get("title").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => return ToolResult::error("Missing required parameter: title".to_string()),
        };
        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => return ToolResult::error("Missing required parameter: content".to_string()),
        };

        let scope = args
            .get("scope")
            .and_then(|v| v.as_str())
            .and_then(MemoryScope::parse);

        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .map(MemoryCategory::parse);

        let importance = args
            .get("importance")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);

        let tags = args.get("tags").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
        });

        // 根据 scope 自动注入 project_path / workspace_name
        let effective_scope = scope.clone().unwrap_or(MemoryScope::Project);
        let project_path = match effective_scope {
            MemoryScope::Project | MemoryScope::Session => self.default_project_path.clone(),
            _ => None,
        };
        let workspace_name = match effective_scope {
            MemoryScope::Workspace | MemoryScope::Project | MemoryScope::Session => {
                self.default_workspace_name.clone()
            }
            _ => None,
        };

        let req = StoreMemoryRequest {
            title,
            content,
            scope: Some(effective_scope),
            category,
            importance,
            workspace_name,
            project_path,
            session_id: None,
            tags,
            source: Some("mcp".to_string()),
        };

        match self.service.store(req) {
            Ok(memory) => {
                let output = format!(
                    "Memory stored successfully.\nID: {}\nTitle: {}\nScope: {}\nCategory: {}",
                    memory.id, memory.title, memory.scope, memory.category
                );
                ToolResult::text(output)
            }
            Err(e) => ToolResult::error(format!("Failed to store memory: {}", e)),
        }
    }

    fn handle_memory_search(&self, args: &serde_json::Value) -> ToolResult {
        let search = args.get("query").and_then(|v| v.as_str()).map(String::from);

        let scope = args
            .get("scope")
            .and_then(|v| v.as_str())
            .and_then(MemoryScope::parse);

        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .map(MemoryCategory::parse);

        let min_importance = args
            .get("min_importance")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);

        let from_date = args
            .get("from_date")
            .and_then(|v| v.as_str())
            .map(String::from);

        let to_date = args
            .get("to_date")
            .and_then(|v| v.as_str())
            .map(String::from);

        let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as u32);

        // 自动注入项目/工作空间上下文
        let query = MemoryQuery {
            search,
            scope,
            category,
            min_importance,
            workspace_name: self.default_workspace_name.clone(),
            project_path: self.default_project_path.clone(),
            session_id: None,
            tags: None,
            from_date,
            to_date,
            sort_by: None,
            limit,
            offset: None,
        };

        match self.service.search(query) {
            Ok(result) => {
                if result.items.is_empty() {
                    return ToolResult::text("No memories found.".to_string());
                }
                let output = format_search_results(&result);
                ToolResult::text(output)
            }
            Err(e) => ToolResult::error(format!("Search failed: {}", e)),
        }
    }

    fn handle_memory_update(&self, args: &serde_json::Value) -> ToolResult {
        let id = match args.get("id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return ToolResult::error("Missing required parameter: id".to_string()),
        };

        let title = args.get("title").and_then(|v| v.as_str()).map(String::from);

        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .map(String::from);

        let importance = args
            .get("importance")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);

        let tags = args.get("tags").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
        });

        let category = args
            .get("category")
            .and_then(|v| v.as_str())
            .map(MemoryCategory::parse);

        let req = UpdateMemoryRequest {
            title,
            content,
            category,
            importance,
            tags,
        };

        match self.service.update(id, req) {
            Ok(true) => ToolResult::text(format!("Memory {} updated successfully.", id)),
            Ok(false) => ToolResult::error(format!("Memory {} not found.", id)),
            Err(e) => ToolResult::error(format!("Failed to update memory: {}", e)),
        }
    }

    fn handle_memory_delete(&self, args: &serde_json::Value) -> ToolResult {
        let id = match args.get("id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return ToolResult::error("Missing required parameter: id".to_string()),
        };

        match self.service.delete(id) {
            Ok(true) => ToolResult::text(format!("Memory {} deleted successfully.", id)),
            Ok(false) => ToolResult::error(format!("Memory {} not found.", id)),
            Err(e) => ToolResult::error(format!("Failed to delete memory: {}", e)),
        }
    }

    fn handle_memory_daily_report(&self, args: &serde_json::Value) -> ToolResult {
        let date = match args.get("date").and_then(|v| v.as_str()) {
            Some(d) => d.to_string(),
            None => return ToolResult::error("Missing required parameter: date".to_string()),
        };

        let workspace_name = args
            .get("workspace_name")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.default_workspace_name.clone());

        let project_path = args
            .get("project_path")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.default_project_path.clone());

        let query = DailyReportQuery {
            date,
            workspace_name,
            project_path,
        };

        let generator = ReportGenerator::new(&self.repo);
        match generator.daily_report(&query) {
            Ok(report) => {
                let markdown = generator.format_daily_report_markdown(&report);
                ToolResult::text(markdown)
            }
            Err(e) => ToolResult::error(format!("Failed to generate daily report: {}", e)),
        }
    }
}

/// 格式化搜索结果为可读的 Markdown
fn format_search_results(result: &MemoryQueryResult) -> String {
    let mut output = format!(
        "Found {} memories (showing {}):\n\n",
        result.total,
        result.items.len()
    );

    for (i, memory) in result.items.iter().enumerate() {
        output.push_str(&format!(
            "### {}. {} [{}] (importance: {})\n",
            i + 1,
            memory.title,
            memory.category,
            memory.importance
        ));
        output.push_str(&format!("**ID**: {}\n", memory.id));
        output.push_str(&format!("**Scope**: {}\n", memory.scope));
        if !memory.tags.is_empty() {
            output.push_str(&format!("**Tags**: {}\n", memory.tags.join(", ")));
        }
        output.push_str(&format!("**Created**: {}\n\n", memory.created_at));
        output.push_str(&memory.content);
        output.push_str("\n\n---\n\n");
    }

    if result.has_more {
        output.push_str("_More results available. Increase `limit` to see more._\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_memory::db::MemoryDatabase;

    fn create_test_handler() -> ToolHandler {
        let db = MemoryDatabase::new_memory().unwrap();
        let repo = MemoryRepository::new(db);
        let db2 = MemoryDatabase::new_memory().unwrap();
        let repo2 = MemoryRepository::new(db2);
        let service = MemoryService::new(repo);
        ToolHandler::new(
            service,
            repo2,
            Some("/test/project".to_string()),
            Some("test-workspace".to_string()),
        )
    }

    #[test]
    fn test_get_tools_returns_five_tools() {
        let handler = create_test_handler();
        let tools = handler.get_tools();
        assert_eq!(tools.len(), 5);

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"memory_add"));
        assert!(names.contains(&"memory_search"));
        assert!(names.contains(&"memory_update"));
        assert!(names.contains(&"memory_delete"));
        assert!(names.contains(&"memory_daily_report"));
    }

    #[test]
    fn test_handle_unknown_tool() {
        let handler = create_test_handler();
        let result = handler.handle_tool_call("unknown_tool", &serde_json::json!({}));
        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("Unknown tool"));
    }

    #[test]
    fn test_handle_memory_add_missing_title() {
        let handler = create_test_handler();
        let result = handler.handle_tool_call(
            "memory_add",
            &serde_json::json!({"content": "test content"}),
        );
        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("title"));
    }

    #[test]
    fn test_handle_memory_add_missing_content() {
        let handler = create_test_handler();
        let result = handler.handle_tool_call("memory_add", &serde_json::json!({"title": "test"}));
        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("content"));
    }

    #[test]
    fn test_handle_memory_search_empty() {
        let handler = create_test_handler();
        let result = handler.handle_tool_call("memory_search", &serde_json::json!({}));
        assert!(result.is_error.is_none());
        assert!(result.content[0].text.contains("No memories found"));
    }

    #[test]
    fn test_handle_memory_update_missing_id() {
        let handler = create_test_handler();
        let result =
            handler.handle_tool_call("memory_update", &serde_json::json!({"title": "new title"}));
        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("id"));
    }

    #[test]
    fn test_handle_memory_delete_missing_id() {
        let handler = create_test_handler();
        let result = handler.handle_tool_call("memory_delete", &serde_json::json!({}));
        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("id"));
    }

    #[test]
    fn test_handle_memory_daily_report_missing_date() {
        let handler = create_test_handler();
        let result = handler.handle_tool_call("memory_daily_report", &serde_json::json!({}));
        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("date"));
    }

    #[test]
    fn test_format_search_results_empty() {
        let result = MemoryQueryResult {
            items: vec![],
            total: 0,
            has_more: false,
        };
        let output = format_search_results(&result);
        assert!(output.contains("Found 0 memories"));
    }
}
