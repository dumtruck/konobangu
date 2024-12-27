use lightningcss::declaration::DeclarationBlock;

pub fn parse_style_attr(style_attr: &str) -> Option<DeclarationBlock> {
    let result = DeclarationBlock::parse_string(style_attr, Default::default()).ok()?;
    Some(result)
}
