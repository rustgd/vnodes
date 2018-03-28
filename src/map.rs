use {Interned, Node, Value, Vnodes};

struct Map {
    idents: Vec<u64>,
    values: Vec<Value<'static>>,
}

impl Node for Map {
    fn call(&self, context: &Vnodes, args: &[Value]) -> Value {
        unimplemented!()
    }

    fn get(&self, context: &Vnodes, ident: Interned) -> Value {
        let index = search(ident.0, &self.idents).unwrap();

        self.values[index]
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
