use std::collections::{HashMap, HashSet};

pub struct Database {
    elements: HashMap<String, TypeSaved>,
}

#[derive(Debug, PartialEq)]
pub enum TypeSaved {
    String(String),
    List(Vec<String>),
    Set(HashSet<String>),
}

impl Database {
    pub fn new() -> Self {
        Database {
            elements: HashMap::new(),
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<TypeSaved> {
        self.elements.remove(key)
    }

    pub fn insert(&mut self, key: String, value: TypeSaved) -> Option<TypeSaved> {
        self.elements.insert(key, value)
    }

    pub fn get(&mut self, key: &str) -> Option<&TypeSaved> {
        self.elements.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TypeSaved> {
        self.elements.get_mut(key)
    }

    // ojo => es Iter de HashMaps
    /* pub fn iter(&self) -> Iter<'_, String, TypeSaved>{
        self.elements.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, String, TypeSaved>{
        self.elements.iter_mut()
    }

    pub fn contain_key(&self, key: &str) -> bool {
        self.elements.contains_key(key)
    }

    pub fn clear(&mut self){
        self.elements.clear()
    }*/
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
