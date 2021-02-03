use crate::markdown::{MarkdownNode, ParagraphItem};
use unicode_segmentation::UnicodeSegmentation;

pub struct Parser<'a> {
    data: Vec<&'a str>,
    index: usize,
}

impl Iterator for Parser<'_> {
    type Item = MarkdownNode;

    fn next(&mut self) -> Option<MarkdownNode> {
        self.next_node()
    }
}

fn is_whitespace(string: &str) -> bool {
    string == " " || string == "\t"
}

fn is_newline(string: &str) -> bool {
    string == "\r\n" || string == "\n"
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

    fn consume_until(&mut self, f: fn(&str) -> bool) -> String {
        let mut result = String::default();

        loop {
            if self.eof() {
                break;
            }

            let c = self.peek(0);

            if f(&c) {
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

        if !is_whitespace(&self.peek(0)) {
            self.go_back(hashtags.len());
            return None;
        }

        self.consume();
        let header_name = String::from(self.consume_until(is_newline).trim());

        Some(MarkdownNode::Header(header_name, hashtags.len()))
    }

    fn parse_paragraph_bold(&mut self) -> Option<ParagraphItem> {
        if self.peek(0) == "*" && self.peek(1) == "*" {
            self.consume();
            self.consume();
            let text = self.consume_until(|c| c == "*");
            self.consume();
            self.consume();
            return Some(ParagraphItem::Bold(text));
        }
        None
    }

    fn parse_paragraph(&mut self) -> Option<MarkdownNode> {
        let mut result: Vec<ParagraphItem> = vec![];
        let mut text = String::default();

        loop {
            if self.eof() {
                break;
            }

            let c = self.peek(0);
            if is_newline(&c) {
                self.consume();
                if self.eof() {
                    break;
                }
                if is_newline(&self.peek(0)) {
                    self.consume();
                    break;
                }
                text.push_str(" ");
            } else {
                let bold = self.parse_paragraph_bold();
                if let Some(bold) = bold {
                    if !text.is_empty() {
                        result.push(ParagraphItem::Text(text));
                        text = String::default();
                    }
                    result.push(bold);
                } else {
                    text.push_str(self.consume());
                }
            }
        }

        if !text.is_empty() {
            result.push(ParagraphItem::Text(text));
        }

        Some(MarkdownNode::Paragraph(result))
    }

    pub fn next_node(&mut self) -> Option<MarkdownNode> {
        self.skip_whitespace();

        if self.eof() {
            return None;
        }

        let current_char = self.peek(0);
        let result_node: Option<MarkdownNode> = match current_char.as_str() {
            "#" => self.parse_header(),
            _ => self.parse_paragraph(),
        };

        result_node
    }
}
