#[derive(Debug)]
pub struct Map<K, V> {
    data: Vec<(K, V)>,
}

impl<K, V> Map<K, V> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn from_iterator(it: impl Iterator<Item = (K, V)>) -> Self {
        Self { data: it.collect() }
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.data.push((k, v))
    }
}

impl<K: PartialEq, V> Map<K, V> {
    pub fn get(&self, k: &K) -> Option<&V> {
        self.data.iter().find(|(o, _)| o == k).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.data.iter_mut().find(|(o, _)| o == k).map(|(_, v)| v)
    }
}
