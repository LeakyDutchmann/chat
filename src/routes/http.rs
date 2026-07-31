use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    pub status: String,
    pub headers: Vec<String>,
    pub body: String,
}

impl Response {
    
}