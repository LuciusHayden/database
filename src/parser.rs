pub struct Parser {
}

#[derive(Debug)]
pub enum Command {
    INSERT(String, String),
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

        let split = &mut line.split(" ");
        let keyword = split.next().unwrap().to_string();
        let key: String = split.next().unwrap_or("").to_string().parse().unwrap_or("".to_string());
        let value = split.next();

        match keyword.as_str() {
            "INSERT" => Command::INSERT(key, value.unwrap().to_string()),
            "GET" => Command::GET(key),
            "DELETE" => Command::DELETE(key),
            "SELECT" => Command::SELECT(key),
            "NEW" => Command::NEW(key),
            _ => Command::ERROR(),
        }
    }
}

