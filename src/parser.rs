use serde_json::Value;
use serde_json::json;

use crate::errors::DatabaseError;


pub struct Parser {
}

#[derive(Debug)]
pub enum Command {
    INSERT(String, Value),
    GET(String),
    DELETE(String),
    SELECT(String),
    NEW(String),
    WHICH(String),
    ERROR(),
}

pub enum Token {
    INSERT,
    GET,
    DELETE,
    SELECT,
    WHICH,
    NEW, 
    IDENTIFIER(String),
    JSON(Value),

}
impl Parser {
    pub fn new()-> Parser  {
        Parser {}
    }

    pub fn get_command(&self, line: &str) -> Result<Command, DatabaseError> {
        let tokens = Parser::lexer(line)?;

        Parser::parse(tokens)
    }

    fn parse(tokens: Vec<Token>) -> Result<Command, DatabaseError> { 
        match tokens.get(0) {
            Some(Token::INSERT) => {
                if let (Some(Token::IDENTIFIER(key)), Some(Token::JSON(value))) = (tokens.get(1), tokens.get(2)) {
                    Ok(Command::INSERT(key.clone(), value.clone()))
                } else {
                    Err(DatabaseError::SyntaxError("Missing identifier or json".to_string()))
                }
            }
            Some(Token::GET) => {
                if let Some(Token::IDENTIFIER(key)) = tokens.get(1) {
                    Ok(Command::GET(key.clone()))
                } else {
                    Err(DatabaseError::SyntaxError("Missing identifier".to_string()))
                }
            }
            Some(Token::DELETE) => {
                if let Some(Token::IDENTIFIER(key)) = tokens.get(1) {
                    Ok(Command::DELETE(key.clone()))
                } else {
                    Err(DatabaseError::SyntaxError("".to_string()))
                }
            }
            Some(Token::SELECT) => {
                if let Some(Token::IDENTIFIER(name)) = tokens.get(1) {
                    Ok(Command::SELECT(name.clone()))
                } else {
                    Err(DatabaseError::SyntaxError("Missing Identifier".to_string()))
                }
            }
            Some(Token::NEW) => {
                if let Some(Token::IDENTIFIER(name)) = tokens.get(1) {
                    Ok(Command::NEW(name.clone()))
                } else {
                    Err(DatabaseError::SyntaxError("Missing Identifier".to_string()))
                }
            }
            Some(Token::WHICH) => {
                if let Some(Token::IDENTIFIER(name)) = tokens.get(1) {
                    Ok(Command::WHICH(name.clone()))
                } else {
                    Err(DatabaseError::SyntaxError("Missing Identifier".to_string()))
                }
            }
            _ => Err(DatabaseError::SyntaxError("Unknown command".to_string())),
        }
    }

    fn lexer(line: &str) -> Result<Vec<Token>, DatabaseError> {
        let result = Parser::lex_insert(line)?;
        let mut results = Vec::new();

        if result.is_some() {
            let result = result.unwrap();
            let token_a = match result.0.to_uppercase().as_str() {
                "INSERT" => Token::INSERT,
                "GET" => Token::GET,
                "DELETE" => Token::DELETE,
                "SELECT" => Token::SELECT,
                "NEW" => Token::NEW,
                "WHICH" => Token::WHICH,
                _ => return Err(DatabaseError::SyntaxError("Unknown command".to_string())),
            };

            let token_b = Token::IDENTIFIER(result.1);

            let token_c = Token::JSON(json!(result.2));

            results.push(token_a);
            results.push(token_b);
            results.push(token_c);

            return Ok(results)

        }
        Err(DatabaseError::SyntaxError("lexer".to_string()))

    }

    fn lex_insert(input: &str) -> Result<Option<(String, String, Option<serde_json::Value>)>, DatabaseError> {
        let mut parts = input.trim().splitn(3, ' '); 
        let cmd = parts.next().ok_or(DatabaseError::SyntaxError("".to_string()))?;
        let collection = parts.next().ok_or(DatabaseError::SyntaxError("".to_string()))?;
        let json_str = parts.next();

        let mut json_value = None;

        if json_str.is_some() {
            json_value = serde_json::from_str(json_str.unwrap()).ok();
        } 
        Ok(Some((cmd.to_string(), collection.to_string(), json_value)))
    }
}

