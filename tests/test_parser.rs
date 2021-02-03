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

#[test]
fn parse_bold_paragraph() {
    let mut parser = Parser::new("This is a paragraph with **bold** text");

    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![
            ParagraphItem::Text("This is a paragraph with ".to_string()),
            ParagraphItem::Bold("bold".to_string()),
            ParagraphItem::Text(" text".to_string()),
        ],))
    );
}

#[test]
fn parse_bold_end_paragraph() {
    let mut parser = Parser::new("This is a paragraph with **bold**");

    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![
            ParagraphItem::Text("This is a paragraph with ".to_string()),
            ParagraphItem::Bold("bold".to_string()),
        ],))
    );
}

#[test]
fn parse_bold_start_paragraph() {
    let mut parser = Parser::new("**bold** is in this paragraph");

    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![
            ParagraphItem::Bold("bold".to_string()),
            ParagraphItem::Text(" is in this paragraph".to_string()),
        ],))
    );
}

#[test]
fn parse_asterix_paragraph() {
    let mut parser = Parser::new("*non bold* is in this paragraph");

    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![ParagraphItem::Text(
            "*non bold* is in this paragraph".to_string()
        ),],))
    );
}

#[test]
fn parse_asterix_end_paragraph() {
    let mut parser = Parser::new("Asterix at the end of this paragraph *");

    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::Paragraph(vec![ParagraphItem::Text(
            "Asterix at the end of this paragraph *".to_string()
        ),],))
    );
}

#[test]
fn parse_basic_list() {
    let mut parser = Parser::new("- Item 1\n- Item 2\n- Item 3");

    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::List(vec![
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 1".to_string())]),
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 2".to_string())]),
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 3".to_string())]),
        ]))
    );
}

#[test]
fn parse_multiple_lists() {
    let mut parser = Parser::new("- Item 1\n- Item 2\n- Item 3\n\n- Item 4\n- Item 5\n- Item 6");

    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::List(vec![
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 1".to_string())]),
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 2".to_string())]),
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 3".to_string())]),
        ]))
    );

    assert_eq!(
        parser.next_node(),
        Some(MarkdownNode::List(vec![
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 4".to_string())]),
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 5".to_string())]),
            MarkdownNode::Paragraph(vec![ParagraphItem::Text("Item 6".to_string())]),
        ]))
    );
}
