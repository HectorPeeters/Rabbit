use std::fs;

mod markdown;
use markdown::*;

fn main() -> Result<(), std::io::Error> {
    let header = include_str!("header.html");
    let footer = include_str!("footer.html");

    let markdown = fs::read_to_string("examples/test_math.md")?;

    let parser: Parser = Parser::new(&markdown);

    let mut result = String::from(header);

    for node in parser {
        let html = node.to_html().expect("Failed to convert node to html");
        result.push_str(&html);
    }

    result.push_str(footer);

    fs::write("index.html", result)?;

    Ok(())
}
