use rusqlite::Connection;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// Memory 数据库
pub struct MemoryDatabase {
    conn: Mutex<Connection>,
}

impl MemoryDatabase {
    /// 打开或创建 memory.db
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        // 确保父目录存在
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("无法创建 memory 数据库目录: {}", e))?;
        }

        let conn = Connection::open(path).map_err(|e| format!("无法打开 memory 数据库: {}", e))?;

        // WAL 模式 + busy timeout
        // PRAGMA journal_mode 始终返回结果行，在 extra_check 下 execute/execute_batch/pragma_update
        // 都会抛出 ExecuteReturnedResults，必须用 query_row 接收结果
        let _: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(|e| format!("设置 PRAGMA journal_mode 失败: {}", e))?;
        // busy_timeout 设置时也会返回结果行
        let _: i32 = conn
            .query_row("PRAGMA busy_timeout=5000", [], |row| row.get(0))
            .map_err(|e| format!("设置 PRAGMA busy_timeout 失败: {}", e))?;
        conn.execute("PRAGMA foreign_keys=ON", [])
            .map_err(|e| format!("设置 PRAGMA foreign_keys 失败: {}", e))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    /// 内存数据库（用于测试）
    pub fn new_memory() -> Result<Self, String> {
        let conn =
            Connection::open_in_memory().map_err(|e| format!("无法创建内存数据库: {}", e))?;
        conn.execute("PRAGMA foreign_keys=ON", [])
            .map_err(|e| format!("设置 PRAGMA foreign_keys 失败: {}", e))?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    /// 获取连接锁
    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("Memory database lock poisoned")
    }

    fn init_tables(&self) -> Result<(), String> {
        let conn = self.conn();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                scope TEXT NOT NULL DEFAULT 'project',
                category TEXT NOT NULL DEFAULT 'fact',
                importance INTEGER NOT NULL DEFAULT 3 CHECK(importance BETWEEN 1 AND 5),
                workspace_name TEXT,
                project_path TEXT,
                session_id TEXT,
                tags TEXT DEFAULT '[]',
                source TEXT NOT NULL DEFAULT 'user',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                accessed_at TEXT NOT NULL,
                access_count INTEGER DEFAULT 0,
                user_id TEXT,
                sync_status TEXT DEFAULT 'local_only',
                sync_version INTEGER DEFAULT 0,
                is_deleted INTEGER DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(scope);
            CREATE INDEX IF NOT EXISTS idx_memories_category ON memories(category);
            CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance);
            CREATE INDEX IF NOT EXISTS idx_memories_workspace ON memories(workspace_name);
            CREATE INDEX IF NOT EXISTS idx_memories_project ON memories(project_path);
            CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at);
            CREATE INDEX IF NOT EXISTS idx_memories_updated ON memories(updated_at);
            CREATE INDEX IF NOT EXISTS idx_memories_sync ON memories(sync_status);

            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                title, content, tags,
                content=memories, content_rowid=rowid
            );

            -- FTS5 同步触发器
            CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(rowid, title, content, tags)
                VALUES (new.rowid, new.title, new.content, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, title, content, tags)
                VALUES ('delete', old.rowid, old.title, old.content, old.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, title, content, tags)
                VALUES ('delete', old.rowid, old.title, old.content, old.tags);
                INSERT INTO memories_fts(rowid, title, content, tags)
                VALUES (new.rowid, new.title, new.content, new.tags);
            END;",
        )
        .map_err(|e| format!("初始化 memory 表失败: {}", e))?;

        Ok(())
    }
}
