use candid::Nat;
use ic_cdk_macros::{query, update};
use ic_http_certification::{
    HttpRequest,
    HttpResponse,
};

use ic_http_certification::http::StatusCode;

use serde_json::Value;

mod handler;
mod rpc_model;

const MCP_ENDPOINT_PATH: &str = "/mcp";

fn create_ic_response(
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> HttpResponse<'static> {
    HttpResponse::builder()
    .with_status_code(StatusCode::from_u16(status_code).unwrap())
    .with_headers(headers)
    .with_body(body)
    .with_upgrade(false)
    .build()
}

// Tạo header Content-Type: application/json
fn json_content_header() -> Vec<(String, String)> {
    vec![
    ("Content-Type".to_string(), "application/json".to_string()),
    ]
}

// Tạo response lỗi JSON-RPC chuẩn cho IC
fn create_json_rpc_error_response(
    status_code: u16, // HTTP status code (thường là 200 cho lỗi RPC, 400 cho parse)
    id: Option<serde_json::Value>,
    code: i32,
    message: String,
) -> HttpResponse<'static> {
    let rpc_response = rpc_model::create_error_response(id, code, message);
    let body_bytes = serde_json::to_vec(&rpc_response).unwrap_or_default();
    create_ic_response(status_code, json_content_header(), body_bytes)
}

// Tạo response thành công JSON-RPC chuẩn cho IC
fn create_json_rpc_success_response(
    id: serde_json::Value,
    result: Value,
) -> HttpResponse<'static> {
    let rpc_response = rpc_model::create_success_response(id, result);
    let body_bytes = serde_json::to_vec(&rpc_response).unwrap_or_default();
    create_ic_response(200, json_content_header(), body_bytes)
}

// --- Canister Entry Points ---

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    ic_cdk::print(format!("Received Query Request: {:?}", req));

    // --- Validation ---
    if req.method() != "POST" || req.url() != MCP_ENDPOINT_PATH {
        // Chỉ chấp nhận POST đến /mcp
        return create_ic_response(
            404, // Hoặc 405 Method Not Allowed
            vec![],
            format!("Not Found or Method Not Allowed. Use POST to {}", MCP_ENDPOINT_PATH).into_bytes(),
        );
    }

    // --- Parse JSON-RPC Request ---
    let parsed_request: Result<rpc_model::JsonRpcRequest, _> = serde_json::from_slice(&req.body());

    match parsed_request {
        Ok(rpc_req) => {
            let request_id = rpc_req.id.clone(); // Clone ID để sử dụng trong response

            if rpc_req.id.is_some() {
                // --- Handle JSON-RPC Request ---
                match rpc_req.method.as_str() {
                    "initialize" | "tools/list" => {
                         ic_cdk::print(format!("Handling Request Method (Query): {}", rpc_req.method));
                        // Các method read-only có thể xử lý ngay trong query
                        match handler::handle_mcp_request(rpc_req) {
                            Ok(result_value) => {
                                create_json_rpc_success_response(request_id.unwrap(), result_value)
                            }
                            Err(rpc_error) => create_json_rpc_error_response(
                                200, // Lỗi RPC vẫn trả về HTTP 200 OK
                                request_id,
                                rpc_error.code,
                                rpc_error.message,
                            ),
                        }
                    }
                    "tools/call" => {
                         ic_cdk::print(format!("Requesting Upgrade for Method: {}", rpc_req.method));
                        // tools/call có thể thay đổi state, yêu cầu upgrade lên update call
                        HttpResponse::builder()
                            .with_status_code(StatusCode::OK)
                            .with_upgrade(true)
                            .build()
                    }
                    _ => {
                         ic_cdk::print(format!("Method Not Found (Query): {}", rpc_req.method));
                        // Method không được hỗ trợ
                        create_json_rpc_error_response(
                            200,
                            request_id,
                            -32601, // Method not found
                            format!("Method not found: {}", rpc_req.method),
                        )
                    }
                }
            } else {
                // --- Handle JSON-RPC Notification ---
                 ic_cdk::print(format!("Handling Notification (Query): {}", rpc_req.method));
                match handler::handle_mcp_notification(rpc_req) {
                    Ok(_) => {
                         // Trả về HTTP 202 Accepted cho notification thành công
                         create_ic_response(202, vec![], vec![])
                    }
                    Err(e) => {
                         // Handler notification không nên trả lỗi, nhưng nếu có thì log
                         ic_cdk::print(format!("Error handling notification (ignored): {}", e));
                         create_ic_response(202, vec![], vec![])
                    }
                }
            }
        }
        Err(e) => {
             ic_cdk::print(format!("JSON Parse Error (Query): {}", e));
            // Lỗi parse JSON
            create_json_rpc_error_response(
                400, // Bad Request
                None,
                -32700, // Parse error
                format!("Parse error: {}", e),
            )
        }
    }
}

#[update]
fn http_request_update(req: HttpRequest) -> HttpResponse {
    ic_cdk::print(format!("Received Update Request: {:?}", req));

    // --- Validation (Update calls chỉ nên đến từ upgrade) ---
     if req.method() != "POST" || req.url() != MCP_ENDPOINT_PATH {
        return create_ic_response(
            400, // Bad Request - Update call không nên đến trực tiếp sai method/url
            vec![],
            "Bad Request: Update call received for invalid method or URL".as_bytes().to_vec(),
        );
    }

    // --- Parse JSON-RPC Request ---
    let parsed_request: Result<rpc_model::JsonRpcRequest, _> = serde_json::from_slice(&req.body());

    match parsed_request {
        Ok(rpc_req) => {
            let request_id = rpc_req.id.clone();

            if rpc_req.id.is_none() {
                 ic_cdk::print("Rejecting Update: Received Notification in update call");
                 // Không nên nhận notification trong update call
                 return create_json_rpc_error_response(
                    400, // Bad Request
                    None,
                    -32600, // Invalid Request
                    "Cannot process notifications in update call".to_string()
                );
            }

             // --- Handle JSON-RPC Request (chỉ method được upgrade) ---
            match rpc_req.method.as_str() {
                "tools/call" => {
                     ic_cdk::print(format!("Handling Request Method (Update): {}", rpc_req.method));
                     match handler::handle_mcp_request(rpc_req) {
                        Ok(result_value) => {
                            create_json_rpc_success_response(request_id.unwrap(), result_value)
                        }
                        Err(rpc_error) => create_json_rpc_error_response(
                            200, // Lỗi RPC vẫn trả về HTTP 200 OK
                            request_id,
                            rpc_error.code,
                            rpc_error.message,
                        ),
                    }
                }
                 _ => {
                    ic_cdk::print(format!("Invalid Method for Update Call: {}", rpc_req.method));
                    // Method không hợp lệ cho update call (chỉ mong đợi những cái đã upgrade)
                    create_json_rpc_error_response(
                        400, // Bad Request
                        request_id,
                        -32601, // Method not found (hoặc Invalid Request)
                        format!("Invalid method for update call: {}", rpc_req.method),
                    )
                 }
            }
        }
        Err(e) => {
             ic_cdk::print(format!("JSON Parse Error (Update): {}", e));
            // Lỗi parse JSON
            create_json_rpc_error_response(
                400, // Bad Request
                None,
                -32700, // Parse error
                format!("Parse error: {}", e),
            )
        }
    }
}

candid::export_service!();