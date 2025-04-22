use serde::{Serialize, Deserialize};

use std::{
    fs,
    io::{Write, Read},
};

use bincode::deserialize_from;
use serde_json::Value;
use std::io::BufReader;

use crate::parser::Command;
use crate::database::Database;
use crate::errors::DatabaseError;

#[derive(Serialize, Deserialize, Debug)]
pub struct WALEntry {
    pub collection: String,
    pub operation: String, 
    pub key: String,
    // needs to be a String as bincode doesnt work with Json values
    pub value: Option<String>,
}

impl WALEntry {

    pub fn new(collection: String, operation: String, key: String, value: Option<Value>) -> WALEntry {
        WALEntry {collection, operation, key, value: serde_json::to_string(&value.unwrap()).ok() }
    }

    pub fn log(&self, path: &str) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        bincode::serialize_into(&mut file, self).unwrap();
    }

    pub fn convert_to_operation(&self) -> Command {
        println!("{}", serde_json::from_str::<Value>(&self.value.as_ref().unwrap()).unwrap());
        match self.operation.as_str() {
            "INSERT" => Command::INSERT(self.key.to_string(), serde_json::from_str(&self.value.as_ref().unwrap()).unwrap()),
            "GET" => Command::GET(self.key.to_string()),
            "DELETE" => Command::DELETE(self.key.to_string()),
            _ => Command::ERROR(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WALManager {
    pub path: String,
}

impl WALManager {
    pub fn new(path : String) -> Self {
        let _ = fs::File::create_new(format!("{}/wal.log", &path));

        WALManager{ path }
    }

    pub fn operate(&self, command: Command, collection: String) -> Option<WALEntry> {
        let entry = match command {
            Command::INSERT(key, value) => Some(WALEntry::new(collection, "INSERT".to_string(), key, Some(value))),
            Command::GET(key) => Some(WALEntry::new(collection, "GET".to_string(), key, None)),
            Command::DELETE(key) => Some(WALEntry::new(collection, "DELETE".to_string(), key, None)),
            _ => None,
        };

        entry.as_ref().unwrap().log(self.path.as_str());
        entry
    }

    pub fn read_wal_log(&self) -> Result<Vec<WALEntry>, DatabaseError> { 
        let log = fs::OpenOptions::new()
            .read(true)
            .open(format!("{}/wal.log", self.path))
            .unwrap();


        let mut contents = BufReader::new(log);
        let mut entries = Vec::new();

        while let Ok(entry) = deserialize_from::<_, WALEntry>(&mut contents) {
            entries.push(entry);
        }

        Ok(entries)

    }
}

#[cfg(test)]
mod tests {

    use crate::wal::WALEntry;
    use crate::parser::Command;

    #[test]
    fn wal_log() {
    }
}
