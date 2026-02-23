mod handler;
mod protocol;

use cc_memory::db::MemoryDatabase;
use cc_memory::models::*;
use cc_memory::repository::MemoryRepository;
use cc_memory::service::MemoryService;
use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Write};

use handler::ToolHandler;
use protocol::*;

#[derive(Parser)]
#[command(name = "cc-memory-mcp", about = "MCP Server for CC-Panes Memory")]
struct Cli {
    /// Memory 数据库路径
    #[arg(long)]
    db_path: String,

    /// 项目路径
    #[arg(long)]
    project_path: Option<String>,

    /// 工作空间名称
    #[arg(long)]
    workspace_name: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 搜索 Memory（CLI 模式，输出到 stdout）
    Search {
        /// 搜索关键词
        #[arg(long)]
        query: Option<String>,

        /// 最小重要度
        #[arg(long)]
        min_importance: Option<u8>,

        /// 数量限制
        #[arg(long, default_value = "10")]
        limit: u32,

        /// 输出格式：json / markdown
        #[arg(long, default_value = "markdown")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // 打开数据库
    let db = match MemoryDatabase::new(&cli.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to open database: {}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Some(Commands::Search {
            query,
            min_importance,
            limit,
            format,
        }) => {
            run_cli_search(
                db,
                &cli.project_path,
                &cli.workspace_name,
                query.as_deref(),
                min_importance,
                limit,
                &format,
            );
        }
        None => {
            run_mcp_server(db, &cli.db_path, cli.project_path, cli.workspace_name);
        }
    }
}

/// CLI 模式：直接搜索并输出结果
fn run_cli_search(
    db: MemoryDatabase,
    project_path: &Option<String>,
    workspace_name: &Option<String>,
    query: Option<&str>,
    min_importance: Option<u8>,
    limit: u32,
    format: &str,
) {
    let repo = MemoryRepository::new(db);
    let service = MemoryService::new(repo);

    let mem_query = MemoryQuery {
        search: query.map(String::from),
        scope: None,
        category: None,
        min_importance,
        workspace_name: workspace_name.clone(),
        project_path: project_path.clone(),
        session_id: None,
        tags: None,
        from_date: None,
        to_date: None,
        sort_by: None,
        limit: Some(limit),
        offset: None,
    };

    match service.search(mem_query) {
        Ok(result) => {
            let output = match format {
                "json" => serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {}\"}}", e)),
                _ => format_cli_markdown(&result),
            };
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Search failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// 格式化 CLI 搜索结果为 Markdown
fn format_cli_markdown(result: &MemoryQueryResult) -> String {
    if result.items.is_empty() {
        return "No memories found.".to_string();
    }

    let mut output = format!("# Search Results ({} found)\n\n", result.total);

    for (i, memory) in result.items.iter().enumerate() {
        output.push_str(&format!(
            "## {}. {} [{}]\n\n",
            i + 1,
            memory.title,
            memory.category
        ));
        output.push_str(&format!("- **ID**: {}\n", memory.id));
        output.push_str(&format!("- **Importance**: {}/5\n", memory.importance));
        output.push_str(&format!("- **Scope**: {}\n", memory.scope));
        if !memory.tags.is_empty() {
            output.push_str(&format!("- **Tags**: {}\n", memory.tags.join(", ")));
        }
        output.push_str(&format!("- **Created**: {}\n\n", memory.created_at));
        output.push_str(&memory.content);
        output.push('\n');
        output.push('\n');
    }

    output
}

/// MCP Server 模式：stdin/stdout JSON-RPC 循环
fn run_mcp_server(
    db: MemoryDatabase,
    db_path: &str,
    project_path: Option<String>,
    workspace_name: Option<String>,
) {
    // 创建两个独立的数据库连接：一个给 service（通过 repo），一个给 report generator
    // MemoryService 拥有 repo，而 ReportGenerator 需要借用 repo
    // 所以这里需要两个独立实例指向同一个 db 文件
    let db2 = match MemoryDatabase::new(db_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Warning: Could not open second database connection: {}", e);
            MemoryDatabase::new_memory().unwrap()
        }
    };

    let repo_for_service = MemoryRepository::new(db);
    let repo_for_report = MemoryRepository::new(db2);
    let service = MemoryService::new(repo_for_service);

    let handler = ToolHandler::new(service, repo_for_report, project_path, workspace_name);

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to read stdin: {}", e);
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                write_response(&mut stdout_lock, &resp);
                continue;
            }
        };

        // 通知（无 id）不需要响应
        let is_notification = request.id.is_none();

        let response = handle_mcp_request(&handler, &request);

        if !is_notification {
            if let Some(resp) = response {
                write_response(&mut stdout_lock, &resp);
            }
        }
    }
}

/// 处理单个 MCP 请求
fn handle_mcp_request(handler: &ToolHandler, request: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ServerCapabilities {
                    tools: ToolsCapability {
                        list_changed: false,
                    },
                },
                server_info: ServerInfo {
                    name: "cc-memory-mcp".to_string(),
                    version: "0.1.0".to_string(),
                },
            };
            let value = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            Some(JsonRpcResponse::success(id, value))
        }

        "notifications/initialized" => {
            // 通知，不需要响应
            None
        }

        "tools/list" => {
            let tools = handler.get_tools();
            let value = serde_json::json!({ "tools": tools });
            Some(JsonRpcResponse::success(id, value))
        }

        "tools/call" => {
            let name = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = request
                .params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            let tool_result = handler.handle_tool_call(name, &arguments);
            let value = serde_json::to_value(tool_result).unwrap_or(serde_json::Value::Null);
            Some(JsonRpcResponse::success(id, value))
        }

        _ => Some(JsonRpcResponse::error(
            id,
            -32601,
            format!("Method not found: {}", request.method),
        )),
    }
}

/// 写入 JSON-RPC 响应到 stdout
fn write_response(writer: &mut impl Write, response: &JsonRpcResponse) {
    if let Ok(json) = serde_json::to_string(response) {
        let _ = writeln!(writer, "{}", json);
        let _ = writer.flush();
    }
}
