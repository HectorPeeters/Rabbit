use rabbit::markdown::*;

#[test]
fn empty_parser() {
    let mut parser = Parser::new("");
    assert!(parser.next_node(false).is_none());
}

#[test]
fn parse_header() {
    let mut parser = Parser::new("# Title");
    let next_node = parser.next_node(false);

    assert!(next_node.is_some());
    let next_node = next_node.unwrap();
    assert!(matches!(next_node, MarkdownNode::Header{..}));
}

