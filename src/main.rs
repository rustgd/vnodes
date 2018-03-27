#![allow(dead_code)]

#[macro_use]
extern crate bitflags;

mod macros;

pub mod data;

mod alloc;

fn intern_byte(b: u8) -> u8 {
    match b {
        b'a'...b'z' => b - b'a',
        b'0'...b'9' => b - b'0' + 26,
        b'-' | b'_' => 36,
        b'.' => 37,
        _ => panic!(),
    }
}

fn intern(mut s: &[u8]) -> u64 {
    let mut result = 0;

    while let Some(&byte) = s.get(0) {
        result <<= 5;
        result |= intern_byte(byte) as u64;
        s = &s[1..];
    }

    result
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

struct Map {
    idents: Vec<u64>,
    nodes: Vec<Node>,
}

enum Node {
    Map(Map),
    None,
}

fn main() {
    let mut a = alloc::Allocator::new();

    let p_a;
    let p_b;
    let p_c;
    let p_d;
    let p_e;

    println!("-- Allocation 1 -- ");
    unsafe {
        a.debug();
        let mut allocate = |ident: &str| {
            println!("Allocating {}..", ident);
            let ptr = a.allocate(32);
            a.debug();

            ptr
        };

        p_a = allocate("a");
        p_b = allocate("b");
        p_c = allocate("c");
        p_d = allocate("d");
    }

    println!("-- De-allocation 1 -- ");
    unsafe {
        a.debug();
        let mut deallocate = |ptr: *mut (), ident: &str| {
            println!("De-allocating {}..", ident);
            a.deallocate(ptr, 32);
            a.debug();
        };

        deallocate(p_b, "b");
        deallocate(p_d, "d");
    }

    println!("-- Allocation 2 -- ");
    unsafe {
        a.debug();
        let mut allocate = |ident: &str| {
            println!("Allocating {}..", ident);
            let ptr = a.allocate(32);
            a.debug();

            ptr
        };

        p_e = allocate("e");
    }

    println!("-- De-allocation 1 -- ");
    unsafe {
        a.debug();
        let mut deallocate = |ptr: *mut (), ident: &str| {
            println!("De-allocating {}..", ident);
            a.deallocate(ptr, 32);
            a.debug();
        };

        deallocate(p_c, "c");
        deallocate(p_e, "e");
        deallocate(p_a, "a");
    }
}
