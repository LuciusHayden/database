use serde::{Serialize, Deserialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::HashMap;

use crate::errors::DatabaseError;
use crate::session::Session;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Permissions {
    Admin(),
    User(),
    Guest(),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    password_hash: String,
    pub permissions: Permissions,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthManager {
    users: HashMap<String, User>,
    current: Option<String>,
}

impl AuthManager {
    pub fn create_session(user: &User) -> Session {
        Session{ user: user.username.clone(), permissions: user.permissions.clone()}
    }

    pub fn new(path: &str) -> Result<AuthManager, DatabaseError> {

        let manager = AuthManager{ users: HashMap::new(), current: None };
        let encoded : Vec<u8> = bincode::serialize(&manager)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("{}/users.log", path))?;

        file.write_all(&encoded)?;
        Ok(manager)
    }

    pub fn login(&mut self, username: String, password : String) -> Result<Session, DatabaseError> {
        match self.users.get(&username) {
            Some(user) => {
                match AuthManager::verify_password(password, &user.password_hash) {
                    true => {
                        self.current = Some(user.username.clone());
                        Ok(AuthManager::create_session(user))
                    }
                    false => Err(DatabaseError::UserError("Incorrect Password".to_string())),
                }
            }
            None => Err(DatabaseError::UserError("User not found".to_string()))
        }
    }
    
    pub fn new_user(&mut self, path: &String, username : &String, password: &String, permissions: Permissions) -> Result<(), DatabaseError> {
        let password_hash = hash(password, DEFAULT_COST)?;
        if self.users.get(username).is_some() {
            return Err(DatabaseError::UserError("Username already taken".to_string()))
        }

        let user = User{ username : username.clone(), password_hash, permissions };
        self.users.insert(username.to_string(), user);

        // might eventually move this
        let encoded : Vec<u8> = bincode::serialize(&self)?;

        let mut file = OpenOptions::new()
            .write(true)
            .open(format!("{}/users.log", path))?;

        file.write_all(&encoded)?;
        Ok(())
    }

    fn verify_password(password: String, hash: &str) -> bool { 
        verify(password, hash).unwrap_or(false)
    }
}

