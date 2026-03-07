use crate::utils::error::AppError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// 数据库连接管理
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// 创建新的数据库连接
    pub fn new(db_path: PathBuf) -> Result<Self, AppError> {
        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::from(format!("Failed to create database directory: {}", e)))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::from(format!("Failed to open database: {}", e)))?;
        Self::init_tables(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 降级到内存数据库（磁盘数据库失败时的 fallback）
    pub fn new_fallback() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| AppError::from(format!("Failed to create fallback in-memory database: {}", e)))?;
        Self::init_tables(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 初始化数据库表结构
    fn init_tables(conn: &Connection) -> Result<(), AppError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                alias TEXT
            )",
            [],
        )
        .map_err(|e| AppError::from(format!("Failed to create projects table: {}", e)))?;

        // 迁移：为旧表添加 alias 字段
        let _ = conn.execute("ALTER TABLE projects ADD COLUMN alias TEXT", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS launch_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                project_name TEXT NOT NULL,
                project_path TEXT NOT NULL,
                launched_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::from(format!("Failed to create launch_history table: {}", e)))?;

        // Todo 表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS todos (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT DEFAULT '',
                status TEXT NOT NULL DEFAULT 'todo',
                priority TEXT NOT NULL DEFAULT 'medium',
                scope TEXT NOT NULL DEFAULT 'global',
                scope_ref TEXT,
                tags TEXT DEFAULT '[]',
                due_date TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::from(format!("Failed to create todos table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS todo_subtasks (
                id TEXT PRIMARY KEY,
                todo_id TEXT NOT NULL,
                title TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (todo_id) REFERENCES todos(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| AppError::from(format!("Failed to create todo_subtasks table: {}", e)))?;

        // 迁移：为 launch_history 表添加 claude_session_id、last_prompt、workspace_name、workspace_path 字段
        let _ = conn.execute("ALTER TABLE launch_history ADD COLUMN claude_session_id TEXT", []);
        let _ = conn.execute("ALTER TABLE launch_history ADD COLUMN last_prompt TEXT", []);
        let _ = conn.execute("ALTER TABLE launch_history ADD COLUMN workspace_name TEXT", []);
        let _ = conn.execute("ALTER TABLE launch_history ADD COLUMN workspace_path TEXT", []);
        let _ = conn.execute("ALTER TABLE launch_history ADD COLUMN launch_cwd TEXT", []);

        // 迁移：为 todos 表添加 my_day、my_day_date、reminder_at、recurrence 字段
        let _ = conn.execute("ALTER TABLE todos ADD COLUMN my_day INTEGER DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE todos ADD COLUMN my_day_date TEXT", []);
        let _ = conn.execute("ALTER TABLE todos ADD COLUMN reminder_at TEXT", []);
        let _ = conn.execute("ALTER TABLE todos ADD COLUMN recurrence TEXT", []);

        // 迁移：为 todos 表添加 todo_type 字段
        let _ = conn.execute("ALTER TABLE todos ADD COLUMN todo_type TEXT DEFAULT ''", []);

        Ok(())
    }

    /// 创建内存数据库（用于测试）
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, AppError> {
        Self::new_fallback()
    }

    /// 获取数据库连接的可变引用
    pub fn connection(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|_| AppError::from("Database lock poisoned"))
    }
}
