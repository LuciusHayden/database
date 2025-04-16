use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection { 
    data: Map<String, Value>,
    pub name: String,
}

impl Collection {
    pub fn new(name: String) -> Collection {
        Collection{ data: Map::new(), name }
    }
    pub fn insert(&mut self, key : String, value: Value) -> Option<Value> {
        self.data.insert(key, value)
    }

    pub fn get(&self, key : String) -> Option<Value> {
        self.data.get(&key).cloned()
    }

    pub fn delete(&mut self, key: String) -> Option<Value> {
        self.data.remove(&key)
    }
}
