pub mod routes;
pub mod chat;
pub mod auth;
pub mod fileserver;
pub mod http_utils;
pub mod db;
pub mod route_models;
pub mod history;
pub mod http;

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use urlencoding::decode;

use super::*;