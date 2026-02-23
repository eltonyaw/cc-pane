use crate::utils::error_codes as EC;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Skill 信息（对应 `.claude/commands/` 下的 `.md` 文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    /// 文件名（不含 .md 扩展名），即 `/命令名`
    pub name: String,
    /// Markdown 内容
    pub content: String,
    /// 文件完整路径
    pub file_path: String,
}

/// Skill 摘要（不含完整内容，用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSummary {
    pub name: String,
    /// 内容的前 200 字符作为摘要
    pub preview: String,
    pub file_path: String,
}

/// Skill 管理服务 — 操作项目目录下的 `.claude/commands/` 目录
#[derive(Default)]
pub struct SkillService;

impl SkillService {
    pub fn new() -> Self {
        Self
    }

    /// 获取 commands 目录路径
    fn commands_dir(project_path: &str) -> PathBuf {
        Path::new(project_path).join(".claude").join("commands")
    }

    /// 列出项目的所有 Skill（摘要）
    pub fn list_skills(&self, project_path: &str) -> Result<Vec<SkillSummary>, String> {
        let dir = Self::commands_dir(project_path);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut skills = Vec::new();
        let entries =
            std::fs::read_dir(&dir).map_err(|e| format!("Failed to read commands directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read skill file: {}", e))?;
                let preview = if content.len() > 200 {
                    format!("{}...", &content[..200])
                } else {
                    content
                };
                skills.push(SkillSummary {
                    name,
                    preview,
                    file_path: path.to_string_lossy().to_string(),
                });
            }
        }

        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(skills)
    }

    /// 获取单个 Skill 的完整内容
    pub fn get_skill(&self, project_path: &str, name: &str) -> Result<Option<SkillInfo>, String> {
        let path = Self::commands_dir(project_path).join(format!("{}.md", name));
        if !path.exists() {
            return Ok(None);
        }
        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("Failed to read skill file: {}", e))?;
        Ok(Some(SkillInfo {
            name: name.to_string(),
            content,
            file_path: path.to_string_lossy().to_string(),
        }))
    }

    /// 创建或更新 Skill
    pub fn save_skill(
        &self,
        project_path: &str,
        name: &str,
        content: &str,
    ) -> Result<SkillInfo, String> {
        // Validate name
        if name.trim().is_empty() {
            return Err(serde_json::json!({"code": EC::SKILL_NAME_EMPTY, "message": "Skill name cannot be empty"}).to_string());
        }
        // Validate name does not contain path separators
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(serde_json::json!({"code": EC::SKILL_NAME_PATH_SEPARATOR, "message": "Skill name cannot contain path separators"}).to_string());
        }
        // Reject hidden file names starting with '.'
        if name.starts_with('.') {
            return Err(serde_json::json!({"code": EC::SKILL_NAME_DOT_PREFIX, "message": "Skill name cannot start with '.'"}).to_string());
        }

        let dir = Self::commands_dir(project_path);
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create commands directory: {}", e))?;

        let path = dir.join(format!("{}.md", name));
        std::fs::write(&path, content).map_err(|e| format!("Failed to write skill file: {}", e))?;

        Ok(SkillInfo {
            name: name.to_string(),
            content: content.to_string(),
            file_path: path.to_string_lossy().to_string(),
        })
    }

    /// 删除 Skill
    pub fn delete_skill(&self, project_path: &str, name: &str) -> Result<bool, String> {
        let path = Self::commands_dir(project_path).join(format!("{}.md", name));
        if !path.exists() {
            return Ok(false);
        }
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete skill file: {}", e))?;
        Ok(true)
    }

    /// 跨项目复制 Skill
    pub fn copy_skill(
        &self,
        source_project: &str,
        target_project: &str,
        name: &str,
    ) -> Result<SkillInfo, String> {
        let skill = self
            .get_skill(source_project, name)?
            .ok_or_else(|| format!("Skill not found in source project: {}", name))?;
        self.save_skill(target_project, name, &skill.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup() -> (SkillService, PathBuf) {
        let temp =
            std::env::temp_dir().join(format!("cc-panes-skill-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp).unwrap();
        (SkillService::new(), temp)
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_list_empty() {
        let (svc, tmp) = setup();
        let path = tmp.to_str().unwrap();
        let result = svc.list_skills(path).unwrap();
        assert!(result.is_empty());
        cleanup(&tmp);
    }

    #[test]
    fn test_save_and_get() {
        let (svc, tmp) = setup();
        let path = tmp.to_str().unwrap();

        let saved = svc
            .save_skill(
                path,
                "create-component",
                "# 创建组件\n\n请帮我创建一个 React 组件...",
            )
            .unwrap();
        assert_eq!(saved.name, "create-component");
        assert!(saved.content.contains("创建组件"));

        let found = svc.get_skill(path, "create-component").unwrap();
        assert!(found.is_some());
        assert_eq!(
            found.unwrap().content,
            "# 创建组件\n\n请帮我创建一个 React 组件..."
        );
        cleanup(&tmp);
    }

    #[test]
    fn test_list_skills() {
        let (svc, tmp) = setup();
        let path = tmp.to_str().unwrap();

        svc.save_skill(path, "skill-b", "B content").unwrap();
        svc.save_skill(path, "skill-a", "A content").unwrap();

        let list = svc.list_skills(path).unwrap();
        assert_eq!(list.len(), 2);
        // 应该按名称排序
        assert_eq!(list[0].name, "skill-a");
        assert_eq!(list[1].name, "skill-b");
        cleanup(&tmp);
    }

    #[test]
    fn test_delete() {
        let (svc, tmp) = setup();
        let path = tmp.to_str().unwrap();

        svc.save_skill(path, "to-delete", "content").unwrap();
        assert!(svc.delete_skill(path, "to-delete").unwrap());
        assert!(!svc.delete_skill(path, "to-delete").unwrap());
        assert!(svc.get_skill(path, "to-delete").unwrap().is_none());
        cleanup(&tmp);
    }

    #[test]
    fn test_copy_skill() {
        let (svc, tmp) = setup();
        let src = tmp.join("project-a");
        let dst = tmp.join("project-b");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();

        svc.save_skill(
            src.to_str().unwrap(),
            "shared-skill",
            "# Shared\n\nContent here",
        )
        .unwrap();
        let copied = svc
            .copy_skill(src.to_str().unwrap(), dst.to_str().unwrap(), "shared-skill")
            .unwrap();

        assert_eq!(copied.name, "shared-skill");
        assert!(copied.content.contains("Shared"));

        let found = svc
            .get_skill(dst.to_str().unwrap(), "shared-skill")
            .unwrap();
        assert!(found.is_some());
        cleanup(&tmp);
    }

    #[test]
    fn test_invalid_name() {
        let (svc, tmp) = setup();
        let path = tmp.to_str().unwrap();

        assert!(svc.save_skill(path, "", "content").is_err());
        assert!(svc.save_skill(path, "../escape", "content").is_err());
        assert!(svc.save_skill(path, "path/traversal", "content").is_err());
        assert!(svc.save_skill(path, "back\\slash", "content").is_err());
        assert!(svc.save_skill(path, ".hidden", "content").is_err());
        assert!(svc.save_skill(path, ".env", "content").is_err());
        cleanup(&tmp);
    }

    #[test]
    fn test_update_existing() {
        let (svc, tmp) = setup();
        let path = tmp.to_str().unwrap();

        svc.save_skill(path, "my-skill", "version 1").unwrap();
        svc.save_skill(path, "my-skill", "version 2").unwrap();

        let found = svc.get_skill(path, "my-skill").unwrap().unwrap();
        assert_eq!(found.content, "version 2");
        cleanup(&tmp);
    }
}
