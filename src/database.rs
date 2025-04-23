use serde::{Serialize, Deserialize};

use std::{
    option::Option,
    fs,
    io::{Read, Write}
};

use bincode;
use serde_json::Value;

use crate::wal::WALManager;
use crate::wal::WALEntry;
use crate::parser::Command;
use crate::collections::Collection;
use crate::auth::{Permissions, AuthManager};
use crate::session::Session;
use crate::errors::DatabaseError;

#[derive(Serialize, Deserialize, Debug)]
enum DatabaseState {
    SelectedCollection(usize),
    Unselected(),
}

#[derive(Debug)]
pub enum Response {
    Message(String),
    Value(Value),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    path: String,
    wal_manager: WALManager, 
    auth_manager: AuthManager, 
    collections: Vec<Collection>,
    state: DatabaseState,
    current_session: Option<Session>,
}

impl Database {
    pub fn new(path: String) -> Database {
        match Database::load_data(path.clone()) {
            Ok(database) => database,
            Err(_) => {
                fs::create_dir_all(&path).unwrap();
                Database { 
                    path: path.clone(),
                    wal_manager: WALManager::new(path.clone()),
                    auth_manager: AuthManager::new(path.as_str()).unwrap(),
                    collections : Vec::new(),
                    state: DatabaseState::Unselected(),
                    current_session: None,
                }

            }
        }
    }

    pub fn login(&mut self, username: String, password: String) -> Result<(), DatabaseError> {
        let session = self.auth_manager.login(username, password)?;
        self.current_session = Some(session);
        self.write_and_clear_wal_log()?;
        self.state = DatabaseState::Unselected();
        Ok(())
    }

    pub fn new_user(&mut self, username: &String, password: &String, permissions: Permissions) -> Result<(), DatabaseError> {
        self.auth_manager.new_user(&self.path, username, password, permissions)?;
        Ok(())
    }

