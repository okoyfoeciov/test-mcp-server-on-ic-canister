use crate::rpc_model::*;
use serde_json::{json, Value};

const PROTOCOL_VERSION: &str = "2025-03-26";
const SERVER_NAME: &str = "MinimalRustMCPServer";
const SERVER_VERSION: &str = "0.1.0";

pub fn handle_mcp_request(req: JsonRpcRequest) -> Result<Value, JsonRpcError> {
    match req.method.as_str() {
        "initialize" => handle_initialize(req.params).map(|res| json!(res)),
        "tools/list" => handle_tools_list().map(|res| json!(res)),
        "tools/call" => handle_tools_call(req.params).map(|res| json!(res)),
        _ => Err(JsonRpcError {
            code: -32601, // Method not found
            message: format!("Method not found: {}", req.method),
            data: None,
        }),
    }
}

pub fn handle_initialize(params: Option<Value>) -> Result<InitializeResult, JsonRpcError> {
    println!("Handling initialize request...");
    // Parse params (không thực sự sử dụng trong ví dụ minimal này, nhưng nên có)
    let _init_params: InitializeParams = params
        .map(|p| serde_json::from_value(p))
        .transpose()
        .map_err(|e| JsonRpcError {
            code: -32602, // Invalid params
            message: format!("Invalid initialize params: {}", e),
            data: None,
        })?
        .ok_or_else(|| JsonRpcError {
            code: -32602, // Invalid params
            message: "Missing initialize params".to_string(),
            data: None,
        })?;

    // TODO: Kiểm tra protocolVersion của client nếu cần thiết

    println!("Client capabilities received (ignored in minimal server)");

    Ok(InitializeResult {
        protocol_version: PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolCapability{}), // Báo là có support tool
            // Các capabilities khác mặc định là None/Default
            ..Default::default()
        },
        server_info: ServerInfo {
            name: SERVER_NAME.to_string(),
            version: SERVER_VERSION.to_string(),
        },
        instructions: Some("Welcome to the minimal MCP server!".to_string()),
    })
}

pub fn handle_tools_list() -> Result<ToolsListResult, JsonRpcError> {
     println!("Handling tools/list request...");
    let add_tool = Tool {
        name: "add".to_string(),
        description: "Adds two numbers (a and b)".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "a": { "type": "number", "description": "The first number" },
                "b": { "type": "number", "description": "The second number" }
            },
            "required": ["a", "b"]
        }),
        annotations: None,
    };

    Ok(ToolsListResult {
        tools: vec![add_tool],
        next_cursor: None, // Không support pagination
    })
}


pub fn handle_tools_call(params: Option<Value>) -> Result<CallResult, JsonRpcError> {
    println!("Handling tools/call request...");
    let call_params: CallParams = params
        .map(|p| serde_json::from_value(p))
        .transpose()
        .map_err(|e| JsonRpcError {
            code: -32602, // Invalid params
            message: format!("Invalid tools/call params: {}", e),
            data: None,
        })?
        .ok_or_else(|| JsonRpcError {
            code: -32602, // Invalid params
            message: "Missing tools/call params".to_string(),
            data: None,
        })?;

    if call_params.name != "add" {
        return Err(JsonRpcError {
            code: -32602, // Invalid params (hoặc có thể là -32002 Resource not found nếu coi tool là resource)
            message: format!("Unknown tool name: {}", call_params.name),
            data: None,
        });
    }

    let a_val = call_params.arguments.get("a")
        .ok_or_else(|| JsonRpcError { code: -32602, message: "Missing argument 'a'".to_string(), data: None })?;
    let b_val = call_params.arguments.get("b")
        .ok_or_else(|| JsonRpcError { code: -32602, message: "Missing argument 'b'".to_string(), data: None })?;

    let a = a_val.as_f64()
        .ok_or_else(|| JsonRpcError { code: -32602, message: "Argument 'a' must be a number".to_string(), data: None })?;
    let b = b_val.as_f64()
        .ok_or_else(|| JsonRpcError { code: -32602, message: "Argument 'b' must be a number".to_string(), data: None })?;

    let sum = a + b;

    Ok(CallResult {
        content: vec![ToolResultContent::Text {
            text: format!("The sum of {} and {} is {}", a, b, sum),
        }],
        is_error: false,
    })
}

// Hàm helper để xử lý notification
pub fn handle_mcp_notification(req: JsonRpcRequest) -> Result<(), String> {
     match req.method.as_str() {
        "notifications/initialized" => {
            println!("Received initialized notification from client. Ready for full operation.");
            Ok(()) // Không làm gì cả, chỉ log
        }
        "notifications/cancelled" => {
             println!("Received cancellation notification (ignored): {:?}", req.params);
             Ok(())
        }
        _ => {
            println!("Received unknown notification: {}", req.method);
            // Không trả về lỗi cho notification không xác định
             Ok(())
        }
    }
}