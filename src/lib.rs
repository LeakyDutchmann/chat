pub mod routes;
use colored::Colorize;

#[derive(Clone)]
pub struct ShutDown;

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