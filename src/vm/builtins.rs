// Builtin functions — fetch API and other native functions

use std::collections::HashMap;
use crate::values::Value;

// ============================================================================
// FETCH API
// ============================================================================

/// Response object returned by fetch()
#[derive(Clone, Debug)]
pub struct FetchResponse {
    pub status: u16,
    pub status_text: String,
    pub ok: bool,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl FetchResponse {
    /// Convert to Value (object representation)
    pub fn to_value(&self) -> Value {
        let mut map = HashMap::new();
        map.insert("status".to_string(), Value::Number(self.status as f64));
        map.insert("statusText".to_string(), Value::Str(self.status_text.clone()));
        map.insert("ok".to_string(), Value::Bool(self.ok));
        map.insert("_body".to_string(), Value::Str(self.body.clone()));
        
        // Headers as object
        let mut headers_map = HashMap::new();
        for (k, v) in &self.headers {
            headers_map.insert(k.clone(), Value::Str(v.clone()));
        }
        map.insert("headers".to_string(), Value::Object(headers_map));
        
        Value::Object(map)
    }

    /// Parse body as JSON
    pub fn json(&self) -> Result<Value, String> {
        self.parse_json_value(&self.body)
    }

    /// Get body as text
    pub fn text(&self) -> String {
        self.body.clone()
    }

    fn parse_json_value(&self, s: &str) -> Result<Value, String> {
        let trimmed = s.trim();
        if trimmed.starts_with('{') {
            self.parse_json_object(trimmed)
        } else if trimmed.starts_with('[') {
            self.parse_json_array(trimmed)
        } else if trimmed.starts_with('"') {
            Ok(Value::Str(trimmed.trim_matches('"').to_string()))
        } else if trimmed == "true" {
            Ok(Value::Bool(true))
        } else if trimmed == "false" {
            Ok(Value::Bool(false))
        } else if trimmed == "null" {
            Ok(Value::Number(0.0))
        } else if let Ok(n) = trimmed.parse::<f64>() {
            Ok(Value::Number(n))
        } else {
            Err(format!("Invalid JSON: {}", s))
        }
    }

    fn parse_json_object(&self, s: &str) -> Result<Value, String> {
        match serde_json::from_str::<serde_json::Value>(s) {
            Ok(json) => Ok(self.json_to_value(&json)),
            Err(e) => Err(format!("JSON parse error: {}", e)),
        }
    }

    fn parse_json_array(&self, s: &str) -> Result<Value, String> {
        match serde_json::from_str::<serde_json::Value>(s) {
            Ok(json) => Ok(self.json_to_value(&json)),
            Err(e) => Err(format!("JSON parse error: {}", e)),
        }
    }

    fn json_to_value(&self, json: &serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Number(0.0),
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => {
                Value::Number(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::String(s) => Value::Str(s.clone()),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.json_to_value(v)).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), self.json_to_value(v));
                }
                Value::Object(map)
            }
        }
    }
}

/// Execute a fetch request (blocking)
pub fn fetch(url: &str, method: &str, headers: &HashMap<String, String>, body: Option<&str>) -> Result<FetchResponse, String> {
    let mut req = ureq::AgentBuilder::new().build().request(method, url);
    
    // Add headers
    for (key, value) in headers {
        req = req.set(key, value);
    }
    
    // Add Content-Type if body is present
    if body.is_some() && !headers.contains_key("Content-Type") {
        req = req.set("Content-Type", "application/json");
    }

    let response = if let Some(b) = body {
        req.send_string(b)
    } else {
        req.call()
    };

    match response {
        Ok(resp) => {
            let status = resp.status();
            let status_text = resp.status_text().to_string();
            let ok = status >= 200 && status < 300;
            
            // Read headers
            let mut resp_headers = HashMap::new();
            for name in resp.headers_names() {
                if let Some(val) = resp.header(&name) {
                    resp_headers.insert(name, val.to_string());
                }
            }
            
            // Read body
            let body_text = resp.into_string().unwrap_or_default();
            
            Ok(FetchResponse {
                status,
                status_text,
                ok,
                body: body_text,
                headers: resp_headers,
            })
        }
        Err(ureq::Error::Status(code, resp)) => {
            // HTTP error status
            let body_text = resp.into_string().unwrap_or_default();
            Ok(FetchResponse {
                status: code,
                status_text: format!("HTTP {}", code),
                ok: false,
                body: body_text,
                headers: HashMap::new(),
            })
        }
        Err(e) => Err(format!("Fetch error: {}", e)),
    }
}

