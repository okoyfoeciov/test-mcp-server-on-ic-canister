use ic_cdk::eprintln;
use ic_http_certification::http::StatusCode;
use ic_http_certification::{HttpRequest, HttpResponse};
pub struct Server {
    // fields
}

#[derive(serde::Serialize, serde::Deserialize)]
struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

impl Server {
    pub fn handle(&self, req: &HttpRequest) -> HttpResponse {
        if req.method() != "POST" || req.url() != "/mcp" {
            return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(404).unwrap())
                .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
                .build();
        }

        match serde_json::from_slice::<JsonRpcRequest>(&req.body()) {
            Ok(parsed_request) => {
                if parsed_request.method != "2.0" {
                    return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(400).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}}"#)
                .build();
                }

                match parsed_request.id {
                    Some(request_id) => {
                        return self.handle_request(
                            request_id,
                            parsed_request.method.as_str(),
                            parsed_request.params,
                        )
                    }
                    None => return self.handle_notification(parsed_request.params),
                }
            }
            Err(error) => {
                eprintln!("Error: {}", error);
                return HttpResponse::builder()
                .with_status_code(StatusCode::from_u16(400).unwrap())
                .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
                .with_body(br#"{"jsonrpc": "2.0", "error": {"code": -32700, "message": "Parse error"}}"#)
                .build();
            }
        }
    }

    fn handle_tools_call(
        &self,
        _request_id: serde_json::Value,
        params: Option<serde_json::Value>,
    ) -> ic_http_certification::HttpResponse {
        return ic_http_certification::HttpResponse::builder()
            .with_status_code(ic_http_certification::http::StatusCode::from_u16(404).unwrap())
            .with_body(b"Not Found or Method Not Allowed. Use POST to /mcp")
            .build();
    }

    fn handle_tools_list(
        &self,
        _request_id: serde_json::Value,
        _params: Option<serde_json::Value>,
    ) -> ic_http_certification::HttpResponse {
        return ic_http_certification::HttpResponse::builder()
            .with_status_code(ic_http_certification::http::StatusCode::from_u16(404).unwrap())
            .with_body(br#"{"jsonrpc": "2.0", "id": 1, "result": {"tools": [{"description": "Adds two numbers (a and b)", "inputSchema": {"properties": {"a": {"description": "The first number", "type": "number"}, "b": {"description": "The second number", "type": "number"}}, "required": ["a", "b"], "type": "object"}, "name": "add"}]}}"#)
            .build();
    }

    fn handle_request(
        &self,
        request_id: serde_json::Value,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> ic_http_certification::HttpResponse {
        match method {
            "initialize" => {
                return self.handle_initialize(request_id,params)
            },
            "tools/list" => {
                return self.handle_tools_list(request_id,params)
            },
            "tools/call" => {
                return self.handle_tools_call(request_id,params)
            },
            _ =>  {HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(200).unwrap())
            .with_headers(vec![("Content-Type".to_string(), "application/json".to_string())])
            .with_body(br#"{"jsonrpc": "2.0",id: "..." "error": {"code": -32601, "message": "Method not found"}}"#)
            .build()}
        }
    }

    fn handle_initialize(
        &self,
        request_id: serde_json::Value,
        _params: Option<serde_json::Value>,
    ) -> ic_http_certification::HttpResponse {
        return ic_http_certification::HttpResponse::builder()
            .with_status_code(ic_http_certification::http::StatusCode::from_u16(404).unwrap())
            .with_body(format!(
                "{{\"jsonrpc\": \"2.0\", \"id\": {}, \"result\": {{\"capabilities\": {{}}, \"instructions\": \"Welcome to the minimal MCP server!\", \"protocolVersion\": \"2025-03-26\", \"serverInfo\": {{\"name\": \"MinimalRustMCPServer\", \"version\": \"0.1.0\"}} }}}}",
                request_id.to_string()
            ).into_bytes())
            .build();
    }

    fn handle_notification(
        &self,
        _params: Option<serde_json::Value>,
    ) -> ic_http_certification::HttpResponse {
        return HttpResponse::builder()
            .with_status_code(StatusCode::from_u16(202).unwrap())
            .build();
    }
}
