# Chat

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-1.53-blue?logo=tokio)
![SQLx](https://img.shields.io/badge/SQLx-0.9-orange?logo=database)
![Serde](https://img.shields.io/badge/Serde-1.0-purple?logo=rust)
![Docker](https://img.shields.io/badge/Docker-blue?logo=docker)
![MySQL](https://img.shields.io/badge/MySQL-4479A1?logo=mysql&logoColor=white)

A fully async, framework-free web chat built from scratch in Rust, including
manual HTTP parsing, WebSocket implementation, cookies, and SQLx-powered persistence.

## Features

- User registration with Argon2 password hashing
- Login/logout endpoints
- Session-based authentication using cookies
- Database integration via SQLx
- WebSocket chat support
- Session verification via the `/me` endpoint
- Clean service-layer architecture
- Pure async architecture
- Manual parsing and handling of HTTP requests
- SHA-1-hashed session cookies
- Frontend session restoration
- Session cleanup on server shutdown
- Manually implemented simple file server
- Pre-built Docker container
- Can be run both inside and outside Docker

## Tech Stack

### Backend

- Rust
- Custom-built web framework (routing, sessions, WebSockets)
- Tokio (async runtime)
- Tokio-Tungstenite (WebSocket implementation)
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

### Deployment

- Docker

## Structure

```text
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

Clone the repository:

```bash
git clone https://github.com/LeakyDutchmann/chat.git
```

Then build and start the application:

```bash
docker compose up --build
```

You're ready to go.

To start the application again later:

```bash
docker compose up
```

or launch it from Docker Desktop.

If you don't have Docker installed, follow the official guide:

https://www.docker.com/get-started/

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

Creates a new user and stores it in the database. Also sets a session cookie in the browser.

Responds with `400 BAD REQUEST` if the user already exists.

### GET /login

Looks up the user in the database, verifies the password, and sets a session cookie.

If the user does not exist or the password is incorrect, the server responds with `400 BAD REQUEST` along with a JSON message describing the error.

### GET /me

Checks the cookie header and looks for a matching session in the database.

If found, returns the authenticated user's username.

Responds with `404 NOT FOUND` if no valid session is found.

### GET /logout

Clears the session cookie and logs the user out.

### GET /history

Retrieves all chat messages from the database and returns them as a JSON array.

### GET /ws

Upgrades the connection to a WebSocket if the user is authenticated.

Responds with `400 BAD REQUEST` if the request does not contain a cookie header.

Responds with `401 UNAUTHORIZED` if the cookie is invalid.

### File server

These endpoints serve static files:

- `GET /` → `index.html`
- `GET /style.css` → `style.css`
- `GET /reset.css` → `reset.css`
- `GET /script.js` → `script.js`
- `GET /favicon.ico` → the site's icon

## A word about this project

This project is a remake of my previous project:

https://github.com/LeakyDutchmann/Chat-app

This time, I decided to implement everything without using a web framework such as Rocket or Axum.

That meant manually parsing HTTP requests, implementing routing, handling sessions, and building a fully asynchronous backend server from scratch.

That's exactly what this project does.

## License

This project is licensed under the **MIT License**.

Copyright (c) 2026 LeakyDutchmann

See the [LICENSE](./LICENSE) file for full details.