use clap::Parser;
use std::io::{self, Write};


use crate::parser::Parser as ReplParser;
use crate::database::Database;
use crate::database::Response;

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

            let command = parser.get_command(&input.trim());
            match command { 
                Ok(command) => {
                    let result = database.operate_db(command);
                    match result {
                        Ok(Response::Value(serde_json::Value::Null)) => (),
                        Ok(Response::Value(result)) => println!("{}", result),
                        Ok(Response::Message(message)) => println!("{}", message),
                        Err(e) => println!("{}", e),
                    }
                    }
                Err(e) => println!("{}", e),
            }
        }
        println!("Saving");
        database.save_data().unwrap();

    }
    
}

