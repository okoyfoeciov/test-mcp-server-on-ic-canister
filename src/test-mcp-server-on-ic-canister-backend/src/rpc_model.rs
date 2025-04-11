use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// --- Generic JSON-RPC Structures ---

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>, // ID có thể là string hoặc number, hoặc null cho notification
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value, // ID phải khớp với request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// --- MCP Specific Structures ---

// Initialize
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: Option<ClientInfo>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    // Thêm các client capabilities khác nếu cần
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapability>,
    // Không có prompts, resources, logging... trong ví dụ minimal này
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolCapability {
    // Không có listChanged vì chúng ta không support nó
}


#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

// Tools List (Trả về danh sách tool, dù chỉ có 1)
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolsListResult {
    pub tools: Vec<Tool>,
     #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>, // Luôn là None vì không hỗ trợ pagination
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value, // Sử dụng Value cho JSON Schema đơn giản
     #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Value>, // Thêm nếu cần, nhưng không bắt buộc cho ví dụ minimal
}

// Tool Call
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallParams {
    pub name: String,
    pub arguments: HashMap<String, Value>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallResult {
    pub content: Vec<ToolResultContent>,
    pub is_error: bool,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolResultContent {
    Text { text: String },
    // Image, Audio, Resource không được support trong ví dụ này
}

// --- Convenience Functions ---
pub fn create_error_response(id: Option<serde_json::Value>, code: i32, message: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        // ID là null nếu request là notification hoặc request ID không parse được
        id: id.unwrap_or(serde_json::Value::Null),
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data: None,
        }),
    }
}

pub fn create_success_response(id: serde_json::Value, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}