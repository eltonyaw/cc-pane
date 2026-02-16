mod commands;
pub mod models;
pub mod repository;
pub mod services;
pub mod utils;

use commands::{
    add_launch_history, add_project, clear_launch_history, create_terminal_session,
    enter_fullscreen, exit_fullscreen, get_all_terminal_status,
    get_git_branch, get_git_status, get_project,
    git_clone, git_fetch, git_pull, git_push, git_stash, git_stash_pop, is_fullscreen, kill_terminal,
    list_all_claude_sessions, list_claude_sessions, scan_broken_sessions,
    clean_session_file, clean_all_broken_sessions,
    list_launch_history, list_projects,
    remove_project, resize_terminal, set_decorations, toggle_always_on_top, enter_mini_mode, exit_mini_mode,
    close_window, minimize_window, maximize_window,
    update_project_alias, update_project_name, write_terminal,
    // Local History 命令
    init_project_history, list_file_versions, get_version_content,
    restore_file_version, get_history_config, update_history_config,
    stop_project_history, cleanup_project_history,
    // Local History - Diff
    get_version_diff, get_versions_diff,
    // Local History - 标签
    put_label, list_labels, delete_label, restore_to_label, create_auto_label,
    // Local History - 目录级历史 + 最近更改
    list_directory_changes, get_recent_changes,
    // Local History - 删除文件 + 压缩
    list_deleted_files, compress_history,
    // Local History - 分支感知 + Worktree
    get_current_branch, get_file_branches, list_file_versions_by_branch,
    list_worktree_recent_changes,
    // Hooks 命令
    is_hooks_enabled, enable_hooks, disable_hooks,
    get_workflow, save_workflow, init_ccpanes,
    // Journal 命令
    add_journal_session, get_journal_index, get_recent_journal,
    // Worktree 命令
    is_git_repo, list_worktrees, add_worktree, remove_worktree,
    // Workspace 命令
    list_workspaces, create_workspace, get_workspace, rename_workspace,
    delete_workspace, add_workspace_project, remove_workspace_project,
    update_workspace_alias, update_workspace_project_alias,
    update_workspace_provider, scan_workspace_directory,
    // Settings 命令
    get_settings, update_settings, test_proxy,
    get_data_dir_info, migrate_data_dir,
    // Provider 命令
    list_providers, get_provider, get_default_provider,
    add_provider, update_provider, remove_provider, set_default_provider,
    // Task 命令
    create_task_board, list_task_boards, get_task_board, delete_task_board, rename_task_board,
    create_task, list_tasks, get_task, update_task, delete_task,
    update_task_status, move_task, update_task_session,
    // MCP 配置命令
    list_mcp_servers, get_mcp_server, upsert_mcp_server, remove_mcp_server,
    // Skill 命令
    list_skills, get_skill, save_skill, delete_skill, copy_skill,
};
use repository::{Database, ProjectRepository, HistoryRepository, TaskRepository};
use services::{ProjectService, TerminalService, HistoryService, HooksService, JournalService, WorktreeService, WorkspaceService, SettingsService, ProviderService, NotificationService, LaunchHistoryService, TaskService, McpConfigService, SkillService};
use utils::AppPaths;
use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

