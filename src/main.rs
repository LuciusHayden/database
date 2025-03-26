mod parser;
mod database;
mod wal;
use crate::parser::{Command, Parser};
use crate::database::Database;
use crate::wal::WALEntry;

fn main() {
    let mut database = Database::load_data("data/database.db").unwrap();

    let parser = Parser::new();

    let command = parser.parse("INSERT TEST 5".to_string());
    let result = operate(&mut database, command);
    println!("{}", result.unwrap());

    let command = parser.parse("GET TEST".to_string());
    let result = operate(&mut database, command);
    println!("{}", result.unwrap());


    WALEntry::operate(Command::INSERT("foo".to_string(), "bar".to_string()));
    database.save_data("data/database.db").unwrap();
}


fn operate(database: &mut Database, command: Command) -> Option<String> {
    match command {
        Command::INSERT(key, value) => database.insert(key, value),
        Command::GET(key) => database.get(key),
        Command::DELETE(key) => database.delete(key),
        _ => None,
    }
}
