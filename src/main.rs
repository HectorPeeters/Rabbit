use std::fs;

mod markdown;
use markdown::*;

// use katex;
// use katex::OutputType;

fn to_html(node: MarkdownNode) -> String {
    let mut result = String::default();

    match node {
        MarkdownNode::Header(text, level) => {
            result.push_str(&format!("<h{}>{}</h{}>", level, text, level));
        }
        MarkdownNode::Paragraph(text) => {
            result.push_str(&format!("<p>{}</p>", text));
        }
        MarkdownNode::List(items) => {
            result.push_str("<ul>");
            for text in items {
                match text {
                    MarkdownListItem::ListItem(text) => {
                        result.push_str(&format!("<li>{}</li>", text));
                    }
                }
            }
            result.push_str("</ul>");
        }
        MarkdownNode::Math(math, mode) => {
            match mode {
                MathMode::NonInline => {
                    result.push_str("<center>");
                }
                _ => {}
            }
            
            result.push_str("$");
            result.push_str(&math);
            result.push_str("$");

            match mode {
                MathMode::NonInline => {
                    result.push_str("</center><br>");
                }
                _ => {}
            }
        }
        MarkdownNode::Code(lang, code) => {
            result.push_str(&format!("<pre><code class=\"{}\">", lang));
            result.push_str(&code);

            result.push_str("</code></pre>");
        }
    }

    result
}

fn main() -> Result<(), std::io::Error> {
    let header = include_str!("header.html");
    let footer = include_str!("footer.html");

    let markdown = fs::read_to_string("examples/test_math.md")?;

    let parser: Parser = Parser::new(&markdown);

    let mut result = String::from(header);

    for node in parser {
        let html = to_html(node);
        result.push_str(&html);
    }

    result.push_str(footer);

    fs::write("index.html", result)?;

    Ok(())
}
