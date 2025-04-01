use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

use crate::wal::WALEntry;



#[derive(Serialize, Deserialize, Debug)]
pub struct Collection { 
    data: BTreeMap<String, String>,
    pub name: String,
}

impl Collection {
    pub fn new(name: String) -> Collection {
        Collection{ data: BTreeMap::new(), name }
    }
    pub fn insert(&mut self, key : String, value: String) -> Option<String> {
        self.data.insert(key, value)
    }

    pub fn get(&self, key : String) -> Option<String> {
        self.data.get(&key).cloned()
    }

    pub fn delete(&mut self, key: String) -> Option<String> {
        self.data.remove(&key)
    }
}