// ============ 应用入口 ============

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. 先加载设置，取得 data_dir
    let settings_service = Arc::new(SettingsService::new());
    let data_dir = settings_service.get_settings().general.data_dir;

    // 2. 构造路径管理器
    let app_paths = Arc::new(AppPaths::new(data_dir));

    // 3. 各服务用 app_paths 初始化
    let db = match Database::new(app_paths.database_path()) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            eprintln!("数据库初始化失败: {}，尝试内存数据库降级", e);
            Arc::new(Database::new_fallback().unwrap_or_else(|e2| {
                panic!("数据库初始化完全失败（含降级）: {}", e2);
            }))
        }
    };
    let project_repo = Arc::new(ProjectRepository::new(db.clone()));
    let history_repo = Arc::new(HistoryRepository::new(db.clone()));
    let task_repo = Arc::new(TaskRepository::new(db));
    let launch_history_service = Arc::new(LaunchHistoryService::new(history_repo));
    let task_service = Arc::new(TaskService::new(task_repo));
    let project_service = Arc::new(ProjectService::new(project_repo));
    let history_service = Arc::new(HistoryService::new());
    let hooks_service = Arc::new(HooksService::new());
    let journal_service = Arc::new(JournalService::new(app_paths.workspaces_dir()));
    let worktree_service = Arc::new(WorktreeService::new());
    let workspace_service = Arc::new(WorkspaceService::new(app_paths.workspaces_dir()));
    let provider_service = Arc::new(ProviderService::new(app_paths.providers_path()));
    let notification_service = Arc::new(NotificationService::new());
    let mcp_config_service = Arc::new(McpConfigService::new());
    let skill_service = Arc::new(SkillService::new());
    let terminal_service = Arc::new(TerminalService::new(
        settings_service.clone(),
        provider_service.clone(),
        notification_service.clone(),
        app_paths.clone(),
    ));

    // 保存引用用于退出时清理
    let terminal_cleanup = terminal_service.clone();
    let history_cleanup = history_service.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_paths)
        .manage(project_service)
        .manage(terminal_service)
        .manage(launch_history_service)
        .manage(history_service)
        .manage(hooks_service)
        .manage(journal_service)
        .manage(worktree_service)
        .manage(workspace_service)
        .manage(settings_service)
        .manage(provider_service)
        .manage(notification_service)
        .manage(task_service)
        .manage(mcp_config_service)
        .manage(skill_service)
        .setup(|app| {
            // ---- 系统托盘 ----
            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出程序", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let icon =
                tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("CC-Panes")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键单击托盘图标 → 显示窗口
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // 点击关闭按钮 → 隐藏到托盘（不退出）
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // 项目命令
            list_projects,
            add_project,
            remove_project,
            get_project,
            update_project_name,
            update_project_alias,
            // 终端命令
            create_terminal_session,
            write_terminal,
            resize_terminal,
            kill_terminal,
            get_all_terminal_status,
            // 窗口命令
            close_window,
            minimize_window,
            maximize_window,
            toggle_always_on_top,
            set_decorations,
            enter_fullscreen,
            exit_fullscreen,
            is_fullscreen,
            enter_mini_mode,
            exit_mini_mode,
            // Git 命令
            get_git_branch,
            get_git_status,
            git_clone,
            git_pull,
            git_push,
            git_fetch,
            git_stash,
            git_stash_pop,
            // Claude 会话命令
            list_claude_sessions,
            list_all_claude_sessions,
            scan_broken_sessions,
            clean_session_file,
            clean_all_broken_sessions,
            // 历史命令
            add_launch_history,
            list_launch_history,
            clear_launch_history,
            // Local History 命令
            init_project_history,
            list_file_versions,
            get_version_content,
            restore_file_version,
            get_history_config,
            update_history_config,
            stop_project_history,
            cleanup_project_history,
            // Local History - Diff
            get_version_diff,
            get_versions_diff,
            // Local History - 标签
            put_label,
            list_labels,
            delete_label,
            restore_to_label,
            create_auto_label,
            // Local History - 目录级历史 + 最近更改
            list_directory_changes,
            get_recent_changes,
            // Local History - 删除文件 + 压缩
            list_deleted_files,
            compress_history,
            // Local History - 分支感知 + Worktree
            get_current_branch,
            get_file_branches,
            list_file_versions_by_branch,
            list_worktree_recent_changes,
            // Hooks 命令
            is_hooks_enabled,
            enable_hooks,
            disable_hooks,
            get_workflow,
            save_workflow,
            init_ccpanes,
            // Journal 命令
            add_journal_session,
            get_journal_index,
            get_recent_journal,
            // Worktree 命令
            is_git_repo,
            list_worktrees,
            add_worktree,
            remove_worktree,
            // Workspace 命令
            list_workspaces,
            create_workspace,
            get_workspace,
            rename_workspace,
            delete_workspace,
            add_workspace_project,
            remove_workspace_project,
            update_workspace_alias,
            update_workspace_project_alias,
            update_workspace_provider,
            scan_workspace_directory,
            // Settings 命令
            get_settings,
            update_settings,
            test_proxy,
            get_data_dir_info,
            migrate_data_dir,
            // Provider 命令
            list_providers,
            get_provider,
            get_default_provider,
            add_provider,
            update_provider,
            remove_provider,
            set_default_provider,
            // Task 命令
            create_task_board,
            list_task_boards,
            get_task_board,
            delete_task_board,
            rename_task_board,
            create_task,
            list_tasks,
            get_task,
            update_task,
            delete_task,
            update_task_status,
            move_task,
            update_task_session,
            // MCP 配置命令
            list_mcp_servers,
            get_mcp_server,
            upsert_mcp_server,
            remove_mcp_server,
            // Skill 命令
            list_skills,
            get_skill,
            save_skill,
            delete_skill,
            copy_skill
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                eprintln!("[cleanup] 应用退出，清理资源...");
                terminal_cleanup.cleanup_all();
                history_cleanup.stop_all_watching();
            }
        });
}
