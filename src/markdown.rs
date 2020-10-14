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
    fn to_html(&self, fast: bool) -> String;
}

impl ToHtml for ParagraphItem {
    fn to_html(&self, fast: bool) -> String {
        match self {
            ParagraphItem::Text(text) => String::from(text),
            ParagraphItem::Italic(text) => format!("<em>{}</em>", text),
            ParagraphItem::Bold(text) => format!("<b>{}</b>", text),
            ParagraphItem::Url(name, url) => format!("<a href=\"{}\">{}</a>", url, name),
            ParagraphItem::InlineMath(math) => {
                if fast {
                    format!("${}$", math)
                } else {
                    tex_to_svg(math)
                }
            }
            ParagraphItem::Image(url, alt_text) => {
                format!("<img src=\"{}\" alt=\"{}\">", url, alt_text)
            }
            ParagraphItem::InlineCode(code) => format!("<code>{}</code>", code),
        }
    }
}

fn tex_to_svg(input: &str) -> String {
    let svg = Command::new("tex2svg").arg(input).output();

    match svg {
        Ok(x) => String::from_utf8(x.stdout).unwrap(),
        Err(_) => String::from("<center>MATH PARSING ERROR</center>"),
    }
}

impl ToHtml for MarkdownNode {
    fn to_html(&self, fast: bool) -> String {
        match self {
            MarkdownNode::Header(text, level) => format!("<h{}>{}</h{}>", level, text, level),
            MarkdownNode::List(items) => {
                let mut result: String = String::default();
                result.push_str("<ul>");
                for node in items {
                    result.push_str(&format!("<li>{}</li>", node.to_html(fast)));
                }
                result.push_str("</ul>");
                result
            }
            MarkdownNode::Math(math) => {
                if fast {
                    format!("<center>${}$</center>", math)
                } else {
                    format!("<center>{}</center>", tex_to_svg(math))
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

                return highlighted_html_for_string(&processed_code, &ss, &syntax, theme);
            }
            MarkdownNode::Paragraph(children, single_line) => {
                let mut result: String = String::default();

                if !single_line {
                    result.push_str("<p>");
                }
                for child in children {
                    result.push_str(child.to_html(fast).as_str());
                }
                if !single_line {
                    result.push_str("</p>");
                }

                result
            }
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
            nodes.push(self.next_node(true).expect("Failed to parse in list"));
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

        let mut lang = String::default();
        if !is_newline(self.peek(0)) {
            lang = self.consume_until(is_newline).trim().to_lowercase();
        }

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
                    let text = self.consume_until(|c| c == "_");
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
                            || is_newline(c)
                    });
                    ParagraphItem::Text(text)
                }
            };

            result.push(child);
        }

        Some(MarkdownNode::Paragraph(result, single_line))
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
            _ => {
                if current_char == "*" && self.peek(1) != "*" {
                    return self.parse_list();
                }
                self.parse_paragraph(single_line)
            }
        };

        result_node
    }

    pub fn to_html(&mut self, fast: bool) -> String {
        let mut result = String::new();

        loop {
            let node = self.next_node(false);

            match node {
                Some(x) => result.push_str(x.to_html(fast).as_str()),
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
