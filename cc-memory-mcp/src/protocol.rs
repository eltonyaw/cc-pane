use serde::{Deserialize, Serialize};

// JSON-RPC 2.0 请求
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

// JSON-RPC 2.0 响应
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// MCP Protocol Types
#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    pub tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Debug, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

// 辅助构建方法
impl JsonRpcResponse {
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

impl ToolResult {
    pub fn text(text: String) -> Self {
        Self {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: None,
        }
    }

    pub fn error(text: String) -> Self {
        Self {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: Some(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_response_success() {
        let resp = JsonRpcResponse::success(
            Some(serde_json::Value::Number(1.into())),
            serde_json::json!({"ok": true}),
        );
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert_eq!(resp.jsonrpc, "2.0");
    }

    #[test]
    fn test_json_rpc_response_error() {
        let resp = JsonRpcResponse::error(
            Some(serde_json::Value::Number(1.into())),
            -32600,
            "Invalid Request".to_string(),
        );
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32600);
        assert_eq!(err.message, "Invalid Request");
    }

    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("hello".to_string());
        assert!(result.is_error.is_none());
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0].content_type, "text");
        assert_eq!(result.content[0].text, "hello");
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("something went wrong".to_string());
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.content[0].text, "something went wrong");
    }

    #[test]
    fn test_deserialize_json_rpc_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "initialize");
        assert!(req.id.is_some());
    }

    #[test]
    fn test_deserialize_json_rpc_request_no_params() {
        let json = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.method, "tools/list");
        assert!(req.params.is_null());
    }

    #[test]
    fn test_serialize_initialize_result() {
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
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("protocolVersion"));
        assert!(json.contains("2024-11-05"));
        assert!(json.contains("listChanged"));
    }
}
