use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};
use serde::{Serializer, Deserializer};
use serde::ser::SerializeMap;
use serde::de::{self, Visitor, MapAccess};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Collection { 
     #[serde(
        serialize_with = "map_value_to_string",
        deserialize_with = "map_string_to_value"
    )]
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

// This mess is because bincode cannot work with Json Values (YAY) so it must be converted into
// string values when being stores on the disk, when deserializing it converts back to being a Json
// Value 

// Serialize: Map<String, Value> → BTreeMap<String, String>
fn map_value_to_string<S>(
    map: &Map<String, Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ser_map = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        ser_map.serialize_entry(k, &v.to_string())?;
    }
    ser_map.end()
}

// Deserialize: BTreeMap<String, String> → Map<String, Value>
fn map_string_to_value<'de, D>(
    deserializer: D,
) -> Result<Map<String, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringToValueVisitor;

    impl<'de> Visitor<'de> for StringToValueVisitor {
        type Value = Map<String, Value>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map of stringified JSON values")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = Map::new();
            while let Some((k, v)) = access.next_entry::<String, String>()? {
                let val: Value = serde_json::from_str(&v)
                    .map_err(de::Error::custom)?;
                map.insert(k, val);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(StringToValueVisitor)
}

