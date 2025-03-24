use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use bincode;
use std::fs::File;
use std::io::{Result, Write};


#[derive(Serialize, Deserialize, Debug, bincode::Encode, bincode::Decode)]
pub struct Database {
    data: BTreeMap<String, String>,
}

impl Database {
    pub fn new() -> Database {
        Database { data: BTreeMap::new() } }
    
    pub fn insert(&mut self, key : String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key : String) {
        self.data.get(&key);
    }

    pub fn delete(&mut self, key: String) {
        self.data.remove(&key);
    }

    pub fn error(&self) {
    }

    fn save_data(&self, path: &str) -> std::io::Result<()> {
        let config = bincode::config::standard()
            .with_big_endian()
            .with_fixed_int_encoding();
        let encoded : Vec<u8> = bincode::encode_to_vec(self, config).unwrap();
        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    fn load_data(path: &str) -> Result<Self> {


        let file = File::open(path)?;
        let config = bincode::config::standard()
            .with_big_endian()
            .with_fixed_int_encoding();

        let data: Database = bincode::decode_from_reader(file, config).unwrap();

        Ok(data)
    }
}
