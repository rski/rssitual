use std::io;

use rss::{Channel, Item};

fn main() {
    let mut ar = Item::default();
    ar.set_author(String::from("foo"));

    let mut ch = Channel::default();
    ch.set_items(vec![ar]);
    ch.pretty_write_to(io::stdout(), b' ', 2).unwrap();
}
