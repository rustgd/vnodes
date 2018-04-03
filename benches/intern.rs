#![feature(test)]

extern crate fnv;
extern crate test;
extern crate vnodes;

use fnv::FnvHashMap;
use test::{black_box, Bencher};
use vnodes::{Interned, InternedMap};

const KEYS: &[&str] = &[
    "abc", "whatever", "hello", "okay", "makes", "sense", "w1th", "numb3rs", "s0m3", "m0r3"
];

fn all_keys() -> Vec<String> {
    KEYS.iter()
        .flat_map(|&key| (0..50).map(move |nr| format!("{}{}", key, nr)))
        .collect()
}

#[bench]
fn bytes_into_interned(b: &mut Bencher) {
    b.iter(|| {
        black_box(Interned::from(black_box(b"mystring" as &[u8])));
    });
}

#[bench]
fn string_into_interned(b: &mut Bencher) {
    b.iter(|| {
        black_box(Interned::from(black_box("mystring")));
    });
}

#[bench]
fn get_interned(b: &mut Bencher) {
    let interned = Interned::from("hello");
    let mut map = InternedMap::new();
    for key in all_keys() {
        map.insert(Interned::from(&key as &str), 22);
    }

    b.iter(|| {
        black_box(map.get(black_box(interned)));
    });
}

#[bench]
fn get_conv_interned(b: &mut Bencher) {
    let mut map = InternedMap::new();
    for key in all_keys() {
        map.insert(Interned::from(&key as &str), 22);
    }

    b.iter(|| {
        let key = Interned::from(black_box("hello"));
        black_box(map.get(black_box(key)));
    });
}

#[bench]
fn get_string(b: &mut Bencher) {
    let string = "hello";
    let mut map = FnvHashMap::default();
    for key in all_keys() {
        map.insert(key, 22);
    }

    b.iter(|| {
        black_box(map.get(black_box(string)));
    });
}
