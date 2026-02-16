use cc_panes_lib::repository::{Database, ProjectRepository};
use std::sync::Arc;

/// 创建基于临时文件的测试数据库和仓储
/// 使用随机文件名确保测试隔离
pub fn create_test_db() -> (Arc<Database>, Arc<ProjectRepository>) {
    let temp_dir = std::env::temp_dir().join("cc-panes-test");
    std::fs::create_dir_all(&temp_dir).expect("创建临时目录失败");
    let db_path = temp_dir.join(format!("test-{}.db", uuid::Uuid::new_v4()));
    let db = Arc::new(Database::new(db_path).expect("创建测试数据库失败"));
    let repo = Arc::new(ProjectRepository::new(db.clone()));
    (db, repo)
}
