pub mod routes;
pub mod chat;
pub mod auth;
pub mod fileserver;

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};