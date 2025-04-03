use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use std::option::Option;

use bincode;
use std::fs;
use std::io::Read;
use std::io::{Result, Write};

use crate::wal::WALManager;
use crate::wal::WALEntry;
use crate::parser::Command;
use crate::collections::Collection;

#[derive(Serialize, Deserialize, Debug)]
enum DatabaseState {
    SelectedCollection(usize),
    Unselected(),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    path: String,
    wal_manager: WALManager, 
    collections: Vec<Collection>,
    state: DatabaseState,
}

impl Database {
    pub fn new(path: String) -> Database {
        Database { 
            path: path.clone(),
            wal_manager: WALManager::new(path),
            collections : Vec::new(),
            state: DatabaseState::Unselected() 
        }
    }

    pub fn insert(&mut self, key : String, value: String) -> Option<String> {
        match self.state {
            DatabaseState::Unselected() => None,
            DatabaseState::SelectedCollection(collection) => {
                let entry = WALEntry::new(self.collections[collection].name.clone(),"INSERT".to_string(), key.clone(), Some(value.clone()));
                entry.log(format!("{}/wal.log", self.path).as_str());
                self.collections[collection].insert(key.clone(), value.clone())
            },
        }
    }

    pub fn get(&self, key : String) -> Option<String> {
        match self.state {
            DatabaseState::Unselected() => Some("Select a collection".to_string()),
            DatabaseState::SelectedCollection(collection) => {
                let entry = WALEntry::new(self.collections[collection].name.clone(), "GET".to_string(), key.clone(), None); 
                entry.log(format!("{}/wal.log", self.path).as_str());
                self.collections[collection].get(key)
            },
        }
    }

    pub fn delete(&mut self, key: String) -> Option<String> {
        match self.state {
            DatabaseState::Unselected() => None,
            DatabaseState::SelectedCollection(collection) => {
                let entry = WALEntry::new(self.collections[collection].name.clone(),"GET".to_string(), key.clone(), None); 
                entry.log(format!("{}/wal.log", self.path).as_str());
                self.collections[collection].delete(key)
            },
        }
    }

    fn select(&mut self, collection: String) -> Option<String> {
        match self.find_collection_by_name(collection) {
            Some(index) => {
                self.state = DatabaseState::SelectedCollection(index);
                Some(index.to_string())
            },
            None => None,
        }
    }

    pub fn new_collection(&mut self, name: String) -> Option<String> {
        let collection = Collection::new(name);
        self.collections.push(collection);
        None
    }

    pub fn find_collection_by_name(&self, name: String) -> Option<usize> {
        Some(self.collections.iter().position(|c| c.name == name)?)
    }

    pub fn save_data(&self) -> std::io::Result<()> {
        let _ = fs::create_dir_all(self.path.clone());
        for collection in &self.collections {
            let encoded : Vec<u8> = bincode::serialize(&collection).unwrap();
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(format!("{}/{}.db", &self.path, &collection.name)).unwrap();

            file.write_all(&encoded).unwrap();
        }

        println!("{}", self.path.clone());

        // clear the WAL log 
        let mut wal = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(format!("{}/wal.log", self.path.clone())).unwrap();

        wal.flush().unwrap();

        Ok(())
    }

    pub fn load_data(path : String) -> Result<Self> {
        let mut collections : Vec<Collection> = Vec::new();
            
        for entry in fs::read_dir(&path).unwrap() {
            let path = entry?.path();
            if path.is_file() && path.extension() == Some("db".as_ref()) {
                let mut file = fs::OpenOptions::new()
                    .read(true)
                    .open(path.clone()).unwrap();

                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();
                let collection : Collection = match bincode::deserialize(&contents) {
                    Ok(collection) => collection,
                    Err(_) => Collection::new(path.file_stem().unwrap().to_str().unwrap().to_string()),
                };
                collections.push(collection);
            }
        }

        if collections.is_empty() {
            return Ok(Database::new("data".to_string()))
        }

        let mut database = Database{ collections, path: path.clone(), wal_manager: WALManager::new(path), state : DatabaseState::Unselected() };

        let logs = database.wal_manager.read_wal_log();

        if logs.is_some() {
            for log in logs.expect("wal.log is not empty") { 
                database.select(log.collection.clone());
                let operation = log.convert_to_operation();
                database.operate_db(operation);
            }
        }
        database.state = DatabaseState::Unselected();

        Ok(database)
    }

    pub fn operate_db(&mut self, command: Command) -> Option<String>{
        match command {
            Command::INSERT(key, value) => self.insert(key, value),
            Command::GET(key) => self.get(key),
            Command::DELETE(key) => self.delete(key),
            Command::SELECT(key) => self.select(key),
            Command::NEW(key) => self.new_collection(key),
            Command::ERROR() => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;
    use std::path::PathBuf;
    use std::fs;
    use tempdir::TempDir;
    use std::path::Path;

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


