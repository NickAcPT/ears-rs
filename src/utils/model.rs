use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rectangle {
    pub(crate) x1: u32,
    pub(crate) y1: u32,

    pub(crate) x2: u32,
    pub(crate) y2: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AlfalfaData {
    pub(crate) version: u8,
    pub(crate) data: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlfalfaDataKey {
    Erase,
    Cape,
    Wings,
    Custom(&'static str),
}

impl From<AlfalfaDataKey> for &'static str {
    fn from(value: AlfalfaDataKey) -> Self {
        match value {
            AlfalfaDataKey::Erase => "erase",
            AlfalfaDataKey::Cape => "cape",
            AlfalfaDataKey::Wings => "wing",
            AlfalfaDataKey::Custom(key) => key,
        }
    }
}

impl AlfalfaData {
    pub fn new() -> Self {
        Self {
            version: 1,
            data: HashMap::new(),
        }
    }

    pub(crate) fn get_data_internal(&self, key: &'static str) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    pub fn get_data(&self, key: AlfalfaDataKey) -> Option<&[u8]> {
        self.get_data_internal(key.into())
    }
    
    pub(crate) fn set_data_internal(&mut self, key: &'static str, value: Vec<u8>) {
        self.data.insert(key.to_owned(), value);
    }
    
    pub fn set_data(&mut self, key: AlfalfaDataKey, value: Vec<u8>) {
        self.set_data_internal(key.into(), value);
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
