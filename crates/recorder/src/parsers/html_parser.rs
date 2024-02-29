use lightningcss::declaration::DeclarationBlock;

pub fn query_selector_first<'a>(
    dom: &'a tl::VDom<'a>,
    selector: &'a str,
    parser: &'a tl::Parser<'a>,
) -> Option<&'a tl::Node<'a>> {
    dom.query_selector(selector)
        .and_then(|mut s| s.next())
        .and_then(|n| n.get(parser))
}

pub fn query_selector_first_tag<'a>(
    dom: &'a tl::VDom<'a>,
    selector: &'a str,
    parser: &'a tl::Parser<'a>,
) -> Option<&'a tl::HTMLTag<'a>> {
    query_selector_first(dom, selector, parser).and_then(|n| n.as_tag())
}

pub fn parse_style_attr(style_attr: &str) -> Option<DeclarationBlock> {
    let result = DeclarationBlock::parse_string(style_attr, Default::default()).ok()?;
    Some(result)
}

pub fn get_tag_style<'a>(tag: &'a tl::HTMLTag<'a>) -> Option<DeclarationBlock<'a>> {
    let style_attr = tag
        .attributes()
        .get("style")
        .flatten()
        .and_then(|s| std::str::from_utf8(s.as_bytes()).ok());

    style_attr.and_then(parse_style_attr)
}
