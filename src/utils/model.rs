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
    pub(crate) fn get_data_internal(&self, key: &'static str) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }
    
    pub fn get_data(&self, key: AlfalfaDataKey) -> Option<&[u8]> {
        self.get_data_internal(key.into())
    }
}