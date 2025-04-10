use serde::{Serialize, Deserialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::fs::OpenOptions;
use std::io::Write;
use std::error::Error;


#[derive(Serialize, Deserialize, Debug)]
pub enum Permissions {
    Admin(),
    User(),
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    password_hash: String,
    permissions: Permissions,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthManager {
    users: Vec<User>,
    current: Option<String>,
}

impl AuthManager {

    pub fn new(path: &str) -> AuthManager {

        let manager = AuthManager{ users: Vec::new(), current: None };
        let encoded : Vec<u8> = bincode::serialize(&manager).unwrap();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(format!("{}/users.log", path)).unwrap();

        file.write_all(&encoded).unwrap();
        manager
    }

    pub fn login(&mut self, username: String, password : String) -> Result<String, Box<dyn Error>> {
        match self.users.iter().find(|u| u.username == username) {
            Some(user) => {
                match hash(password, DEFAULT_COST) {
                    Ok(hash) => {
                        if hash == user.password_hash {
                            self.current = Some(user.username.clone());
                            Ok(user.username.clone())
                        } else {
                           Err("Incorrect Password".into()) 
                        }
                    }
                    Err(_) => Err("Failed to hash".into())
                }
            }
            None => Err("User doesnt exist".into())
        }
    }
    
    pub fn new_user(&mut self, username : String, password: String, permissions: Permissions) {
        let password_hash = hash(password, DEFAULT_COST).unwrap();
        self.users.push(User{ username, password_hash, permissions });
    }

    fn verify_password(password: String, hash: &str) -> bool { 
        verify(password, hash).unwrap_or(false)
    }
}

