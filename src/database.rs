use std::collections::{HashMap, HashSet, LinkedList};

pub struct Database {
    elements: HashMap<String, TypeSaved>,
}

#[derive(Debug, PartialEq)]
pub enum TypeSaved {
    String(String),
    List(LinkedList<String>),
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
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
