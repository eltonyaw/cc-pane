use serde::Deserialize;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Hook input from Claude Code (subset of fields we care about).
#[derive(Debug, Deserialize)]
struct HookInput {
    hook_event_name: Option<String>,
    tool_name: Option<String>,
    tool_input: Option<ToolInput>,
}

#[derive(Debug, Deserialize)]
struct ToolInput {
    file_path: Option<String>,
}

/// PostToolUse hook entry point.
/// Reads hook JSON from stdin and archives plan files to `.ccpanes/plans/`.
pub fn run() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let hook: HookInput = match serde_json::from_str(&input) {
        Ok(h) => h,
        Err(_) => return,
    };

    // Verify this is the event we care about
    let event = hook.hook_event_name.as_deref().unwrap_or_default();
    let tool = hook.tool_name.as_deref().unwrap_or_default();
    if event != "PostToolUse" || tool != "Write" {
        return;
    }

    let file_path = match hook.tool_input.and_then(|t| t.file_path) {
        Some(p) => p,
        None => return,
    };

    // Check if the file is in ~/.claude/plans/
    let plans_dir = match get_claude_plans_dir() {
        Some(d) => d,
        None => return,
    };

    let file_path_buf = PathBuf::from(&file_path);
    // Normalize for comparison
    let canonical_file = file_path_buf.canonicalize().unwrap_or(file_path_buf.clone());
    let canonical_plans = plans_dir.canonicalize().unwrap_or(plans_dir.clone());

    if !canonical_file.starts_with(&canonical_plans) {
        return;
    }

    // Get project directory
    let project_dir = match std::env::var("CLAUDE_PROJECT_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => return,
    };

    // Create .ccpanes/plans/ if needed
    let target_dir = project_dir.join(".ccpanes").join("plans");
    if let Err(e) = fs::create_dir_all(&target_dir) {
        eprintln!("[ccpanes] Failed to create plans directory: {}", e);
        return;
    }

    // Copy the plan file
    let file_name = match Path::new(&file_path).file_name() {
        Some(n) => n,
        None => return,
    };
    let target_path = target_dir.join(file_name);

    match fs::copy(&file_path, &target_path) {
        Ok(_) => {
            eprintln!(
                "[ccpanes] Plan archived: {} -> {}",
                file_path,
                target_path.display()
            );
        }
        Err(e) => {
            eprintln!("[ccpanes] Failed to archive plan: {}", e);
        }
    }
}

fn get_claude_plans_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let plans_dir = home.join(".claude").join("plans");
    if plans_dir.is_dir() {
        Some(plans_dir)
    } else {
        // Return the path even if it doesn't exist yet;
        // the file comparison will handle it
        Some(plans_dir)
    }
}
