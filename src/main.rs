use std::io::{self, Read};

mod markdown;
use markdown::*;
mod parser;
use parser::*;

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    let mut stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_to_string(&mut buffer)?;

    let parser = Parser::new(&buffer);

    let mut html = String::new();

    for node in parser {
        html += &node.to_html();
    }

    println!("{}", html);
    Ok(())
}
