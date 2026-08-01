pub mod routes;
pub mod fileserver;
pub mod http_utils;
pub mod db;
pub mod session_utils;

use colored::Colorize;

#[derive(Clone)]
pub struct Shutdown;

pub fn log(line: &str, color: &str, underline: bool) {
    let line = match color {
        "green" => line.green(),
        "blue" => line.blue(),
        "red" => line.red(),
        "yellow" => line.yellow(),
        "magenta" => line.magenta(),
        _ => line.white(),
    };
    if underline {
        println!("{}", line.underline());
    } else {
        println!("{}", line);
    }
}