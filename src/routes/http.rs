pub fn get_ws_key(request: String) -> Option<String> {
    let lines: Vec<String> = request.lines().map(|l| l.to_string()).collect();
    for line in &lines {
        if line.starts_with("Sec-WebSocket-Key: ") {
            return Some(line[19..].to_string());
        }
    }
    None
}

pub fn parse_request_body(request: String) -> String {
    let lines: Vec<String> = request.lines().map(|l| l.to_string()).collect();
    let mut collect = false;
    let mut body = String::from("");
    for line in &lines {
        if collect {
            body.push_str(line);
        }
        if line.is_empty() {
            collect = true;
        }
    }
    body
}