    pub fn insert(&mut self, key : String, value: Value) -> Result<Response, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }

        if &self.current_session.as_ref().unwrap().permissions == &Permissions::Guest() {
            return Err(DatabaseError::PermissionDenied("Guest permissions cannot write data".to_string()))
        }

        match self.state {
            DatabaseState::Unselected() => Err(DatabaseError::CollectionError("Select a collection".to_string())),
            DatabaseState::SelectedCollection(collection) => {
                let entry = WALEntry::new(self.collections[collection].name.clone(),"INSERT".to_string(), key.clone(), Some(value.clone()));
                entry.log(format!("{}/wal.log", self.path).as_str());
                self.collections[collection].insert(key.clone(), value);
                Ok(Response::Value(Value::Null))
            },
        }
    }

    pub fn get(&self, key : String) -> Result<Response, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string()))
        }

        match self.state {
            DatabaseState::Unselected() => Err(DatabaseError::CollectionError("Select a collection".to_string())),
            DatabaseState::SelectedCollection(collection) => {
                match self.collections[collection].get(key.clone()) {
                    Some(value) => Ok(Response::Value(value)),
                    None => Err(DatabaseError::ValueNotFound(key))
                }
            },
        }
    }

    pub fn delete(&mut self, key: String) -> Result<Response, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }
        if &self.current_session.as_ref().unwrap().permissions == &Permissions::Guest() {
            return Err(DatabaseError::PermissionDenied("Guest permissions cannot write data".to_string()))
        }
        match self.state {
            DatabaseState::Unselected() => Err(DatabaseError::CollectionError("Select a collection".to_string())),
            DatabaseState::SelectedCollection(collection) => {
                let entry = WALEntry::new(self.collections[collection].name.clone(),"DELETE".to_string(), key.clone(), None); 
                entry.log(format!("{}/wal.log", self.path).as_str());
                match self.collections[collection].delete(key.clone()) {
                    Some(value) => Ok(Response::Value(value)),
                    None => Err(DatabaseError::ValueNotFound(key))
                }
            },
        }
    }

    pub fn select(&mut self, collection: String) -> Result<Response, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }
        match self.find_collection_by_name(&collection) {
            Some(index) => {
                self.state = DatabaseState::SelectedCollection(index);
                Ok(Response::Message(format!("{} selected", collection)))
            },
            None => Err(DatabaseError::CollectionNotFound(collection))
        }
    }

    pub fn new_collection(&mut self, name: &String) -> Result<Response, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }
        if &self.current_session.as_ref().unwrap().permissions == &Permissions::Guest() {
            return Err(DatabaseError::PermissionDenied("Guest permissions cannot write data".to_string()))
        }
        let collection = Collection::new(name.clone());
        self.collections.push(collection);
        fs::File::create(format!("{}/{}.db", self.path, name))?;
        Ok(Response::Message(format!("{} created", name)))
    }

    pub fn which(&self, key: String) -> Result<Response, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }

        if key == "collection".to_string() {
            match self.state {
                DatabaseState::SelectedCollection(index) => return Ok(Response::Message(format!("{} selected", self.collections.get(index).unwrap().name))),
                DatabaseState::Unselected() => return Ok(Response::Message("No collection selected".to_string()))
            }
        };
        if key == "path".to_string() {
            return Ok(Response::Message(self.path.clone()))
        };
        if key == "user".to_string() {
            return Ok(Response::Message(self.current_session.as_ref().unwrap().user.clone()))
        }
        Err(DatabaseError::ValueNotFound(format!("{} invalid", key)))
    }

    pub fn find_collection_by_name(&self, name: &String) -> Option<usize> {
        Some(self.collections.iter().position(|c| &c.name == name)?)
    }

    pub fn save_data(&mut self) -> Result<(), DatabaseError> {
        if &self.current_session.as_ref().unwrap().permissions == &Permissions::Guest() {
            return Err(DatabaseError::PermissionDenied("Guest permissions cannot write data".to_string()))
        }

        self.write_and_clear_wal_log()?;
        fs::create_dir_all(self.path.clone())?;
        for collection in &self.collections {
            let encoded : Vec<u8> = bincode::serialize(&collection)?;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(format!("{}/{}.db", &self.path, &collection.name))?;

            file.write_all(&encoded)?;
        }

        Ok(())
    }

    pub fn load_data(path : String) -> Result<Self, DatabaseError> {
        let mut collections : Vec<Collection> = Vec::new();

        for entry in fs::read_dir(&path)? {
            let path = entry?.path();
            if path.is_file() && path.extension() == Some("db".as_ref()) {
                let mut file = fs::OpenOptions::new()
                    .read(true)
                    .open(path.clone())?;

                let mut contents = Vec::new();
                file.read_to_end(&mut contents)?;
                let collection : Collection = match bincode::deserialize(&contents) {
                    Ok(collection) => collection,
                    Err(e) => {
                        println!("{}", e);
                        Collection::new(path.file_stem().unwrap().to_str().unwrap().to_string())
                    }
                };
                collections.push(collection);
                std::mem::drop(file);
            }
        }

        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(format!("{}/users.log", path.clone()))?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        std::mem::drop(file);
        let auth_manager: AuthManager = bincode::deserialize(&contents)?;

        let database = Database{ 
            collections, 
            path: path.clone(), 
            auth_manager, 
            wal_manager: WALManager::new(path), 
            state : DatabaseState::Unselected(),
            current_session: None,
        };


        Ok(database)
    }

    pub fn operate_db(&mut self, command: Command) -> Result<Response, DatabaseError> {
        match command {
            Command::INSERT(key, value) => self.insert(key, value),
            Command::GET(key) => self.get(key),
            Command::DELETE(key) => self.delete(key),
            Command::SELECT(key) => self.select(key),
            Command::NEW(key) => self.new_collection(&key),
            Command::WHICH(key) => self.which(key),
            Command::ERROR() => Err(DatabaseError::Other("syntax error".to_string())),
        }
    }

    pub fn write_and_clear_wal_log(&mut self) -> Result<(), DatabaseError> {
        let logs = self.wal_manager.read_wal_log();
        if logs.is_ok() {
            for log in logs.expect("wal.log is not empty") { 
                self.select(log.collection.clone()).unwrap();
                let operation = log.convert_to_operation();
                self.operate_db(operation)?;
            }
        }

        // clear the WAL 
        let mut wal = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(format!("{}/wal.log", &self.path))?;

        wal.flush()?;
        std::mem::drop(wal);
        Ok(())
    }
}

