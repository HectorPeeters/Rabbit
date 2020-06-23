use unicode_segmentation::UnicodeSegmentation;

pub enum MathMode {
    NonInline,
    Inline,
}

pub enum MarkdownNode {
    Header(String, usize),
    Paragraph(String),
    List(Vec<MarkdownListItem>),
    Math(String, MathMode),
    Code(String, String),
    URL(String, String),
}

pub enum MarkdownListItem {
    ListItem(String),
}

pub struct Parser<'a> {
    data: Vec<&'a str>,
    index: usize,
}

pub trait ToHtml {
    fn to_html(&self) -> String;
}

impl ToHtml for MarkdownNode {
    fn to_html(&self) -> String {
        match self {
            MarkdownNode::Header(text, level) => format!("<h{}>{}</h{}>", level, text, level),
            MarkdownNode::Paragraph(text) => format!("<p>{}</p>", text),
            MarkdownNode::List(items) => {
                let mut result: String = String::default();
                result.push_str("<ul>");
                for text in items {
                    match text {
                        MarkdownListItem::ListItem(text) => {
                            result.push_str(&format!("<li>{}</li>", text));
                        }
                    }
                }
                result.push_str("</ul>");
                result
            }
            MarkdownNode::Math(math, mode) => match mode {
                MathMode::NonInline => format!("<center>${}$</center><br>", math),
                MathMode::Inline => format!("${}$", math),
            },
            MarkdownNode::Code(lang, code) => {
                format!("<pre><code class=\"{}\">{}</code></pre>", lang, code)
            }
            MarkdownNode::URL(name, url) => {
                format!("<a href=\"{}\">{}</a>", url, name)
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

    pub fn next_node(&mut self) -> Option<MarkdownNode> {
        self.skip_whitespace();

        if self.eof() {
            return None;
        }

        let current_char = self.peek(0);
        let result_node: Option<MarkdownNode> = match current_char.as_str() {
            "#" => {
                let hashtags = self.consume_chars("#");

                if !is_whitespace(self.peek(0)) {
                    self.go_back(hashtags.len());
                    return None;
                }

                self.consume();
                let header_name = String::from(self.consume_until(is_newline).trim());
                return Some(MarkdownNode::Header(header_name, hashtags.len()));
            }
            "*" => {
                let mut nodes: Vec<MarkdownListItem> = Vec::new();

                if !is_whitespace(self.peek(1)) {
                    return None;
                }

                while !self.eof() && self.peek(0) == "*" {
                    self.consume();
                    let text = String::from(self.consume_until(is_newline).trim());
                    nodes.push(MarkdownListItem::ListItem(text));
                    self.skip_whitespace();
                }

                return Some(MarkdownNode::List(nodes));
            }
            "$" => {
                let dollars = self.consume_chars("$");

                let mut mode = MathMode::Inline;
                if dollars.len() == 2 {
                    mode = MathMode::NonInline;
                }

                let math = String::from(self.consume_until(|c| c == "$").trim());
                self.consume_until(|c| c != "$");

                return Some(MarkdownNode::Math(math, mode));
            }
            "`" => {
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

                return Some(MarkdownNode::Code(lang, code));
            }
            "<" => {
                self.consume();

                let url = self.consume_until(|c| c == ">");
                let name = url.clone();

                self.consume();

                return Some(MarkdownNode::URL(name, url));
            }
            _ => None,
        };

        match result_node {
            Some(x) => return Some(x),
            None => {
                if self.eof() {
                    return None;
                }

                let text = String::from(self.consume_until(is_newline).trim());
                return Some(MarkdownNode::Paragraph(text));
            }
        }
    }
}

impl Iterator for Parser<'_> {
    type Item = MarkdownNode;

    fn next(&mut self) -> Option<MarkdownNode> {
        self.next_node()
    }
}
