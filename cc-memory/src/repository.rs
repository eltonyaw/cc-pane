use crate::db::MemoryDatabase;
use crate::models::*;
use rusqlite::params;
use rusqlite::OptionalExtension;

pub struct MemoryRepository {
    db: MemoryDatabase,
}

impl MemoryRepository {
    pub fn new(db: MemoryDatabase) -> Self {
        Self { db }
    }

    /// 存储新 Memory
    pub fn store(&self, memory: &Memory) -> Result<(), String> {
        let conn = self.db.conn();
        let tags_json = serde_json::to_string(&memory.tags).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO memories (id, title, content, scope, category, importance,
             workspace_name, project_path, session_id, tags, source,
             created_at, updated_at, accessed_at, access_count,
             user_id, sync_status, sync_version, is_deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                memory.id,
                memory.title,
                memory.content,
                memory.scope.as_str(),
                memory.category.as_str(),
                memory.importance,
                memory.workspace_name,
                memory.project_path,
                memory.session_id,
                tags_json,
                memory.source,
                memory.created_at,
                memory.updated_at,
                memory.accessed_at,
                memory.access_count,
                memory.user_id,
                memory.sync_status,
                memory.sync_version,
                memory.is_deleted as i32,
            ],
        )
        .map_err(|e| format!("存储 memory 失败: {}", e))?;

