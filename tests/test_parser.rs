use rabbit::markdown::*;
use rabbit::parser::*;

#[test]
fn empty_parser() {
    let mut parser = Parser::new("");
    assert!(parser.next_node().is_none());
}

#[test]
fn parse_single_header() {
    let mut parser = Parser::new("# Title");
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Header("Title".to_string(), 1))
    );
    assert_eq!(parser.next_node(), None);
}

#[test]
fn parse_multiple_header() {
    let mut parser = Parser::new("# Title!\n## Subtitle 1\n### Subtitle on level 3");
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Header("Title!".to_string(), 1))
    );
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Header("Subtitle 1".to_string(), 2))
    );
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Header("Subtitle on level 3".to_string(), 3))
    );
    assert_eq!(parser.next_node(), None);
}

#[test]
fn parse_single_line_paragraph() {
    let mut parser = Parser::new("This is a paragraph");
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![ParagraphItem::Text(
            "This is a paragraph".to_string()
        )]))
    );
}

#[test]
fn parse_multi_line_paragraph() {
    let mut parser = Parser::new("This is a paragraph\nwith two lines");
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![ParagraphItem::Text(
            "This is a paragraph with two lines".to_string()
        )]))
    );
}

#[test]
fn parse_multiple_paragraphs() {
    let mut parser = Parser::new("This is a paragraph\nwith two lines\n\nThis is a new paragraph on a single line\n\nThis is the last paragraph\nwhich also consists of two lines.");
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![ParagraphItem::Text(
            "This is a paragraph with two lines".to_string()
        ),]))
    );
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![ParagraphItem::Text(
            "This is a new paragraph on a single line".to_string()
        )]))
    );
    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![ParagraphItem::Text(
            "This is the last paragraph which also consists of two lines.".to_string()
        )]))
    );
}
