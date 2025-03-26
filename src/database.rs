use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use std::option::Option;

use bincode;
use std::fs;
use std::io::Read;
use std::io::{Result, Write};

use crate::wal::WALManager;


#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    data: BTreeMap<String, String>,
    wal_manager: WALManager, 
}

impl Database {
    pub fn new(path: String) -> Database {
        Database { data: BTreeMap::new(), wal_manager: WALManager::new(path) } }
    
    pub fn insert(&mut self, key : String, value: String) -> Option<String> {
        self.data.insert(key, value)
    }

    pub fn get(&self, key : String) -> Option<String> {
        self.data.get(&key).cloned()
    }

    pub fn delete(&mut self, key: String) -> Option<String> {
        self.data.remove(&key)
    }

    pub fn save_data(&self, path: &str) -> std::io::Result<()> {
        fs::create_dir_all("data")?;
        let encoded : Vec<u8> = bincode::serialize(self).unwrap();
        let mut file = fs::File::create(path)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    pub fn load_data(path: &str) -> Result<Self> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .create(true)
            .open(path)
            .unwrap();

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        match bincode::deserialize(&contents) {
            Ok(data) => Ok(data),
            Err(_) => Ok(Self::new()),
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

        let mut db = Database::new();
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

        let mut db = Database::new();
        
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


