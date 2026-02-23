use serde::{Deserialize, Serialize};

/// Memory 作用域
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    Global,
    Workspace,
    Project,
    Session,
}

impl MemoryScope {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Global => "global",
            Self::Workspace => "workspace",
            Self::Project => "project",
            Self::Session => "session",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "global" => Some(Self::Global),
            "workspace" => Some(Self::Workspace),
            "project" => Some(Self::Project),
            "session" => Some(Self::Session),
            _ => None,
        }
    }
}

impl std::fmt::Display for MemoryScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Memory 类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    Decision,
    Lesson,
    Preference,
    Pattern,
    Fact,
    Plan,
    Custom(String),
}

impl MemoryCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Decision => "decision",
            Self::Lesson => "lesson",
            Self::Preference => "preference",
            Self::Pattern => "pattern",
            Self::Fact => "fact",
            Self::Plan => "plan",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "decision" => Self::Decision,
            "lesson" => Self::Lesson,
            "preference" => Self::Preference,
            "pattern" => Self::Pattern,
            "fact" => Self::Fact,
            "plan" => Self::Plan,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Memory 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub title: String,
    pub content: String,
    pub scope: MemoryScope,
    pub category: MemoryCategory,
    pub importance: u8, // 1-5
    pub workspace_name: Option<String>,
    pub project_path: Option<String>,
    pub session_id: Option<String>,
    pub tags: Vec<String>,
    pub source: String,     // "user" / "agent" / "mcp" / "hook"
    pub created_at: String, // RFC3339
    pub updated_at: String,
    pub accessed_at: String,
    pub access_count: u32,
    // 云端预留
    pub user_id: Option<String>,
    pub sync_status: String, // local_only / synced / pending_sync
    pub sync_version: u64,
    pub is_deleted: bool,
}

/// Memory 查询参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub search: Option<String>, // FTS5 全文搜索
    pub scope: Option<MemoryScope>,
    pub category: Option<MemoryCategory>,
    pub min_importance: Option<u8>,
    pub workspace_name: Option<String>,
    pub project_path: Option<String>,
    pub session_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub from_date: Option<String>, // RFC3339
    pub to_date: Option<String>,
    pub sort_by: Option<String>, // "relevance" / "created_at" / "updated_at" / "importance"
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Memory 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQueryResult {
    pub items: Vec<Memory>,
    pub total: u64,
    pub has_more: bool,
}

/// Memory 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total: u64,
    pub by_scope: std::collections::HashMap<String, u64>,
    pub by_category: std::collections::HashMap<String, u64>,
}

/// 日报查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReportQuery {
    pub date: String, // YYYY-MM-DD
    pub workspace_name: Option<String>,
    pub project_path: Option<String>,
}

/// 日报条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReportEntry {
    pub category: String,
    pub items: Vec<Memory>,
}

/// 日报结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReport {
    pub date: String,
    pub entries: Vec<DailyReportEntry>,
    pub total_count: u64,
}

/// 创建 Memory 的请求
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoreMemoryRequest {
    pub title: String,
    pub content: String,
    pub scope: Option<MemoryScope>,
    pub category: Option<MemoryCategory>,
    pub importance: Option<u8>,
    pub workspace_name: Option<String>,
    pub project_path: Option<String>,
    pub session_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub source: Option<String>,
}

/// 更新 Memory 的请求
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateMemoryRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<MemoryCategory>,
    pub importance: Option<u8>,
    pub tags: Option<Vec<String>>,
}
