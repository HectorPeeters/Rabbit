// extern crate markdown;

// use markdown::{Block, ListItem, Span};
use std::fs;

mod markdown;
use markdown::*;

// use katex;
// use katex::OutputType;

// fn convert_span_html(span: &Span) -> String {
//     let mut result: String = String::default();

//     match span {
//         Span::Emphasis(children) => {
//             result.push_str("<em>");
//             for child in children {
//                 result.push_str(&convert_span_html(child));
//             }
//             result.push_str("</em>");
//         }
//         Span::Strong(children) => {
//             result.push_str("<strong>");
//             for child in children {
//                 result.push_str(&convert_span_html(child));
//             }
//             result.push_str("</strong>");
//         }
//         Span::Text(text) => {
//             result.push_str(&text);
//         }
//         Span::Link(title, url, unknown) => {
//             result.push_str(&format!("<a href=\"{}\">{}</a>", url, title));
//         }
//         Span::Image(title, url, unknown) => {
//             result.push_str(&format!("<img src=\"{}\" title=\"{}\">", url, title));
//         }
//         _ => println!("Unhandleded span {:?}", span),
//     }

//     result
// }

// fn convert_block_html(block: &Block) -> String {
//     let mut result: String = String::default();

//     match block {
//         Block::Header(children, size) => {
//             result.push_str(&format!("<h{}>", size));
//             for child in children {
//                 result.push_str(&convert_span_html(child));
//             }
//             result.push_str(&format!("</h{}>", size));
//         }
//         Block::Paragraph(children) => {
//             for child in children {
//                 result.push_str(&convert_span_html(child));
//             }
//             result += "\n";
//         }
//         Block::UnorderedList(children) => {
//             result.push_str("<ul>");
//             for child in children {
//                 result.push_str("<li>");
//                 match child {
//                     ListItem::Simple(span_children) => {
//                         for s in span_children {
//                             result.push_str(&convert_span_html(s));
//                         }
//                     }
//                     ListItem::Paragraph(paragraph_children) => {
//                         for p in paragraph_children {
//                             result.push_str(&convert_block_html(p));
//                         }
//                     }
//                 }
//                 result.push_str("</li>");
//             }
//             result.push_str("</ul>");
//         }
//         _ => println!("Unhandleded block {:?}", block),
//     }

//     result
// }

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
        MarkdownNode::Math(_math) => {
            result.push_str("<center>");

            // let opts = katex::Opts::builder().output_type(OutputType::Mathml).build().unwrap();
            // result.push_str(&katex::render_with_opts(&math, opts).unwrap());

            result.push_str("</center>");
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

    let markdown = fs::read_to_string("examples/test_code.md")?;

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
