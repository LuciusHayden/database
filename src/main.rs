mod collections;
mod parser;
mod database;
mod wal;
mod auth;
mod session;
mod errors;

use std::env;
use std::io::{self, Write};

use crate::parser::{Command, Parser};
use crate::database::Database;
use crate::auth::Permissions;

fn main() {

    let args: Vec<String> = env::args().collect();

    let username = &args[1];
    let password = &args[2];

    //let mut database = Database::new("data".to_string());
    let mut database = Database::load_data("data".to_string()).unwrap();
    //let _ = database.new_user("lucius".to_string(), "123".to_string(), Permissions::Admin());
    match database.login(username.to_string(), password.to_string()) {
        Ok(val) => val,
        Err(e) => {
            println!("{}", e);
            return ();
        }
    }
    //println!("{:#?}", database);
    //
    let parser = Parser::new();
    //let command = parser.parse("NEW collection".to_string());
    //database.operate_db(command);
    //
    //let command = parser.parse("SELECT collection".to_string());
    //database.operate_db(command);
    //
    //let command = parser.parse("INSERT TEST 5".to_string());
    //let _result = database.operate_db(command);
    //
    //let command = parser.parse("GET TEST".to_string());
    //
    //let result = database.operate_db(command);
    //println!("{}", result.unwrap());

    let mut input = String::new();
    loop {
        print!("Database > ");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().eq("EXIT") || input.trim().eq("QUIT") {
            break
        }

        let command = parser.parse(input.trim().to_string());
        let result = database.operate_db(command);
        match result {
            Some(result) => println!("{}", result),
            None => (),
        }
    }
    database.save_data().unwrap();

}

