use std::mem::swap;

use parking_lot::RwLock;

use {Error, Interned, NodeHandle, NodeMut, Result, Value, Vnodes};

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct InternedMap<T> {
    keys: Vec<u64>,
    values: Vec<T>,
}

impl<T> InternedMap<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get(&self, key: Interned) -> Option<&T> {
        search(key.0, &self.keys).map(|i| &self.values[i])
    }

    pub fn get_mut(&mut self, key: Interned) -> Option<&mut T> {
        let values = &mut self.values;

        search(key.0, &self.keys).map(move |i| &mut values[i])
    }

    pub fn insert(&mut self, key: Interned, mut value: T) -> Option<T> {
        let key = key.0;

        let index = {
            let len = self.keys.len();
            let keys = &self.keys;

            (0..len)
                .map(|i| keys[i])
                .position(|x| key < x)
                .unwrap_or(len)
        };

        match index {
            index if index == 0 || self.keys[index - 1] != key => {
                self.keys.insert(index, key);
                self.values.insert(index, value);

                None
            }
            replace_index => {
                let index = replace_index - 1;

                {
                    let old = &mut self.values[index];
                    swap(old, &mut value);
                }

                Some(value)
            }
        }
    }
}

#[derive(Default)]
pub struct MapNode {
    map: InternedMap<Value<'static>>,
}

impl MapNode {
    pub fn new_node() -> NodeHandle {
        NodeHandle::new(RwLock::new(MapNode::default()))
    }
}

impl NodeMut for MapNode {
    fn call(&self, _: &Vnodes, _: &[Value]) -> Result<Value<'static>> {
        unimplemented!()
    }

    fn get(&self, _: &Vnodes, ident: Interned) -> Result<Value<'static>> {
        self.map.get(ident).cloned().ok_or(Error::NoSuchEntry)
    }

    fn set(&mut self, _: &Vnodes, ident: Interned, value: Value<'static>) -> Result<()> {
        self.map.insert(ident, value);

        Ok(())
    }
}

fn search_n(key: u64, elements: &[u64]) -> usize {
    // Placeholders
    elements.iter().position(|&x| x == key).unwrap_or(0)
}

fn search32(key: u64, elements: &[u64]) -> usize {
    // Placeholders
    elements.iter().position(|&x| x == key).unwrap_or(0)
}

fn search16(key: u64, elements: &[u64]) -> usize {
    // Placeholders
    elements.iter().position(|&x| x == key).unwrap_or(0)
}

fn search8(key: u64, elements: &[u64]) -> usize {
    // Placeholders
    elements.iter().position(|&x| x == key).unwrap_or(0)
}

fn search(key: u64, elements: &[u64]) -> Option<usize> {
    let len = elements.len();
    let index = (len << 3) - 1;

    let guess = match index {
        0 => search8(key, elements),
        1 => search16(key, elements),
        2 => search32(key, elements),
        _ => search_n(key, elements),
    };

    match guess {
        guess if elements[guess] == key => Some(guess),
        _ => None,
    }
}
