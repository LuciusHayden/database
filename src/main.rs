mod collections;
mod parser;
mod database;
mod wal;
mod auth;
mod session;
mod errors;
mod cli;

use crate::parser::{Command, Parser};
use crate::database::Database;
use crate::auth::Permissions;
use crate::cli::CLI;

fn main() {

    let (username, password, dir, new_user) = CLI::get_args();

    //let mut database = Database::new("data".to_string());
    let mut database = Database::load_data(dir).unwrap();

    if new_user {
        database.new_user(&username, &password, Permissions::User()).unwrap();
    }
    match database.login(username.to_string(), password.to_string()) {
        Ok(val) => val,
        Err(e) => {
            println!("{}", e);
            return ();
        }
    }
    let parser = Parser::new();

    CLI::start_repl(database, parser);
}