// ============================================================================
// HTTP MODULE (Node.js-style API)
// ============================================================================

/// http.get(url, callback) — GET request, callback receives response
/// Returns a Promise that resolves with the callback result
pub fn http_get(url: &str) -> Result<FetchResponse, String> {
    fetch(url, "GET", &HashMap::new(), None)
}

/// http.post(url, body) — POST request with JSON body
pub fn http_post(url: &str, body: &str) -> Result<FetchResponse, String> {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    fetch(url, "POST", &headers, Some(body))
}

/// http.put(url, body) — PUT request with JSON body
pub fn http_put(url: &str, body: &str) -> Result<FetchResponse, String> {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    fetch(url, "PUT", &headers, Some(body))
}

/// http.delete(url) — DELETE request
pub fn http_delete(url: &str) -> Result<FetchResponse, String> {
    fetch(url, "DELETE", &HashMap::new(), None)
}

/// http.patch(url, body) — PATCH request with JSON body
pub fn http_patch(url: &str, body: &str) -> Result<FetchResponse, String> {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    fetch(url, "PATCH", &headers, Some(body))
}

/// http.request(url, method, headers, body) — universal request
pub fn http_request(url: &str, method: &str, headers: &HashMap<String, String>, body: Option<&str>) -> Result<FetchResponse, String> {
    fetch(url, method, headers, body)
}

// ============================================================================
// HTTP SERVER
// ============================================================================

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

/// Parsed incoming request
#[derive(Clone, Debug)]
pub struct ServerRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Server handle — wraps a TcpListener for external control
pub struct ServerHandle {
    pub port: u16,
}

/// Parse an HTTP request from a stream
fn parse_request(mut reader: impl BufRead) -> Option<ServerRequest> {
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return None,
            Ok(_) => {
                let trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
                if trimmed.is_empty() { break; }
                lines.push(trimmed.to_string());
            }
            Err(_) => return None,
        }
    }

    if lines.is_empty() { return None; }

    // Parse request line
    let parts: Vec<&str> = lines[0].split_whitespace().collect();
    let method = parts.first().unwrap_or(&"GET").to_string();
    let url = parts.get(1).unwrap_or(&"/").to_string();

    // Parse headers
    let mut headers = HashMap::new();
    let mut content_length: usize = 0;
    for line in &lines[1..] {
        if let Some((key, val)) = line.split_once(": ") {
            if key.to_lowercase() == "content-length" {
                content_length = val.parse().unwrap_or(0);
            }
            headers.insert(key.to_string(), val.to_string());
        }
    }

    // Read body
    let mut body = String::new();
    if content_length > 0 {
        let mut buf = vec![0u8; content_length];
        if reader.read_exact(&mut buf).is_ok() {
            body = String::from_utf8_lossy(&buf).to_string();
        }
    }

    Some(ServerRequest { method, url, headers, body })
}

/// Convert ServerRequest to a Value (Object)
pub fn req_to_value(req: &ServerRequest) -> crate::values::Value {
    use crate::values::Value;
    let mut map = HashMap::new();
    map.insert("method".to_string(), Value::Str(req.method.clone()));
    map.insert("url".to_string(), Value::Str(req.url.clone()));
    map.insert("body".to_string(), Value::Str(req.body.clone()));

    let mut hmap = HashMap::new();
    for (k, v) in &req.headers {
        hmap.insert(k.clone(), Value::Str(v.clone()));
    }
    map.insert("headers".to_string(), Value::Object(hmap));

    Value::Object(map)
}

/// Send an HTTP response
fn send_response(mut stream: std::net::TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, status_text, content_type, body.len(), body
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// Start a server on a port, handle requests synchronously.
/// For each request, calls the handler closure and sends the response.
pub fn start_server<F>(port: u16, handler: F) -> Result<(), String>
where
    F: Fn(&ServerRequest) -> (u16, String, String), // returns (status, content_type, body)
{
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("Cannot bind to port {}: {}", port, e))?;

    eprintln!("Server running on http://127.0.0.1:{}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let reader = BufReader::new(&stream);
                if let Some(req) = parse_request(reader) {
                    let (status, content_type, body) = handler(&req);
                    send_response(stream.try_clone().unwrap(), status, &content_type, &body);
                }
            }
            Err(_) => continue,
        }
    }

    Ok(())
}
