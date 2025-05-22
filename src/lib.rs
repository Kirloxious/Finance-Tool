pub mod account;
pub mod app;
pub mod catergorization;
pub mod database;
pub mod parser;
pub mod transaction;
pub mod user;

use std::hash::{DefaultHasher, Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> i64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    (s.finish() & 0x0000_FFFF_FFFF_FFFF) as i64
}

pub trait Routable {
    fn route() -> &'static str;
}
