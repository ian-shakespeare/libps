use std::io;

use libps::scan;

fn main() {
    let input = io::stdin();
    let mut buf = String::new();
    if input.read_line(&mut buf).is_err() {
        panic!("Failed to read input.");
    }

    scan(&buf);
}
