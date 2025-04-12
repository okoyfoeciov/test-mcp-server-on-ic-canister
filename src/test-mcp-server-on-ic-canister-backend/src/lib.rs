use ic_cdk_macros::{query, update};
use ic_http_certification::{HttpRequest, HttpResponse};

use ic_http_certification::http::StatusCode;

use serde_json::Value;

mod handler;
mod rpc_model;

const MCP_ENDPOINT_PATH: &str = "/mcp";

fn create_ic_response<'a>(
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> HttpResponse<'a> {
    HttpResponse::builder()
        .with_status_code(StatusCode::from_u16(status_code).unwrap())
        .with_headers(headers)
        .with_body(body)
        .with_upgrade(false)
        .build()
}

fn json_content_header() -> Vec<(String, String)> {
    vec![("Content-Type".to_string(), "application/json".to_string())]
}

fn create_json_rpc_error_response<'a>(
    status_code: u16,
    id: Option<serde_json::Value>,
    code: i32,
    message: String,
) -> HttpResponse<'a> {
    let rpc_response = rpc_model::create_error_response(id, code, message);
    let body_bytes = serde_json::to_vec(&rpc_response).unwrap_or_default();
    create_ic_response(status_code, json_content_header(), body_bytes)
}

fn create_json_rpc_success_response<'a>(id: serde_json::Value, result: Value) -> HttpResponse<'a> {
    let rpc_response = rpc_model::create_success_response(id, result);
    let body_bytes = serde_json::to_vec(&rpc_response).unwrap_or_default();
    create_ic_response(200, json_content_header(), body_bytes)
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    if req.method() != "POST" || req.url() != MCP_ENDPOINT_PATH {
        return create_ic_response(
            404,
            vec![],
            format!(
                "Not Found or Method Not Allowed. Use POST to {}",
                MCP_ENDPOINT_PATH
            )
            .into_bytes(),
        );
    }

    HttpResponse::builder()
        .with_status_code(StatusCode::OK)
        .with_upgrade(true)
        .build()
}

#[update]
fn http_request_update(req: HttpRequest) -> HttpResponse {
    if req.method() != "POST" || req.url() != MCP_ENDPOINT_PATH {
        return create_ic_response(
            404,
            vec![],
            format!(
                "Not Found or Method Not Allowed. Use POST to {}",
                MCP_ENDPOINT_PATH
            )
            .into_bytes(),
        );
    }

    let parsed_request: Result<rpc_model::JsonRpcRequest, _> = serde_json::from_slice(&req.body());

    match parsed_request {
        Ok(rpc_req) => {
            let request_id = rpc_req.id.clone();

            if rpc_req.id.is_none() {
                return create_ic_response(202, vec![], vec![]);
            }

            match rpc_req.method.as_str() {
                "initialize" | "tools/list" | "tools/call" => {
                    match handler::handle_mcp_request(rpc_req) {
                        Ok(result_value) => {
                            create_json_rpc_success_response(request_id.unwrap(), result_value)
                        }
                        Err(rpc_error) => create_json_rpc_error_response(
                            200,
                            request_id,
                            rpc_error.code,
                            rpc_error.message,
                        ),
                    }
                }
                _ => create_json_rpc_error_response(
                    200,
                    request_id,
                    -32601,
                    format!("Method not found"),
                ),
            }
        }
        Err(e) => create_json_rpc_error_response(400, None, -32700, format!("Parse error: {}", e)),
    }
}

candid::export_service!();
