use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub enum MarkdownNode {
    Header(String, usize),
    Paragraph(Vec<ParagraphItem>, bool),
    List(Vec<MarkdownNode>),
    Math(String),
    Code(String, String),
    Table(Vec<MarkdownNode>, Vec<MarkdownNode>),
    PageBreak(),
}

#[derive(Debug)]
pub enum ParagraphItem {
    Text(String),
    Italic(String),
    Bold(String),
    Url(String, String),
    InlineMath(String),
    Image(String, String),
    InlineCode(String),
}

pub struct Parser<'a> {
    data: Vec<&'a str>,
    index: usize,
}

pub trait ToHtml {
    fn to_html(&self, base_path: &Path, fast: bool) -> String;
}

impl ToHtml for ParagraphItem {
    fn to_html(&self, base_path: &Path, fast: bool) -> String {
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
                    if url.contains("www.") || url.contains("http://") {
                        return format!("<img src=\"{}\" alt=\"{}\">", url, alt_text);
                    }

                    let mut f = File::open(base_path.join(url)).expect("no file found");
                    let metadata =
                        fs::metadata(base_path.join(url)).expect("unable to read metadata");
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
    fn to_html(&self, base_path: &Path, fast: bool) -> String {
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

fn is_whitespace(string: String) -> bool {
    string == " " || string == "\t"
}

fn is_newline(string: String) -> bool {
    string == "\r\n" || string == "\n"
}

fn preprocess_html(string: String) -> String {
    string.replace("<", "&lt;").replace(">", "&gt;")
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser {
            data: UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>(),
            index: 0,
        }
    }

    fn eof(&mut self) -> bool {
        self.index >= self.data.len()
    }

    fn peek(&mut self, index: usize) -> String {
        self.data[self.index + index].to_owned()
    }

    fn consume(&mut self) -> &str {
        let result = self.data[self.index];
        self.index += 1;
        result
    }

    fn go_back(&mut self, count: usize) {
        self.index -= count;
    }

    fn consume_chars(&mut self, character: &str) -> String {
        let mut result = String::default();

        loop {
            if self.eof() {
                break;
            }

            let c = self.peek(0);

            if c != character {
                break;
            }

            result.push_str(self.consume());
        }

        result
    }

    fn consume_until(&mut self, f: fn(String) -> bool) -> String {
        let mut result = String::default();

        loop {
            if self.eof() {
                break;
            }

            let c = self.peek(0);

            if f(c) {
                break;
            }

            result.push_str(self.consume());
        }

        result
    }

    fn skip_whitespace(&mut self) {
        self.consume_until(|c| c != " " && c != "\t" && c != "\n" && c != "\r\n");
    }

    fn parse_header(&mut self) -> Option<MarkdownNode> {
        let hashtags = self.consume_chars("#");

        if !is_whitespace(self.peek(0)) {
            self.go_back(hashtags.len());
            return None;
        }

        self.consume();
        let header_name = String::from(self.consume_until(is_newline).trim());

        Some(MarkdownNode::Header(header_name, hashtags.len()))
    }

    fn parse_list(&mut self) -> Option<MarkdownNode> {
        let mut nodes: Vec<MarkdownNode> = Vec::new();

        if !is_whitespace(self.peek(1)) {
            return None;
        }

        while !self.eof() && (self.peek(0) == "*" || self.peek(0) == "-") {
            self.consume();
            nodes.push(self.parse_paragraph(true).expect("Failed to parse in list"));
            self.skip_whitespace();
        }

        Some(MarkdownNode::List(nodes))
    }

    fn parse_math(&mut self) -> Option<MarkdownNode> {
        self.consume_chars("$");

        let math = String::from(self.consume_until(|c| c == "$").trim());
        self.consume_until(|c| c != "$");

        Some(MarkdownNode::Math(math))
    }

    fn parse_url(&mut self) -> ParagraphItem {
        self.consume();

        let url = self.consume_until(|c| c == ">");
        let name = url.clone();

        self.consume();

        ParagraphItem::Url(name, url)
    }

    fn parse_code(&mut self) -> Option<MarkdownNode> {
        if self.consume_chars("`").len() != 3 {
            return None;
        }

        let lang = if !is_newline(self.peek(0)) {
            self.consume_until(is_newline).trim().to_lowercase()
        } else {
            String::default()
        };

        let mut code = String::from(self.consume_until(|c| c == "`").trim());

        if lang == "html" {
            code = preprocess_html(code);
        }
        self.consume_chars("`");

        Some(MarkdownNode::Code(lang, code))
    }

    fn parse_named_url(&mut self) -> ParagraphItem {
        self.consume();

        let name = self.consume_until(|c| c == "]");

        self.consume();

        let mut url = String::default();

        if self.peek(0) == "(" {
            self.consume();
            url = self.consume_until(|c| c == ")");

            self.consume();
        }

        ParagraphItem::Url(name, url)
    }

    fn parse_image(&mut self) -> ParagraphItem {
        self.consume();
        self.consume();

        let alt_text = self.consume_until(|c| c == "]");

        self.consume();
        self.consume();

        let url = self.consume_until(|c| c == ")");
        self.consume();

        ParagraphItem::Image(url, alt_text)
    }

    fn parse_paragraph(&mut self, single_line: bool) -> Option<MarkdownNode> {
        let mut result: Vec<ParagraphItem> = Vec::new();

        loop {
            if self.eof() {
                break;
            }

            //     self.consume_until(|c| !is_whitespace(c));

            let curr = self.peek(0);

            if curr == "\n" || curr == "\r\n" {
                break;
            }

            let child = match curr.as_str() {
                "*" => {
                    let stars = self.consume_until(|c| c != "*").len();
                    let text = self.consume_until(|c| c == "*");
                    self.consume_chars("*");

                    if stars == 1 {
                        ParagraphItem::Italic(text)
                    } else {
                        ParagraphItem::Bold(text)
                    }
                }
                "_" => {
                    self.consume();
                    let text = if single_line {
                        self.consume_until(|c| c == "_" || is_newline(c))
                    } else {
                        self.consume_until(|c| c == "_")
                    };
                    self.consume();

                    ParagraphItem::Italic(text)
                }
                "$" => {
                    self.consume();
                    let text = self.consume_until(|c| c == "$");
                    self.consume();
                    ParagraphItem::InlineMath(text)
                }
                "<" => self.parse_url(),
                "[" => self.parse_named_url(),
                "!" => self.parse_image(),
                "`" => {
                    self.consume();
                    let code = self.consume_until(|c| c == "`");
                    self.consume();
                    ParagraphItem::InlineCode(code)
                }
                _ => {
                    let text = self.consume_until(|c| {
                        c == "_"
                            || c == "<"
                            || c == "*"
                            || c == "$"
                            || c == "["
                            || c == "`"
                            || c == "|"
                            || is_newline(c)
                    });
                    //TODO: trim text here
                    ParagraphItem::Text(text)
                }
            };

            result.push(child);

            //TODO: dirty fix this should be changed
            if single_line && self.peek(0) == "|" {
                break;
            }
        }

        Some(MarkdownNode::Paragraph(result, single_line))
    }

    fn parse_table(&mut self) -> Option<MarkdownNode> {
        let mut headers: Vec<MarkdownNode> = vec![];

        //TODO: replace all consume_chars("|") by assert_consume("|")

        // Parse headers
        self.consume_chars("|");
        while !is_newline(self.peek(0)) {
            headers.push(self.parse_paragraph(true).unwrap());

            self.consume_chars("|");
        }

        self.skip_whitespace();

        // Parse horizontal line
        self.consume_chars("|");
        while !is_newline(self.peek(0)) {
            self.consume_until(|c| c != "-" && !is_whitespace(c));

            self.consume_chars("|");
        }

        self.skip_whitespace();

        // Parse data
        let mut data: Vec<MarkdownNode> = vec![];

        while !self.eof() && self.peek(0) == "|" {
            self.consume_chars("|");

            while !is_newline(self.peek(0)) {
                data.push(self.parse_paragraph(true).unwrap());

                self.consume_chars("|");
            }

            self.skip_whitespace();
        }

        Some(MarkdownNode::Table(headers, data))
    }

    pub fn next_node(&mut self, single_line: bool) -> Option<MarkdownNode> {
        self.skip_whitespace();

        if self.eof() {
            return None;
        }

        let current_char = self.peek(0);
        let result_node: Option<MarkdownNode> = match current_char.as_str() {
            "#" => self.parse_header(),
            "$" => self.parse_math(),
            "`" => self.parse_code(),
            "-" => self.parse_list(),
            "|" => self.parse_table(),
            "@" => {
                self.consume();
                Some(MarkdownNode::PageBreak())
            }
            _ => {
                if current_char == "*" && self.peek(1) != "*" {
                    return self.parse_list();
                }
                self.parse_paragraph(single_line)
            }
        };

        result_node
    }

    pub fn get_html(&mut self, base_path: &Path, fast: bool) -> String {
        let mut result = String::new();

        loop {
            let node = self.next_node(false);

            match node {
                Some(x) => result.push_str(x.to_html(base_path, fast).as_str()),
                None => break,
            }
        }

        result
    }
}

impl Iterator for Parser<'_> {
    type Item = MarkdownNode;

    fn next(&mut self) -> Option<MarkdownNode> {
        self.next_node(false)
    }
}
