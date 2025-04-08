mod collections;
mod parser;
mod database;
mod wal;
use crate::parser::{Command, Parser};
use crate::database::Database;
use crate::wal::WALEntry;

fn main() {
    let mut database = Database::load_data("data".to_string()).unwrap();
    println!("{:#?}", database);

    let parser = Parser::new();
    let command = parser.parse("NEW collection".to_string());
    database.operate_db(command);

    let command = parser.parse("SELECT collection".to_string());
    database.operate_db(command);


    let command = parser.parse("INSERT TEST 5".to_string());
    let _result = database.operate_db(command);

    let command = parser.parse("GET TEST".to_string());

    let result = database.operate_db(command);
    println!("{}", result.unwrap());

    database.save_data().unwrap();
}

