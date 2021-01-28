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
