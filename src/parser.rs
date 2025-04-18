use serde_json::Value;
use serde_json::json;


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

pub enum Token {
    INSERT,
    GET,
    DELETE,
    SELECT,
    NEW, 
    IDENTIFIER(String),
    JSON(Value),
    NONE(),

}
impl Parser {
    pub fn new()-> Parser  {
        Parser {}
    }

    pub fn get_command(&self, line: &str) -> Command {
        let tokens = Parser::lexer(line);
        match Parser::parse(tokens) {
            Some(command) => command,
            None => Command::ERROR(),
        }
    }

    fn parse(tokens: Vec<Token>) -> Option<Command> { 
        match tokens.get(0).unwrap() {
            Token::INSERT=> {
            if let (Some(Token::IDENTIFIER(key)), Some(Token::JSON(value))) = (tokens.get(1), tokens.get(2)) {
                Some(Command::INSERT(key.clone(), value.clone()))
            } else {
                None
            }
        }
        Token::GET=> {
            if let Some(Token::IDENTIFIER(key)) = tokens.get(1) {
                Some(Command::GET(key.clone()))
            } else {
                None
            }
        }
        Token::DELETE=> {
            if let Some(Token::IDENTIFIER(key)) = tokens.get(1) {
                Some(Command::DELETE(key.clone()))
            } else {
                None
            }
        }
        Token::SELECT=> {
            if let Some(Token::IDENTIFIER(name)) = tokens.get(1) {
                Some(Command::SELECT(name.clone()))
            } else {
                None
            }
        }
        Token::NEW=> {
            if let Some(Token::IDENTIFIER(name)) = tokens.get(1) {
                Some(Command::NEW(name.clone()))
            } else {
                None
            }
        }
        _ => None,
        }
    }

    fn lexer(line: &str) -> Vec<Token> {
        let result = Parser::lex_insert(line).unwrap();
        let mut results = Vec::new();

        let token_a = match result.0.as_str() {
            "INSERT" => Token::INSERT,
            "GET" => Token::GET,
            "DELETE" => Token::DELETE,
            "SELECT" => Token::SELECT,
            "NEW" => Token::NEW,
            _ => Token::NONE(),
        };


        let token_b = Token::IDENTIFIER(result.1);

        let token_c = Token::JSON(json!(result.2));

        results.push(token_a);
        results.push(token_b);
        results.push(token_c);

        results
    }

    fn lex_insert(input: &str) -> Option<(String, String, Option<serde_json::Value>)> {
        let mut parts = input.trim().splitn(3, ' '); // only split into 3 parts
        let cmd = parts.next().unwrap();
        let collection = parts.next().unwrap();
        let json_str = parts.next();

        let mut json_value = None;

        if json_str.is_some() {
            json_value= serde_json::from_str(json_str.unwrap()).ok()?;
        }
        Some((cmd.to_string(), collection.to_string(), json_value))
    }
}

