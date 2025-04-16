use serde_json::Value;


pub struct Parser {
}

#[derive(Debug)]
pub enum Command {
    INSERT(String, Value),
    GET(String),
    DELETE(String),
    SELECT(String),
    NEW(String),
    ERROR(),
}

impl Parser {
    pub fn new()-> Parser  {
        Parser {}
    }

    pub fn parse(&self, line: String) -> Command { 

        // currently cant handle json since it splits at the space
        // aslo collections dont work for some reason 
        let split = &mut line.split(" ");
        let keyword = split.next().unwrap().to_string();
        let key: String = split.next().unwrap_or("").to_string().parse().unwrap_or("".to_string());
        let value = split.next();

        match keyword.as_str() {
            "INSERT" => Command::INSERT(key, serde_json::from_str::<Value>(value.unwrap()).unwrap()),
            "GET" => Command::GET(key),
            "DELETE" => Command::DELETE(key),
            "SELECT" => Command::SELECT(key),
            "NEW" => Command::NEW(key),
            _ => Command::ERROR(),
        }
    }
}

