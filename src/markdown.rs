use crate::parser::Parser;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

#[derive(Debug, PartialEq)]
pub enum MarkdownNode {
    Header(String, usize),
    Paragraph(Vec<ParagraphItem>, bool),
    List(Vec<MarkdownNode>),
    Math(String),
    Code(String, String),
    Table(Vec<MarkdownNode>, Vec<MarkdownNode>),
    PageBreak(),
}

#[derive(Debug, PartialEq)]
pub enum ParagraphItem {
    Text(String),
    Italic(String),
    Bold(String),
    Url(String, String),
    InlineMath(String),
    Image(String, String),
    InlineCode(String),
}

pub trait ToHtml {
    fn to_html(&self, base_path: &str, fast: bool) -> String;
}

impl ToHtml for ParagraphItem {
    fn to_html(&self, base_path: &str, fast: bool) -> String {
        match self {
            ParagraphItem::Text(text) => String::from(text),
            ParagraphItem::Italic(text) => format!("<em>{}</em>", text),
            ParagraphItem::Bold(text) => format!("<b>{}</b>", text),
            ParagraphItem::Url(name, url) => format!("<a href=\"{}\">{}</a>", url, name),
            ParagraphItem::InlineMath(math) => {
                if fast {
                    format!("${}$", math)
                } else {
                    tex_to_svg(math, true)
                }
            }
            ParagraphItem::Image(url, alt_text) => {
                if fast {
                    format!("<img src=\"{}\" alt=\"{}\">", url, alt_text)
                } else {
                    if url.contains("www.") || url.contains("http://") || url.contains("https://") {
                        return format!("<img src=\"{}\" alt=\"{}\">", url, alt_text);
                    }

                    let full_path = Path::new(base_path).join(url);

                    let mut f = File::open(&full_path).expect("no file found");
                    let metadata = fs::metadata(&full_path).expect("unable to read metadata");
                    let mut buffer = vec![0; metadata.len() as usize];
                    f.read_exact(&mut buffer).expect("buffer overflow");
                    let image_data = base64::encode(buffer);

                    let extension = Path::new(url)
                        .extension()
                        .and_then(OsStr::to_str)
                        .expect("Failed to get file extension");

                    format!(
                        "<img src=\"data:image/{};base64,{}\" alt=\"{}\">",
                        extension, image_data, alt_text
                    )
                }
            }
            ParagraphItem::InlineCode(code) => format!("<code>{}</code>", code),
        }
    }
}

fn tex_to_svg(input: &str, inline: bool) -> String {
    let mut command = Command::new("tex2svg");
    command.arg(input);

    if inline {
        command.arg("--inline");
    }

    match command.output() {
        Ok(x) => String::from_utf8(x.stdout).unwrap(),
        Err(_) => {
            eprintln!("Failed to parse math: ${}$", input);
            String::from("<center>MATH PARSING ERROR</center>")
        }
    }
}

impl ToHtml for MarkdownNode {
    fn to_html(&self, base_path: &str, fast: bool) -> String {
        match self {
            MarkdownNode::Header(text, level) => format!("<h{}>{}</h{}>", level, text, level),
            MarkdownNode::List(items) => {
                let mut result: String = String::default();
                result.push_str("<ul>");
                for node in items {
                    result.push_str(&format!("<li>{}</li>", node.to_html(base_path, fast)));
                }
                result.push_str("</ul>");
                result
            }
            MarkdownNode::Math(math) => {
                if fast {
                    format!("<center>${}$</center>", math)
                } else {
                    format!("<center>{}</center>", tex_to_svg(math, false))
                }
            }
            MarkdownNode::Code(lang, code) => {
                let ss = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
                let theme = &ts.themes["base16-ocean.dark"];

                let syntax = if lang.trim().is_empty() {
                    ss.find_syntax_plain_text()
                } else {
                    ss.find_syntax_by_token(&lang.to_lowercase())
                        .expect(&format!("Failed to load syntax for {}", lang))
                };
                let processed_code = code.replace("&lt;", "<").replace("&gt;", ">");

                highlighted_html_for_string(&processed_code, &ss, &syntax, theme)
            }
            MarkdownNode::Paragraph(children, single_line) => {
                let mut result: String = String::default();

                if !single_line {
                    result.push_str("<p>");
                }
                for child in children {
                    result.push_str(child.to_html(base_path, fast).as_str());
                }
                if !single_line {
                    result.push_str("</p>");
                }

                result
            }
            MarkdownNode::Table(headers, data) => {
                let mut header_html = String::default();

                for header in headers {
                    header_html += &format!("<th>{}</th>", header.to_html(base_path, fast));
                }

                let mut data_html = String::new();
                for i in 0..data.len() {
                    if i % headers.len() == 0 {
                        data_html += "<tr>";
                    }

                    data_html += &format!("<td>{}</td>", data[i].to_html(base_path, fast));

                    if i % headers.len() == headers.len() - 1 {
                        data_html += "</tr>";
                    }
                }

                format!(
                    "<table><tr>{}</tr><tr>{}</tr></table>",
                    header_html, data_html,
                )
            }
            MarkdownNode::PageBreak() => String::from("<p style=\"page-break-after: always;\"</p>"),
        }
    }
}

pub fn convert_to_html(path: &str, fast: bool) -> String {
    let mut result = String::new();

    let markdown = std::fs::read_to_string(path).unwrap();

    let mut parser = Parser::new(&markdown);

    for node in parser {
        result.push_str(node.to_html(path, fast).as_str())
    }

    result
}
