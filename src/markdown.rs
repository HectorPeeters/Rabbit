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
}

pub enum MarkdownListItem {
    ListItem(String),
}

pub struct Parser<'a> {
    data: Vec<&'a str>,
    index: usize,
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
        if current_char == "#" {
            let hashtags = self.consume_chars("#");

            if is_whitespace(self.peek(0)) {
                self.consume();
                let header_name = String::from(self.consume_until(is_newline).trim());
                return Some(MarkdownNode::Header(header_name, hashtags.len()));
            } else {
                self.go_back(hashtags.len());
            }
        } else if current_char == "*" {
            let mut nodes: Vec<MarkdownListItem> = Vec::new();

            if is_whitespace(self.peek(1)) {
                while !self.eof() && self.peek(0) == "*" {
                    self.consume();

                    let text = String::from(self.consume_until(is_newline).trim());
                    nodes.push(MarkdownListItem::ListItem(text));

                    self.skip_whitespace();
                }

                return Some(MarkdownNode::List(nodes));
            }
        } else if current_char == "$" {
            let dollars = self.consume_chars("$");

            let mut mode = MathMode::Inline;
            if dollars.len() == 2 {
                mode = MathMode::NonInline;
            }

            let math = String::from(self.consume_until(|c| c == "$").trim());
            self.consume_until(|c| c != "$");

            return Some(MarkdownNode::Math(math, mode));
        } else if current_char == "`" {
            let hashtags = self.consume_chars("`");
            if hashtags.len() == 3 {
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
        }

        if self.eof() {
            return None;
        }

        let text = String::from(self.consume_until(is_newline).trim());

        return Some(MarkdownNode::Paragraph(text));
    }
}

impl Iterator for Parser<'_> {
    type Item = MarkdownNode;

    fn next(&mut self) -> Option<MarkdownNode> {
        self.next_node()
    }
}
