use std::collections::HashMap;

#[derive(Debug)]
pub struct OrderedHashMap<K, V> {
    map: HashMap<K, V>,
    keys: Vec<K>,
}

impl<K: std::hash::Hash + Eq + Clone, V> Default for OrderedHashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: std::hash::Hash + Eq + Clone, V> OrderedHashMap<K, V> {
    pub fn new() -> Self {
        OrderedHashMap {
            map: HashMap::new(),
            keys: Vec::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if !self.map.contains_key(&key) {
            self.keys.push(key.clone());
        }
        self.map.insert(key, value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys
            .iter()
            .filter_map(move |key| self.map.get(key).map(|value| (key, value)))
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}
