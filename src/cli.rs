use clap::Parser;
use std::io::{self, Write};


use crate::parser::Parser as ReplParser;
use crate::database::Database;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CLI {
    #[arg(short, long)]
    username: String,

    #[arg(short, long)]
    password: String,

    #[arg(short, long, default_value="./data")]
    dir: String,

    #[arg(short, long, default_value_t=false)]
    new_user: bool,

}


impl CLI {
    pub fn get_args() -> (String, String, String, bool) {
        let args = CLI::parse();
        (args.username, args.password, args.dir, args.new_user)
    }

    pub fn start_repl(mut database : Database, parser: ReplParser) -> () {
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
    
}

