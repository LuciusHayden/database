mod parser;
mod database;
use crate::parser::{Command, Parser};
use crate::database::Database;

fn main() {

    let parser = Parser::new();
    let command = parser.parse("INSERT TEST 5".to_string());
    let mut database = Database::new();

    let _result = match command {
        Command::INSERT(key, value) => database.insert(key, value),
        Command::GET(key) => database.get(key),
        Command::DELETE(key) => database.delete(key),
        _ => None,
    };
}


