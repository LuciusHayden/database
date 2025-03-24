pub struct Parser {
}

pub enum Command {
    INSERT(String, String),
    GET(String),
    DELETE(String),
    ERROR(),
}

impl Parser {
    pub fn new()-> Parser  {
        Parser {}
    }

    pub fn parse(&self, line: String) -> Command { 

        let split = &mut line.split(" ");
        let keyword = split.next().unwrap().to_string();
        let key: String = split.next().unwrap().to_string().parse().unwrap();
        let value = split.next().unwrap().to_string();

        match keyword.as_str() {
            "INSERT" => Command::INSERT(key, value),
            "GET" => Command::GET(key),
            "DELETE" => Command::DELETE(key),
            _ => Command::ERROR(),
        }
    }
}

