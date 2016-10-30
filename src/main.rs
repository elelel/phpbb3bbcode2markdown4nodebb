#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
#[macro_use]
extern crate lazy_static;

mod bbcode;
mod nodebb;

use nodebb::NodeBB;

fn main() {
    let mut nodebb = NodeBB::new("localhost", &27017, "forum");
    nodebb.convert_bbcode("^post", "content");
}
