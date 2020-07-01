use std::fs;

use clap::{App, Arg};

mod markdown;
use markdown::*;

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("Markdown Parser")
        .version("1.0")
        .author("Hector Peeters")
        .about("Convert Markdown files into HTML!")
        .arg(Arg::with_name("input").required(true).index(1))
        .arg(Arg::with_name("header").short("h").takes_value(true))
        .arg(Arg::with_name("footer").short("f").takes_value(true))
        .get_matches();

    let header: String = match matches.value_of("header") {
        Some(x) => fs::read_to_string(x).expect("Failed to read header file"),
        None => String::from(include_str!("header.html")),
    };
    let footer: String = match matches.value_of("footer") {
        Some(x) => fs::read_to_string(x).expect("Failed to read footer file"),
        None => String::from(include_str!("footer.html")),
    };

    let markdown = fs::read_to_string(matches.value_of("input").unwrap())?;

    let mut parser: Parser = Parser::new(&markdown);

    let mut result = String::from(header);
    result.push_str(parser.to_html().as_str());
    result.push_str(&footer);

    fs::write("index.html", result)?;

    Ok(())
}
