use unicode_segmentation::UnicodeSegmentation;

pub enum MarkdownNode {
    Header(String, usize),
    Paragraph(String),
    List(Vec<MarkdownListItem>),
    Math(String),
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

    fn push_back(&mut self, count: usize) {
        self.index -= count;
    }

    fn consume_char(&mut self, character: &str) -> String {
        let mut result = String::default();

        loop {
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
        self.consume_until(|c| c != " " && c != "\t" && c != "\n" && c != "\r");
    }

    pub fn next_node(&mut self) -> Option<MarkdownNode> {
        self.skip_whitespace();

        if self.eof() {
            return None;
        }

        let current_char = self.peek(0);
        if current_char == "#" {
            let hashtags = self.consume_char("#");

            if is_whitespace(self.peek(0)) {
                let header_name =
                    String::from(self.consume_until(|c| c == "\n" || c == "\r").trim());
                return Some(MarkdownNode::Header(header_name, hashtags.len()));
            } else {
                self.push_back(hashtags.len());
            }
        } else if current_char == "*" {
            let mut nodes: Vec<MarkdownListItem> = Vec::new();

            if is_whitespace(self.peek(1)) {
                while !self.eof() && self.peek(0) == "*" {
                    self.consume();

                    let text = String::from(self.consume_until(|c| c == "\n" || c == "\r").trim());
                    nodes.push(MarkdownListItem::ListItem(text));

                    self.skip_whitespace();
                }

                return Some(MarkdownNode::List(nodes));
            }
        } else if current_char == "$" {
            self.consume();
            let math = self.consume_until(|c| c == "$");
            self.consume();

            return Some(MarkdownNode::Math(math));
        }

        let text = String::from(self.consume_until(|c| c == "\n").trim());

        return Some(MarkdownNode::Paragraph(text));
    }
}

impl Iterator for Parser<'_> {
    type Item = MarkdownNode;

    fn next(&mut self) -> Option<MarkdownNode> {
        self.next_node()
    }
}
