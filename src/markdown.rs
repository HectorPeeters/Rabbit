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
    Paragraph(Vec<ParagraphItem>),
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
    fn to_html(&self) -> String;
}

impl ToHtml for ParagraphItem {
    fn to_html(&self) -> String {
        match self {
            ParagraphItem::Text(text) => String::from(text),
            ParagraphItem::Italic(text) => format!("<em>{}</em>", text),
            ParagraphItem::Bold(text) => format!("<b>{}</b>", text),
            ParagraphItem::Url(name, url) => format!("<a href=\"{}\">{}</a>", url, name),
            ParagraphItem::InlineMath(math) => {
                format!("${}$", math)
            }
            ParagraphItem::InlineCode(code) => format!("<code>{}</code>", code),
            _ => unimplemented!(),
        }
    }
}


impl ToHtml for MarkdownNode {
    fn to_html(&self) -> String {
        match self {
            MarkdownNode::Header(text, level) => format!("<h{}>{}</h{}>", level, text, level),
            MarkdownNode::List(items) => {
                let mut result: String = String::default();
                result.push_str("<ul>");
                for node in items {
                    result.push_str(&format!("<li>{}</li>", node.to_html()));
                }
                result.push_str("</ul>");
                result
            }
            MarkdownNode::Math(math) => {
                format!("<center>${}$</center>", math)
            }
            MarkdownNode::Code(lang, code) => {
                let ss = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
                let theme = &ts.themes["base16-ocean.dark"];

                let syntax = if lang.trim().is_empty() {
                    ss.find_syntax_plain_text()
                } else {
                    ss.find_syntax_by_token(&lang.to_lowercase())
                        .unwrap_or_else(|| panic!("Failed to load syntax for {}", lang))
                };
                let processed_code = code.replace("&lt;", "<").replace("&gt;", ">");

                highlighted_html_for_string(&processed_code, &ss, &syntax, theme)
            }
            MarkdownNode::Paragraph(children) => {
                let mut result: String = String::default();

                result.push_str("<p>");
                for child in children {
                    result.push_str(child.to_html().as_str());
                }
                result.push_str("</p>");

                result
            }
            MarkdownNode::Table(headers, data) => {
                let mut header_html = String::default();

                for header in headers {
                    header_html += &format!("<th>{}</th>", header.to_html());
                }

                let mut data_html = String::new();
                for (i, x) in data.iter().enumerate() {
                    if i % headers.len() == 0 {
                        data_html += "<tr>";
                    }

                    data_html += &format!("<td>{}</td>", x.to_html());

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
