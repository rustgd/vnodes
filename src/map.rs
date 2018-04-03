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

    #[inline]
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

unsafe fn search_n(key: u64, elements: &[u64]) -> usize {
    let len = elements.len();
    let check = len - 1;
    // 2^(exp-1) < len <= 2^exp
    let exp = (0..).map(|s| check >> s).take_while(|&n| n > 0).count();
    let half = 1 << (exp - 1);

    let mut exp_counter = exp - 2;
    let mut ret = 0;
    ret += if key >= get(elements, half) { len - half } else { 0 };
    loop {
        let pow = 1 << exp_counter;
        ret += if key >= get(elements, ret + pow) { pow } else { 0 };

        if exp_counter == 0 {
            break;
        }
        exp_counter -= 1;
    }

    ret
}

unsafe fn search32(key: u64, elements: &[u64]) -> usize {
    let len = elements.len();
    let first = len - 16;

    let mut ret = if key >= get(elements, 16) { first } else { 0 };
    ret += if key >= get(elements, ret + 8) { 8 } else { 0 };
    ret += if key >= get(elements, ret + 4) { 4 } else { 0 };
    ret += if key >= get(elements, ret + 2) { 2 } else { 0 };
    ret += if key >= get(elements, ret + 1) { 1 } else { 0 };

    ret
}

unsafe fn search16(key: u64, elements: &[u64]) -> usize {
    let len = elements.len();
    let first = len - 8;

    let mut ret = if key >= get(elements, 8) { first } else { 0 };
    ret += if key >= get(elements, ret + 4) { 4 } else { 0 };
    ret += if key >= get(elements, ret + 2) { 2 } else { 0 };
    ret += if key >= get(elements, ret + 1) { 1 } else { 0 };

    ret
}

unsafe fn search8(key: u64, elements: &[u64]) -> usize {
    let len = elements.len();
    let first = len - 4;

    let mut ret = if key >= get(elements, 4) { first } else { 0 };
    ret += if key >= get(elements, ret + 2) { 2 } else { 0 };
    ret += if key >= get(elements, ret + 1) { 1 } else { 0 };

    ret
}

fn search(key: u64, elements: &[u64]) -> Option<usize> {
    let len = elements.len();

    let guess = unsafe {
        match len {
            0...4 => elements.iter().position(|&e| e == key).unwrap_or(0),
            5...8 => search8(key, elements),
            9...16 => search16(key, elements),
            17...32 => search32(key, elements),
            _ => search_n(key, elements),
        }
    };

    match guess {
        guess if elements[guess] == key => Some(guess),
        _ => None,
    }
}

#[inline(always)]
unsafe fn get(elements: &[u64], i: usize) -> u64 {
    *elements.get_unchecked(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn check_insert_get() {
        const KEYS: &[&str] = &[
            "abc", "whatever", "hello", "okay", "makes", "sense", "w1th", "numb3rs", "s0m3", "m0r3"
        ];

        let mut cmp: HashMap<_, _> = HashMap::default();
        let mut map = InternedMap::new();

        for (i, &key) in KEYS.iter().enumerate() {
            let interned = Interned::from(key);
            map.insert(interned, i);
            cmp.insert(interned, i);
        }

        for (key, value) in cmp {
            assert_eq!(map.get(key), Some(&value));
        }
    }
}
