use serde::{Serialize, Deserialize};
use std::fs;
use std::io::Write;
use bincode;

use crate::parser::Command;


#[derive(Serialize, Deserialize)]
pub struct WALEntry {
    operation: String, 
    key: String,
    value: Option<String>,
}

impl WALEntry {

    fn new(operation: String, key: String, value: Option<String>) -> WALEntry {
        WALEntry {operation, key, value}
    }

    fn log(path: &str, entry: &WALEntry) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        let encoded : Vec<u8> = bincode::serialize(entry).unwrap();
        file.write_all(&encoded).unwrap();
    }

    pub fn operate(command: Command) -> Option<Self> {
        let entry = match command {
            Command::INSERT(key, value) => Some(WALEntry::new("INSERT".to_string(), key, Some(value))),
            Command::GET(key) => Some(WALEntry::new("GET".to_string(), key, None)),
            Command::DELETE(key) => Some(WALEntry::new("DELETE".to_string(), key, None)),
            _ => None,
        };

        WALEntry::log("data/wal.log", &entry.as_ref().unwrap());
        entry
    }
}


#[cfg(test)]
mod tests {

    use crate::wal::WALEntry;
    use crate::parser::Command;

    #[test]
    fn wal_log() {
        WALEntry::operate(Command::INSERT("foo".to_string(), "bar".to_string()));
    }
}
