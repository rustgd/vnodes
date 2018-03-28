use parking_lot::RwLock;

use {Interned, NodeHandle, NodeMut, Value, Vnodes};

#[derive(Default)]
pub struct Map {
    idents: Vec<u64>,
    values: Vec<Value<'static>>,
}

impl Map {
    pub fn new_node() -> NodeHandle {
        NodeHandle::new(RwLock::new(Map::default()))
    }
}

impl NodeMut for Map {
    fn call(&self, _: &Vnodes, _: &[Value]) -> Value<'static> {
        unimplemented!()
    }

    fn get(&self, _: &Vnodes, ident: Interned) -> Value<'static> {
        let index = search(ident.0, &self.idents).unwrap();

        self.values[index].clone()
    }

    fn set(&mut self, _: &Vnodes, ident: Interned, value: Value<'static>) {
        let key = ident.0;

        let index = {
            let len = self.idents.len();
            let idents = &self.idents;

            (0..len)
                .map(|i| idents[i])
                .position(|x| key < x)
                .unwrap_or(len)
        };

        self.idents.insert(index, key);
        self.values.insert(index, value);
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
