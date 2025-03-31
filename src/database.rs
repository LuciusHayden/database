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
    SelectedCollection(Collection),
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
            DatabaseState::SelectedCollection(ref mut collection) => {
                let entry = WALEntry::new("INSERT".to_string(), key.clone(), Some(value.clone()));
                entry.log(self.wal_manager.path.as_str());
                collection.insert(key.clone(), value.clone())
            },
        }
    }

    pub fn get(&self, key : String) -> Option<String> {
        match self.state {
            DatabaseState::Unselected() => None,
            DatabaseState::SelectedCollection(ref collection) => {
                let entry = WALEntry::new("GET".to_string(), key.clone(), None); 
                entry.log(self.wal_manager.path.as_str());
                collection.get(key)
            },
        }
    }

    pub fn delete(&mut self, key: String) -> Option<String> {
        match self.state {
            DatabaseState::Unselected() => None,
            DatabaseState::SelectedCollection(ref mut collection) => {
                let entry = WALEntry::new("GET".to_string(), key.clone(), None); 
                entry.log(self.wal_manager.path.as_str());
                collection.delete(key)
            },
        }
    }

    pub fn save_data(&self, path: &str) -> std::io::Result<()> {
        fs::create_dir_all("data")?;
        let encoded : Vec<u8> = bincode::serialize(self).unwrap();
        let mut file = fs::File::create(path)?;
        file.write_all(&encoded)?;

        // clear the WAL log 
        let mut wal = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.wal_manager.path.clone())?;

        wal.flush()?;

        Ok(())
    }

    pub fn load_data(path: &str) -> Result<Self> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(path.to_string())?;
            
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        if contents.is_empty() {
            return Ok(Database::new("data/wal.log".to_string()))
        }

        let mut database : Result<Database> = match bincode::deserialize(&contents) {
            Ok(data) => Ok(data),
            Err(_) => Ok(Self::new(path.to_string())),
        };

        let logs = database.as_ref().unwrap().wal_manager.read_wal_log();

        if logs.is_some() {
            for log in logs.expect("wal.log is not empty") { 
                let operation = log.convert_to_operation();
                database.as_mut().unwrap().operate_db(operation);
            }
        }

        Ok(database?)
    }

    pub fn operate_db(&mut self, command: Command) -> Option<String>{
        match command {
            Command::INSERT(key, value) => self.insert(key, value),
            Command::GET(key) => self.get(key),
            Command::DELETE(key) => self.delete(key),
            Command::ERROR() => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;
    use std::fs;

    #[test]
    fn saving_and_loading() {
        let path = "data/database-test1.db";
        let _ = fs::remove_file(path).is_ok();

        let mut db = Database::new(path.to_string());
        db.insert("foo".to_string(), "bar".to_string());
        db.save_data(path).unwrap();

        // ensure its actually loading the data
        std::mem::drop(db);

        let db = Database::load_data(path).unwrap();
        let bar = db.get("foo".to_string()).unwrap();
        assert_eq!(bar, "bar" );
    }

    #[test]
    fn inserting_and_deleting() -> Result<(), String> {
        let path = "data/database-test2.db";
        let _ = fs::remove_file(path).is_ok();

        let mut db = Database::new(path.to_string());
        
        db.insert("woo".to_string(), "warr".to_string());
        db.save_data(path).unwrap();

        std::mem::drop(db);

        let mut db = Database::load_data(path).unwrap();
        db.delete("woo".to_string());
        match db.get("woo".to_string()) {
            Some(_)=> Err("Value was not deleted".to_string()),
            None => Ok(()),
        }
    }
}


