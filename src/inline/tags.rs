pub enum TagKind {
  Span, // HTML `span` element
  Size, // Wikidot Custom: [[size]]
  Link, // HTML `a` element
  Collapsible, // Wikidot Custom: [[collapsible]]
  Footnote, // Wikidot Custom: [[footnote]]
  FootnoteTarget, // Wikidot Custom: [[footnoteblock]]
  Include, // Wikidot Custom: [[include ...]]
  Div, // HTML `div` element
}

pub fn get_tag_kind_from_str(s: &str) -> Option<TagKind> {
  match s.to_lowercase().trim() {
    "span" => Some(TagKind::Span),
    "size" => Some(TagKind::Size),
    "a" => Some(TagKind::Link),
    "collapsible" => Some(TagKind::Collapsible),
    "footnote" => Some(TagKind::Footnote),
    "footnoteblock" => Some(TagKind::FootnoteTarget),
    "include" => Some(TagKind::Include),
    "div" => Some(TagKind::Div),
    &_ => None
  }
}