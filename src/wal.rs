use serde::{Serialize, Deserialize};
use std::fs;
use std::io::Write;
use bincode;
use std::io::Read;

use crate::parser::Command;


#[derive(Serialize, Deserialize)]
pub struct WALEntry {
    pub operation: String, 
    pub key: String,
    pub value: Option<String>,
}

impl WALEntry {

    pub fn new(operation: String, key: String, value: Option<String>) -> WALEntry {
        WALEntry {operation, key, value}
    }

    pub fn log(&self, path: &str) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        let encoded : Vec<u8> = bincode::serialize(self).unwrap();
        file.write_all(&encoded).unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WALManager {
    pub path: String,
}

impl WALManager {
    pub fn new(path : String) -> Self {
        WALManager{ path }
    }

    pub fn operate(&self, command: Command) -> Option<WALEntry> {
        let entry = match command {
            Command::INSERT(key, value) => Some(WALEntry::new("INSERT".to_string(), key, Some(value))),
            Command::GET(key) => Some(WALEntry::new("GET".to_string(), key, None)),
            Command::DELETE(key) => Some(WALEntry::new("DELETE".to_string(), key, None)),
            _ => None,
        };

        entry.as_ref().unwrap().log(self.path.as_str());
        entry
    }


    pub fn read_wal_log(&self) -> Option<Vec<WALEntry>> { 
        let mut log = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .open(self.path.as_str())
            .unwrap();

        let mut contents = Vec::new();
        log.read_to_end(&mut contents).unwrap();

        match bincode::deserialize(&contents) {
            Ok(data) => Some(data),
            Err(_) => None, 
        }
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