        Ok(())
    }

    /// 按 ID 获取
    pub fn get_by_id(&self, id: &str) -> Result<Option<Memory>, String> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, scope, category, importance,
             workspace_name, project_path, session_id, tags, source,
             created_at, updated_at, accessed_at, access_count,
             user_id, sync_status, sync_version, is_deleted
             FROM memories WHERE id = ?1 AND is_deleted = 0",
            )
            .map_err(|e| format!("prepare 失败: {}", e))?;

        let result = stmt
            .query_row(params![id], |row| Ok(Self::row_to_memory(row)))
            .optional()
            .map_err(|e| format!("查询失败: {}", e))?;

        match result {
            Some(inner) => inner.map(Some),
            None => Ok(None),
        }
    }

    /// 更新
    pub fn update(&self, id: &str, req: &UpdateMemoryRequest) -> Result<bool, String> {
        let conn = self.db.conn();
        let now = chrono::Utc::now().to_rfc3339();

        // 检查存在性
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM memories WHERE id = ?1 AND is_deleted = 0",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("查询失败: {}", e))?;

        if !exists {
            return Ok(false);
        }

        // 动态构建 SQL -- 只更新非 None 字段
        // 由于 rusqlite 不支持动态参数列表，我们逐字段更新
        if let Some(ref title) = req.title {
            conn.execute(
                "UPDATE memories SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![title, &now, id],
            )
            .map_err(|e| format!("更新 title 失败: {}", e))?;
        }
        if let Some(ref content) = req.content {
            conn.execute(
                "UPDATE memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![content, &now, id],
            )
            .map_err(|e| format!("更新 content 失败: {}", e))?;
        }
        if let Some(ref category) = req.category {
            conn.execute(
                "UPDATE memories SET category = ?1, updated_at = ?2 WHERE id = ?3",
                params![category.as_str(), &now, id],
            )
            .map_err(|e| format!("更新 category 失败: {}", e))?;
        }
        if let Some(importance) = req.importance {
            conn.execute(
                "UPDATE memories SET importance = ?1, updated_at = ?2 WHERE id = ?3",
                params![importance, &now, id],
            )
            .map_err(|e| format!("更新 importance 失败: {}", e))?;
        }
        if let Some(ref tags) = req.tags {
            let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
            conn.execute(
                "UPDATE memories SET tags = ?1, updated_at = ?2 WHERE id = ?3",
                params![tags_json, &now, id],
            )
            .map_err(|e| format!("更新 tags 失败: {}", e))?;
        }

        Ok(true)
    }

    /// 软删除
    pub fn delete(&self, id: &str) -> Result<bool, String> {
        let conn = self.db.conn();
        let now = chrono::Utc::now().to_rfc3339();
        let affected = conn
            .execute(
                "UPDATE memories SET is_deleted = 1, updated_at = ?1 WHERE id = ?2 AND is_deleted = 0",
                params![&now, id],
            )
            .map_err(|e| format!("删除失败: {}", e))?;

        Ok(affected > 0)
    }

    /// FTS5 搜索 + 过滤
    pub fn search(&self, query: &MemoryQuery) -> Result<MemoryQueryResult, String> {
        let conn = self.db.conn();
        let limit = query.limit.unwrap_or(20) as i64;
        let offset = query.offset.unwrap_or(0) as i64;

        // 优先使用 FTS5，失败时降级为 LIKE
        if let Some(ref search_text) = query.search {
            if !search_text.trim().is_empty() {
                return self
                    .search_fts(&conn, query, search_text, limit, offset)
                    .or_else(|_| self.search_like(&conn, query, search_text, limit, offset));
            }
        }

        // 无搜索词 -- 纯过滤
        self.search_filter(&conn, query, limit, offset)
    }

    /// 统计
    pub fn stats(
        &self,
        workspace_name: Option<&str>,
        project_path: Option<&str>,
    ) -> Result<MemoryStats, String> {
        let conn = self.db.conn();

        let mut where_clauses = vec!["is_deleted = 0".to_string()];
        if let Some(ws) = workspace_name {
            where_clauses.push(format!("workspace_name = '{}'", ws.replace('\'', "''")));
        }
        if let Some(pp) = project_path {
            where_clauses.push(format!("project_path = '{}'", pp.replace('\'', "''")));
        }
        let where_sql = where_clauses.join(" AND ");

        let total: u64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM memories WHERE {}", where_sql),
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("统计失败: {}", e))?;

        let mut by_scope = std::collections::HashMap::new();
        {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT scope, COUNT(*) FROM memories WHERE {} GROUP BY scope",
                    where_sql
                ))
                .map_err(|e| format!("统计 scope 失败: {}", e))?;
            let scope_rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
                })
                .map_err(|e| format!("查询 scope 失败: {}", e))?;
            for row in scope_rows {
                let (scope, count) = row.map_err(|e| format!("读取 scope 行失败: {}", e))?;
                by_scope.insert(scope, count);
            }
        }

        let mut by_category = std::collections::HashMap::new();
        {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT category, COUNT(*) FROM memories WHERE {} GROUP BY category",
                    where_sql
                ))
                .map_err(|e| format!("统计 category 失败: {}", e))?;
            let cat_rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
                })
                .map_err(|e| format!("查询 category 失败: {}", e))?;
            for row in cat_rows {
                let (cat, count) = row.map_err(|e| format!("读取 category 行失败: {}", e))?;
                by_category.insert(cat, count);
            }
        }

        Ok(MemoryStats {
            total,
            by_scope,
            by_category,
        })
    }

    /// 更新访问时间和次数
    pub fn touch(&self, id: &str) -> Result<(), String> {
        let conn = self.db.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE memories SET accessed_at = ?1, access_count = access_count + 1 WHERE id = ?2",
            params![&now, id],
        )
        .map_err(|e| format!("更新访问记录失败: {}", e))?;
        Ok(())
    }

    /// 按日期范围查询（日报用）
    pub fn list_by_date_range(
        &self,
        from: &str,
        to: &str,
        workspace_name: Option<&str>,
        project_path: Option<&str>,
    ) -> Result<Vec<Memory>, String> {
        let conn = self.db.conn();
        let mut where_clauses = vec![
            "is_deleted = 0".to_string(),
            "created_at >= ?1".to_string(),
            "created_at < ?2".to_string(),
        ];
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(from.to_string()), Box::new(to.to_string())];

        if let Some(ws) = workspace_name {
            where_clauses.push(format!("workspace_name = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(ws.to_string()));
        }
        if let Some(pp) = project_path {
            where_clauses.push(format!("project_path = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(pp.to_string()));
        }

        let sql = format!(
            "SELECT id, title, content, scope, category, importance,
             workspace_name, project_path, session_id, tags, source,
             created_at, updated_at, accessed_at, access_count,
             user_id, sync_status, sync_version, is_deleted
             FROM memories WHERE {} ORDER BY created_at ASC",
            where_clauses.join(" AND ")
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("prepare 失败: {}", e))?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| Ok(Self::row_to_memory(row)))
            .map_err(|e| format!("查询失败: {}", e))?;

        let mut result = Vec::new();
        for row in rows {
            let inner = row.map_err(|e| format!("读取行失败: {}", e))?;
            result.push(inner?);
        }

        Ok(result)
    }

    // ---- 内部方法 ----

    fn search_fts(
        &self,
        conn: &rusqlite::Connection,
        query: &MemoryQuery,
        search_text: &str,
        limit: i64,
        offset: i64,
    ) -> Result<MemoryQueryResult, String> {
        let mut where_clauses = vec!["m.is_deleted = 0".to_string()];
        self.build_filter_clauses(&mut where_clauses, query, "m.");
        let where_sql = where_clauses.join(" AND ");

        // 计总数
        let count_sql = format!(
            "SELECT COUNT(*) FROM memories m
             INNER JOIN memories_fts ON memories_fts.rowid = m.rowid
             WHERE memories_fts MATCH ?1 AND {}",
            where_sql
        );
        let total: u64 = conn
            .query_row(&count_sql, params![search_text], |row| row.get(0))
            .map_err(|e| format!("FTS5 计数失败: {}", e))?;

        // 查数据
        let data_sql = format!(
            "SELECT m.id, m.title, m.content, m.scope, m.category, m.importance,
             m.workspace_name, m.project_path, m.session_id, m.tags, m.source,
             m.created_at, m.updated_at, m.accessed_at, m.access_count,
             m.user_id, m.sync_status, m.sync_version, m.is_deleted
             FROM memories m
             INNER JOIN memories_fts ON memories_fts.rowid = m.rowid
             WHERE memories_fts MATCH ?1 AND {}
             ORDER BY rank
             LIMIT ?2 OFFSET ?3",
            where_sql
        );

        let mut stmt = conn
            .prepare(&data_sql)
            .map_err(|e| format!("FTS5 查询 prepare 失败: {}", e))?;
        let rows = stmt
            .query_map(params![search_text, limit, offset], |row| {
                Ok(Self::row_to_memory(row))
            })
            .map_err(|e| format!("FTS5 查询失败: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            let inner = row.map_err(|e| format!("读取行失败: {}", e))?;
            items.push(inner?);
        }

        Ok(MemoryQueryResult {
            has_more: total > (offset as u64 + items.len() as u64),
            items,
            total,
        })
    }

    fn search_like(
        &self,
        conn: &rusqlite::Connection,
        query: &MemoryQuery,
        search_text: &str,
        limit: i64,
        offset: i64,
    ) -> Result<MemoryQueryResult, String> {
        let like_pattern = format!("%{}%", search_text.replace('%', "\\%").replace('_', "\\_"));

        let mut where_clauses = vec![
            "is_deleted = 0".to_string(),
            "(title LIKE ?1 ESCAPE '\\' OR content LIKE ?1 ESCAPE '\\' OR tags LIKE ?1 ESCAPE '\\')"
                .to_string(),
        ];
        self.build_filter_clauses(&mut where_clauses, query, "");
        let where_sql = where_clauses.join(" AND ");

        let count_sql = format!("SELECT COUNT(*) FROM memories WHERE {}", where_sql);
        let total: u64 = conn
            .query_row(&count_sql, params![&like_pattern], |row| row.get(0))
            .map_err(|e| format!("LIKE 计数失败: {}", e))?;

        let data_sql = format!(
            "SELECT id, title, content, scope, category, importance,
             workspace_name, project_path, session_id, tags, source,
             created_at, updated_at, accessed_at, access_count,
             user_id, sync_status, sync_version, is_deleted
             FROM memories WHERE {}
             ORDER BY importance DESC, updated_at DESC
             LIMIT ?2 OFFSET ?3",
            where_sql
        );

        let mut stmt = conn
            .prepare(&data_sql)
            .map_err(|e| format!("LIKE 查询 prepare 失败: {}", e))?;
        let rows = stmt
            .query_map(params![&like_pattern, limit, offset], |row| {
                Ok(Self::row_to_memory(row))
            })
            .map_err(|e| format!("LIKE 查询失败: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            let inner = row.map_err(|e| format!("读取行失败: {}", e))?;
            items.push(inner?);
        }

        Ok(MemoryQueryResult {
            has_more: total > (offset as u64 + items.len() as u64),
            items,
            total,
        })
    }

    fn search_filter(
        &self,
        conn: &rusqlite::Connection,
        query: &MemoryQuery,
        limit: i64,
        offset: i64,
    ) -> Result<MemoryQueryResult, String> {
        let mut where_clauses = vec!["is_deleted = 0".to_string()];
        self.build_filter_clauses(&mut where_clauses, query, "");
        let where_sql = where_clauses.join(" AND ");

        let sort = match query.sort_by.as_deref() {
            Some("importance") => "importance DESC, updated_at DESC",
            Some("created_at") => "created_at DESC",
            Some("updated_at") | None => "updated_at DESC",
            Some(_) => "updated_at DESC",
        };

        let count_sql = format!("SELECT COUNT(*) FROM memories WHERE {}", where_sql);
        let total: u64 = conn
            .query_row(&count_sql, [], |row| row.get(0))
            .map_err(|e| format!("过滤计数失败: {}", e))?;

        let data_sql = format!(
            "SELECT id, title, content, scope, category, importance,
             workspace_name, project_path, session_id, tags, source,
             created_at, updated_at, accessed_at, access_count,
             user_id, sync_status, sync_version, is_deleted
             FROM memories WHERE {} ORDER BY {} LIMIT ?1 OFFSET ?2",
            where_sql, sort
        );

        let mut stmt = conn
            .prepare(&data_sql)
            .map_err(|e| format!("过滤查询 prepare 失败: {}", e))?;
        let rows = stmt
            .query_map(params![limit, offset], |row| Ok(Self::row_to_memory(row)))
            .map_err(|e| format!("过滤查询失败: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            let inner = row.map_err(|e| format!("读取行失败: {}", e))?;
            items.push(inner?);
        }

        Ok(MemoryQueryResult {
            has_more: total > (offset as u64 + items.len() as u64),
            items,
            total,
        })
    }

    fn build_filter_clauses(&self, clauses: &mut Vec<String>, query: &MemoryQuery, prefix: &str) {
        if let Some(ref scope) = query.scope {
            clauses.push(format!("{}scope = '{}'", prefix, scope.as_str()));
        }
        if let Some(ref category) = query.category {
            clauses.push(format!(
                "{}category = '{}'",
                prefix,
                category.as_str().replace('\'', "''")
            ));
        }
        if let Some(min_imp) = query.min_importance {
            clauses.push(format!("{}importance >= {}", prefix, min_imp));
        }
        if let Some(ref ws) = query.workspace_name {
            clauses.push(format!(
                "{}workspace_name = '{}'",
                prefix,
                ws.replace('\'', "''")
            ));
        }
        if let Some(ref pp) = query.project_path {
            clauses.push(format!(
                "{}project_path = '{}'",
                prefix,
                pp.replace('\'', "''")
            ));
        }
        if let Some(ref sid) = query.session_id {
            clauses.push(format!(
                "{}session_id = '{}'",
                prefix,
                sid.replace('\'', "''")
            ));
        }
        if let Some(ref from) = query.from_date {
            clauses.push(format!(
                "{}created_at >= '{}'",
                prefix,
                from.replace('\'', "''")
            ));
        }
        if let Some(ref to) = query.to_date {
            clauses.push(format!(
                "{}created_at < '{}'",
                prefix,
                to.replace('\'', "''")
            ));
        }
        if let Some(ref tags) = query.tags {
            for tag in tags {
                let escaped = tag.replace('\'', "''");
                clauses.push(format!(
                    "({}tags LIKE '%\"{}\",%' OR {}tags LIKE '%\"{}\"%')",
                    prefix, escaped, prefix, escaped
                ));
            }
        }
    }

    fn row_to_memory(row: &rusqlite::Row) -> Result<Memory, String> {
        let tags_str: String = row.get::<_, String>(9).unwrap_or_else(|_| "[]".to_string());
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        let scope_str: String = row.get(3).map_err(|e| format!("读取 scope 失败: {}", e))?;
        let category_str: String = row
            .get(4)
            .map_err(|e| format!("读取 category 失败: {}", e))?;
        let is_deleted_int: i32 = row.get(18).unwrap_or(0);

        Ok(Memory {
            id: row.get(0).map_err(|e| format!("读取 id 失败: {}", e))?,
            title: row.get(1).map_err(|e| format!("读取 title 失败: {}", e))?,
            content: row
                .get(2)
                .map_err(|e| format!("读取 content 失败: {}", e))?,
            scope: MemoryScope::parse(&scope_str).unwrap_or(MemoryScope::Project),
            category: MemoryCategory::parse(&category_str),
            importance: row.get(5).unwrap_or(3),
            workspace_name: row.get(6).ok(),
            project_path: row.get(7).ok(),
            session_id: row.get(8).ok(),
            tags,
            source: row.get(10).unwrap_or_else(|_| "user".to_string()),
            created_at: row
                .get(11)
                .map_err(|e| format!("读取 created_at 失败: {}", e))?,
            updated_at: row
                .get(12)
                .map_err(|e| format!("读取 updated_at 失败: {}", e))?,
            accessed_at: row.get(13).unwrap_or_default(),
            access_count: row.get(14).unwrap_or(0),
            user_id: row.get(15).ok(),
            sync_status: row.get(16).unwrap_or_else(|_| "local_only".to_string()),
            sync_version: row.get(17).unwrap_or(0),
            is_deleted: is_deleted_int != 0,
        })
    }
}
