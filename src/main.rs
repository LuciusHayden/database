mod collections;
mod parser;
mod database;
mod wal;
mod auth;
mod session;
mod errors;
mod cli;

use std::io::{self, Write};

use crate::parser::{Command, Parser};
use crate::database::Database;
use crate::auth::Permissions;
use crate::cli::CLI;

fn main() {

    let (username, password, dir) = CLI::get_args();

    //let mut database = Database::new("data".to_string());
    let mut database = Database::load_data(dir).unwrap();
    let _ = database.new_user("lucius".to_string(), "123".to_string(), Permissions::Admin());
    match database.login(username.to_string(), password.to_string()) {
        Ok(val) => val,
        Err(e) => {
            println!("{}", e);
            return ();
        }
    }
    let parser = Parser::new();

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
            Ok(Some(result)) => println!("{}", result),
            Ok(None) => (),
            Err(e) => println!("{}", e),
        }
    }
    database.save_data().unwrap();
}

