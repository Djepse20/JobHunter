pub mod Job_query;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::ptr::hash;

use serde::de::value;
// #[tokio::main]
// async fn main() {}
#[derive(Debug, PartialEq)]
struct Type;
struct Value;
struct Environment<T>(Vec<HashMap<String, T>>);

impl<T> Environment<T> {
    fn new() -> Self {
        Environment(vec![HashMap::new()])
    }

    fn get(&self, key: &String) -> Option<&T> {
        for map in &self.0 {
            if let Some(value) = map.get(key) {
                return Some(value);
            }
        }
        None
    }

    fn insert(&mut self, key: String, value: T) {
        for map in &mut self.0 {
            if map.contains_key(&key) {
                map.insert(key, value);
                return;
            }
        }

        self.0.last_mut().unwrap().insert(key, value);
    }

    fn append_scope(&mut self) {
        self.0.push(HashMap::new());
    }

    fn remove_scope(&mut self) {
        self.0.pop();
    }

    fn decl_var(&mut self, key: String, value: T) {
        if self.get(&key).is_some() {
            panic!("Redeclaration of variable '{key}' not allowed!");
        }

        self.insert(key, value);
    }

    fn decl_func(&mut self, key: String, value: T) {
        if self.get(&key).is_some() {
            panic!("Redeclaration of function '{key}' not allowed!");
        }

        self.insert(key, value);
    }
}

impl Environment<Value> {
    fn assign_var(&mut self, key: String, value: Value) {
        if self.get(&key).is_none() {
            panic!("Can't assign value to undeclared variable '{key}'");
        }

        self.insert(key, value);
    }
}

impl Environment<Type> {
    fn assign_var(&mut self, key: String, value: Type) {
        match self.get(&key) {
            Some(val) if *val == value => {
                self.insert(key, value);
            }
            Some(_) => {
                panic!("Can't assign value of type '{value:?}' to variable '{key}'!");
            }
            None => panic!("Can't assign value to undeclared variable '{key}'"),
        }
    }
}

trait IntoBytes {
    fn into_bytes(&self) -> &[u8];
}
impl IntoBytes for String {
    fn into_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl IntoBytes for str {
    fn into_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

struct Mappu<K, V> {
    buckets: Vec<Option<(K, V)>>,
}

fn fnv1(bytes: &[u8], size: usize) -> usize {
    const FNV_OFFSET_BASIS: usize = 0xcbf29ce484222325;
    const FNV_PRIME: usize = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash = hash.wrapping_mul(FNV_PRIME);
        hash ^= usize::from(*byte);
    }
    hash % size
}
impl<K: IntoBytes + PartialEq + Eq + Clone, V: Clone> Mappu<K, V> {
    pub fn new() -> Self {
        let buckets = vec![None::<(K, V)>; 200];

        Mappu { buckets }
    }
    fn fnv1(&self, bytes: &[u8]) -> usize {
        const FNV_OFFSET_BASIS: usize = 0xcbf29ce484222325;
        const FNV_PRIME: usize = 0x100000001b3;

        let mut hash = FNV_OFFSET_BASIS;
        for byte in bytes {
            hash = hash.wrapping_mul(FNV_PRIME);
            hash ^= usize::from(*byte);
        }
        hash % self.buckets.len()
    }
    pub fn insert(&mut self, key: K, value: V) {
        let key_bytes = key.into_bytes();
        let idx = self.fnv1(key_bytes);
        if self.buckets[idx].is_some() {
            panic!("alreay taken");
        }
        self.buckets[idx] = Some((key, value));
    }
    pub fn get<Q>(&self, key: &Q) -> &V
    where
        Q: ?Sized + IntoBytes,
        K: Borrow<Q>,
    {
        let key_bytes = key.borrow().into_bytes();
        let idx = self.fnv1(key_bytes);
        if let Some((_, v)) = &self.buckets[idx] {
            return v;
        }
        panic!("key not in map")
    }
}

fn main() {
    let mut map: Mappu<String, u32> = Mappu::new();
    map.insert("sus".to_string(), 20);
    map.insert("haha".to_string(), 10);
    map.insert("ggg".to_string(), 11);

    map.insert("fff".to_string(), 40);
    println!("{}", map.get("sus"));
    println!("{}", map.get("haha"));

    println!("{}", map.get("ggg"));
}
