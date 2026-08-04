# Chat 

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Tokio](https://img.shields.io/badge/Tokio-1.53-red)
![SQLx](https://img.shields.io/badge/SQLx-0.9-blue)
![Serde](https://img.shields.io/badge/Serde-1.0-purple)

A fully async framework-free web chat built from scratch in Rust -- including  
manual HTTP parsing, WebSocket implementation, Cookies and SQLx-powered persistence.


## Features
- User registration with Argon2 password hashing
- Login/logout endpoints
- Session-based authentication using cookies
- Database integration via Sqlx
- WebSocket chat support
- Session verification via /me endpoint
- Clean service-layer architecture
- Pure async framework
- Manual parsing and handling of HTTP requests
- Sha1-hashed cookies
- Frontend session restoration
- Session cleanup on server shutdown
- Manually implemented simple fileserver
## Tech Stack
### Backend
- Rust
- Custom-built web framework (routing, sessions, WebSockets)
- Tokio (async runtime)
- Tokio-tungstenite (WebSocket stream)
- SQLx (database layer)
- MySQL (persistent storage)
- Session cookies (authentication)

### Frontend
- HTML / CSS / JavaScript
- Fetch API
- WebSockets (real-time messaging)

### Security
- Argon2 (password hashing)

### Serialization
- Serde (JSON handling)

## Structure
```
SRC
│   db.rs
│   fileserver.rs
│   http_utils.rs
│   lib.rs
│   session_utils.rs
│
├───bin
│       main.rs
│
└───routes
    │   history.rs
    │   mod.rs
    │   routes.rs
    │   route_models.rs
    │
    ├───auth
    │       endpoints.rs
    │       form.rs
    │       mod.rs
    │
    └───chat
            mod.rs
            models.rs
            websocket.rs
```
## Configuration
### MySql migrations
In order to make server work, you have to instal my
```sql
create database chat_db;
```
```sql
use chat_db;
```
```sql
create table messages(
room varchar(255) not null,
username varchar(255) not null,
message text not null
);
```
```sql
create table users(
id int unique primary key auto_increment,
username varchar(255) not null,
password_hash varchar(255) not null
);
```
```sql
create table session (
id int unique primary key auto_increment,
username varchar(255) not null,
session_id varchar(255) not null
);
```
### Db connection
To connect you database you only need to paste it's url into `db_url` variable.
```rust
#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<Shutdown>(1024);
    let mut shutdown_rx = shutdown_tx.subscribe();
    let (tx, _rx) = tokio::sync::broadcast::channel::<ChatMessage>(1024);
    let db_url = "mysql://user:password@localhost:3306/dbname";
//snippet
```

### Dependencies
```rust
anyhow = "1.0.104"
argon2 = "0.5.3"
base64 = "0.22.1"
colored = "3.1.1"
cookie = "0.18"
futures-util = "0.3.32"
hex = "0.4.3"
rand = "0.8"
rand_core = { version = "0.6", features = ["std"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.150"
sha1 = "0.11.0"
sqlx = { version = "0.9", features = ["mysql", "runtime-tokio", "macros", "chrono", "mysql-rsa"] }
tokio = { version = "1.53", features = ["full"]}
tokio-tungstenite = "0.30.0"
urlencoding = "2.1.3"
```

## Endpoints
### GET /register
creates user and saves it to a database. Also sets up a cookie for a browser. Responds with ``400 BAD REQUEST`` if user exists.
### GET /login
looks up for a user in database, verifies password and sets up cookie for a browser  
If user does not exist, or incorrect password was passed - it responsds with ``400 BAD REQUEST`` and json message to specify problem.
### GET /me 
checks a cookie header and looks up for a match in databse if found, returns username as a response.  
Responds with ``404 NOT FOUND`` if no match is found.
### GET /logout
clears the cookie and logs out the user.
### GET /history 
collects all messages from database and returns them to client as an array.
### GET /ws 
upgrades connection to a WebSocket if user is verified. Responds with ``400 BAD REQUEST`` if there is  no cookie header in request. Responds with ``401 UNAUTHORIZED`` if cookie is invalid.
### fileserver: those are just endpoints that serve static files
- GET / responds with index.html
- GET /style.css responds with style.css
- GET /reset.css responds with reset.css
- GET /script.js responds with script.js
- GET /favicon.ico responds with page's icon
## A word about 
This project is remake of my old project:
https://github.com/LeakyDutchmann/Chat-app  
but with this time I decided to try implementing everythings without any   
framework such as [Rocket](https://rocket.rs/) or [Axum](https://crates.io/crates/axum)  
That meant, that I would have to parse all HTTP reaquests manually  
and implement fully async backend server on my own.  
That was what I did.

## License

This project is licensed under the **MIT License**.

Copyright (c) 2026 LeakyDutchmann

See the [LICENSE](./LICENSE) file for full details.