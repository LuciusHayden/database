use serde::{Serialize, Deserialize};

use std::{
    option::Option,
    fs,
    io::{Read, Write}
};

use bincode;
use serde_json::{Map, Value};

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

    pub fn login(&mut self, username: String, password: String) -> Result<(), DatabaseError> {
        let session = self.auth_manager.login(username, password)?;
        self.current_session = Some(session);
        Ok(())
    }

    pub fn new_user(&mut self, username: &String, password: &String, permissions: Permissions) -> Result<(), DatabaseError> {
        self.auth_manager.new_user(&self.path, username, password, permissions)?;
        Ok(())
    }

    pub fn insert(&mut self, key : String, value: Value) -> Result<Value, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }

        match self.state {
            DatabaseState::Unselected() => Err(DatabaseError::CollectionError("Select a collection".to_string())),
            DatabaseState::SelectedCollection(collection) => {
                let entry = WALEntry::new(self.collections[collection].name.clone(),"INSERT".to_string(), key.clone(), Some(value.clone()));
                entry.log(format!("{}/wal.log", self.path).as_str());
                self.collections[collection].insert(key.clone(), value);
                Ok(Value::Null)
            },
        }
    }

    pub fn get(&self, key : String) -> Result<Value, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string()))
        }

        match self.state {
            DatabaseState::Unselected() => Err(DatabaseError::CollectionError("Select a collection".to_string())),
            DatabaseState::SelectedCollection(collection) => {
                let entry = WALEntry::new(self.collections[collection].name.clone(), "GET".to_string(), key.clone(), None); 
                entry.log(format!("{}/wal.log", self.path).as_str());
                match self.collections[collection].get(key.clone()) {
                    Some(value) => Ok(value),
                    None => Err(DatabaseError::ValueNotFound(key))
                }
            },
        }
    }

    pub fn delete(&mut self, key: String) -> Result<Value, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }
        match self.state {
            DatabaseState::Unselected() => Err(DatabaseError::CollectionError("Select a collection".to_string())),
            DatabaseState::SelectedCollection(collection) => {
                let entry = WALEntry::new(self.collections[collection].name.clone(),"DELETE".to_string(), key.clone(), None); 
                entry.log(format!("{}/wal.log", self.path).as_str());
                match self.collections[collection].delete(key.clone()) {
                    Some(value) => Ok(value),
                    None => Err(DatabaseError::ValueNotFound(key))
                }
            },
        }
    }

    fn select(&mut self, collection: String) -> Result<Value, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }
        match self.find_collection_by_name(&collection) {
            Some(index) => {
                self.state = DatabaseState::SelectedCollection(index);
                Ok(serde_json::from_str(format!("{} selected", &collection).as_str())?)
            },
            None => Err(DatabaseError::CollectionNotFound(collection))
        }
    }

    pub fn new_collection(&mut self, name: &String) -> Result<Value, DatabaseError> {
        if self.current_session.is_none() {
            return Err(DatabaseError::UserError("Login to access the database".to_string())) 
        }
        let collection = Collection::new(name.clone());
        self.collections.push(collection);
        fs::File::create(format!("{}/{}.db", self.path, name))?;
        Ok(serde_json::from_str(format!("{} created", name).as_str())?)
    }

    pub fn find_collection_by_name(&self, name: &String) -> Option<usize> {
        Some(self.collections.iter().position(|c| &c.name == name)?)
    }

    pub fn save_data(&self) -> Result<(), DatabaseError> {
        let _ = fs::create_dir_all(self.path.clone());
        for collection in &self.collections {
            let encoded : Vec<u8> = bincode::serialize(&collection)?;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(format!("{}/{}.db", &self.path, &collection.name))?;

            file.write_all(&encoded)?;
        }

        // clear the WAL 
        let mut wal = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(format!("{}/wal.log", self.path.clone()))?;

        wal.flush()?;

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
                    Err(_) => Collection::new(path.file_stem().unwrap().to_str().unwrap().to_string()),
                };
                collections.push(collection);
            }
        }

        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(format!("{}/users.log", path.clone()))?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        let auth_manager: AuthManager = bincode::deserialize(&contents)?;

        if collections.is_empty() {
            return Ok(Database::new(path.clone()))
        }

        let mut database = Database{ 
            collections, 
            path: path.clone(), 
            auth_manager, 
            wal_manager: WALManager::new(path), 
            state : DatabaseState::Unselected(),
            current_session: None,
        };

        let logs = database.wal_manager.read_wal_log();

        if logs.is_some() {
            for log in logs.expect("wal.log is not empty") { 
                database.select(log.collection.clone())?;
                let operation = log.convert_to_operation();
                database.operate_db(operation)?;
            }
        }
        database.state = DatabaseState::Unselected();

        Ok(database)
    }

    pub fn operate_db(&mut self, command: Command) -> Result<Value, DatabaseError> {
        match command {
            Command::INSERT(key, value) => self.insert(key, value),
            Command::GET(key) => self.get(key),
            Command::DELETE(key) => self.delete(key),
            Command::SELECT(key) => self.select(key),
            Command::NEW(key) => self.new_collection(&key),
            Command::ERROR() => Err(DatabaseError::Other("syntax error".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;
    use std::path::PathBuf;
    use std::fs;

    fn setup_persistent_test_database() -> (Database, String) {
        let test_dir = PathBuf::from("data/test_database");
        let db_path = test_dir.join("database_test.db");

        // Ensure the test directory exists
        fs::create_dir_all(&test_dir).unwrap();

        // Clear the old database file if it exists
        if db_path.exists() {
            fs::remove_file(&db_path).unwrap();
        }

        // Create a new, empty database file
        fs::File::create(&db_path).unwrap();

        (Database::new(db_path.to_str().unwrap().to_string()), db_path.to_str().unwrap().to_string())
    }

    #[test]
    fn saving_and_loading() {
        let (mut db , path) = setup_persistent_test_database();
        db.insert("foo".to_string(), "bar".to_string());
        db.save_data().unwrap();

        // ensure its actually loading the data
        std::mem::drop(db);

        let db = Database::load_data(path.to_string()).unwrap();
        let bar = db.get("foo".to_string()).unwrap();
        assert_eq!(bar, "bar" );
    }

    #[test]
    fn inserting_and_deleting() -> Result<(), String> {
        let path = "data";
        let _ = fs::remove_file(path).is_ok();

        let mut db = Database::new(path.to_string());
        
        db.insert("woo".to_string(), "warr".to_string());
        db.save_data().unwrap();

        std::mem::drop(db);

        let mut db = Database::load_data(path.to_string()).unwrap();
        db.delete("woo".to_string());
        match db.get("woo".to_string()) {
            Some(_)=> Err("Value was not deleted".to_string()),
            None => Ok(()),
        }
    }
}


