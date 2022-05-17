use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    println!("Starting the game");
    let mut buffer = String::new();
    let stdin = io::stdin();
    loop {
        let mut handle = stdin.lock();

        handle.read_line(&mut buffer)?;
        println!("{}", buffer);
        buffer = String::new();
    }
}
