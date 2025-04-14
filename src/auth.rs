use serde::{Serialize, Deserialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::fs::OpenOptions;
use std::io::Write;
use std::error::Error;
use std::collections::HashMap;

use crate::session::Session;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub fn new(path: &str) -> AuthManager {

        let manager = AuthManager{ users: HashMap::new(), current: None };
        let encoded : Vec<u8> = bincode::serialize(&manager).unwrap();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("{}/users.log", path)).unwrap();

        file.write_all(&encoded).unwrap();
        manager
    }

    pub fn login(&mut self, username: String, password : String) -> Result<Session, Box<dyn Error>> {
        match self.users.get(&username) {
            Some(user) => {
                match AuthManager::verify_password(password, &user.password_hash) {
                    true => {
                        self.current = Some(user.username.clone());
                        Ok(AuthManager::create_session(user))
                    }
                    false => Err("Incorrect Password".into()),
                }
            }
            None => Err("User doesnt exist".into())
        }
    }
    
    pub fn new_user(&mut self, path: &String, username : String, password: String, permissions: Permissions) -> Result<(), Box<dyn Error>> {
        let password_hash = hash(password, DEFAULT_COST).unwrap();
        if self.users.get(&username).is_none() {
            return Err("Username already taken".into())
        }
        let user = User{ username : username.clone(), password_hash, permissions };
        self.users.insert(username, user);

        // might eventually move this
        let encoded : Vec<u8> = bincode::serialize(&self).unwrap();
        println!("{:#?}", self);

        let mut file = OpenOptions::new()
            .write(true)
            .open(format!("{}/users.log", path))
            .unwrap();

        file.write_all(&encoded).unwrap();
        Ok(())
    }

    fn verify_password(password: String, hash: &str) -> bool { 
        verify(password, hash).unwrap_or(false)
    }
}

