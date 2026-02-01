mod inline_builder;

use crate::ast::{self, CssSize};
use crate::constants;
use crate::inline::tags::TagKind;
use crate::tokenizer::Token;

pub fn parse_inline(tokens: Vec<crate::tokenizer::Token>) -> Vec<crate::ast::TreeElement> {
  let mut db = inline_builder::InlineBuilder::new();

  for token in tokens {
    if let Ok(frame) = token.clone().try_into() {
      db.switch_element(frame);
      continue;
    } else {
      match token {
        Token::Bold | Token::Italics | Token::Underline | Token::Strikethrough | Token::Superscript | Token::Subscript => {
          unreachable!() // handled above
        }

        Token::MonospacedOpen => {
          db.push(ast::ParseFrame::Monospaced);
        }

        Token::MonospacedClose => {
          db.close_element(ast::ParseFrameKind::Monospaced);
        }

        Token::ElementBegin { name, attributes } => {
          if let Some(e) = crate::inline::tags::get_tag_kind_from_str(&name) {
            let mut properties: Vec<(&str, &str)> = vec![];
            let mut unnnamed_properties = vec![];
            for (key, value) in &attributes {
              match key.as_str() {
                "style" | "class" => { properties.push((key, value)); }
                "" => { unnnamed_properties.push(value); }
                &_ => {}
              }
            }

            match e {
              TagKind::Span => {
                let properties_owned = properties.iter().map(|(a, b)| { (a.to_string(), b.to_string()) }).collect();
                db.push(ast::ParseFrame::HtmlElement { tag: "span".to_string(), properties: properties_owned });
              }

              TagKind::Size => {
                if unnnamed_properties.len() >= 1 {
                  let arg = unnnamed_properties[0].to_lowercase();
                  let arg = arg.trim();
                  db.push(ast::ParseFrame::Size { scale: CssSize::new(arg) });
                } else {
                  db.push(ast::ParseFrame::Size { scale: CssSize::new("1em") });
                }
              }

              TagKind::Link => {
                for (key, value) in &attributes {
                  match key.as_str() {
                    "href" => { properties.push((key, value)); }
                    &_ => {}
                  }
                }

                let properties = properties.iter().map(|(a,b)| { (a.to_string(), b.to_string()) }).collect();
                db.push(ast::ParseFrame::HtmlElement { tag: "a".to_string(), properties });
              }

              TagKind::Collapsible => {
                let mut show_str = constants::collapsible::SHOW_BLOCK_DEFAULT_STRING;
                let mut hide_str = constants::collapsible::HIDE_BLOCK_DEFAULT_STRING;

                for (key, value) in &attributes {
                  match key.as_str() {
                    "show" => { show_str = &value; }
                    "hide" => { hide_str = &value; }
                    &_ => {}
                  }
                }

                db.push(ast::ParseFrame::Collapsible { text_open: show_str.to_string(), text_closed: hide_str.to_string() });
              }

              TagKind::Footnote => {
                let footnote_id = db.register_footnote();
                db.push(ast::ParseFrame::Footnote(footnote_id));
              }

              TagKind::FootnoteTarget => {
                todo!();
              }

              TagKind::Include => {
                todo!();
              }

              TagKind::Div => {
                let properties = properties.iter().map(|(a,b)| { (a.to_string(), b.to_string()) }).collect();
                db.push(ast::ParseFrame::HtmlElement { tag: "div".to_string(), properties });
              }
            }
          } else {
            // ignore
          }
        }

        Token::ElementEnd(name) => {
          if let Some(e) = crate::inline::tags::get_tag_kind_from_str(&name) {
            match e {
              TagKind::Span | TagKind::Size | TagKind::Link | TagKind::Collapsible | TagKind::Div => {
                db.pop_and_merge();
              }

              TagKind::Footnote => {
                todo!();
              }

              TagKind::FootnoteTarget | TagKind::Include => {
                // ignore
              }
            }
          } else { // if not allowed
            // ignore
          }
        }

        Token::ColoredBeginColorName(name) => {
          db.push(match name.as_str() {
            "aqua"    => ast::ParseFrame::Colored { red: 0x00, green: 0xFF, blue: 0xFF },
            "black"   => ast::ParseFrame::Colored { red: 0x00, green: 0x00, blue: 0x00 },
            "blue"    => ast::ParseFrame::Colored { red: 0x00, green: 0x00, blue: 0xFF },
            "fuchsia" => ast::ParseFrame::Colored { red: 0xFF, green: 0x00, blue: 0xFF }, // magenta
            "grey"    => ast::ParseFrame::Colored { red: 0x80, green: 0x80, blue: 0x80 },
            "green"   => ast::ParseFrame::Colored { red: 0x00, green: 0x80, blue: 0x00 },
            "lime"    => ast::ParseFrame::Colored { red: 0x00, green: 0xFF, blue: 0x00 },
            "maroon"  => ast::ParseFrame::Colored { red: 0x80, green: 0x00, blue: 0x00 },
            "navy"    => ast::ParseFrame::Colored { red: 0x00, green: 0x00, blue: 0x80 },
            "olive"   => ast::ParseFrame::Colored { red: 0x80, green: 0x80, blue: 0x00 },
            "purple"  => ast::ParseFrame::Colored { red: 0x80, green: 0x00, blue: 0x80 },
            "red"     => ast::ParseFrame::Colored { red: 0xFF, green: 0x00, blue: 0x00 },
            "silver"  => ast::ParseFrame::Colored { red: 0xC0, green: 0xC0, blue: 0xC0 },
            "teal"    => ast::ParseFrame::Colored { red: 0x00, green: 0x80, blue: 0x80 },
            "white"   => ast::ParseFrame::Colored { red: 0xFF, green: 0xFF, blue: 0xFF },
            "yellow"  => ast::ParseFrame::Colored { red: 0xFF, green: 0xFF, blue: 0x00 },
            &_        => unreachable!(),
          });
        }

        Token::ColoredBeginColorCode(code) => {
          let chars: Vec<char> = code.chars().collect();

          let r = u8::from_str_radix(&chars[0..2].iter().collect::<String>(), 16).unwrap_or(0);
          let g = u8::from_str_radix(&chars[2..4].iter().collect::<String>(), 16).unwrap_or(0);
          let b = u8::from_str_radix(&chars[4..6].iter().collect::<String>(), 16).unwrap_or(0);

          db.push(ast::ParseFrame::Colored { red: r, green: g, blue: b });
        }

        Token::ColoredEnd => {
          db.close_element(ast::ParseFrameKind::Colored);
        }

        Token::NamedLink { link, name } => {
          db.add(ast::TreeElement::Link { href: ast::Url(link), open_in_new_tab: false, name });
        }

        Token::PageLink { link, name } => {
          db.add(ast::TreeElement::InternalLink { href: link, open_in_new_tab: false, name });
        }

        Token::BlockQuote(_) => {
          unreachable!(); // already handled in block parsing
        }

        Token::CellSeparator(_) => {
          unreachable!(); // already handled in block parsing
        }

        Token::NewLine => {
          db.add(ast::TreeElement::NewLine);
        }

        Token::Text(text) => {
          db.add(ast::TreeElement::Text(text));
        }
      }
    }
  }

  db.into()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::TreeElement;

  fn make_paragraph(children: Vec<TreeElement>) -> Vec<TreeElement> {
    children
  }

  fn text(s: &str) -> TreeElement {
    TreeElement::Text(s.to_string())
  }

  #[test]
  fn test_plain_text() {
    let tokens = vec![Token::Text("Hello world".to_string())];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![text("Hello world")]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_multiple_text_tokens() {
    let tokens = vec![
      Token::Text("Hello".to_string()),
      Token::Text(" ".to_string()),
      Token::Text("world".to_string()),
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      text("Hello"),
      text(" "),
      text("world"),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_simple_bold() {
    let tokens = vec![
      Token::Bold,
      Token::Text("bold text".to_string()),
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![text("bold text")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_simple_italics() {
    let tokens = vec![
      Token::Italics,
      Token::Text("italic text".to_string()),
      Token::Italics,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Italics(vec![text("italic text")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_simple_underline() {
    let tokens = vec![
      Token::Underline,
      Token::Text("underlined".to_string()),
      Token::Underline,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Underline(vec![text("underlined")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_simple_strikethrough() {
    let tokens = vec![
      Token::Strikethrough,
      Token::Text("struck".to_string()),
      Token::Strikethrough,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Strikethrough(vec![text("struck")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_simple_superscript() {
    let tokens = vec![
      Token::Superscript,
      Token::Text("super".to_string()),
      Token::Superscript,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Superscript(vec![text("super")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_simple_subscript() {
    let tokens = vec![
      Token::Subscript,
      Token::Text("sub".to_string()),
      Token::Subscript,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Subscript(vec![text("sub")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_monospaced() {
    let tokens = vec![
      Token::MonospacedOpen,
      Token::Text("code".to_string()),
      Token::MonospacedClose,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Monospaced(vec![text("code")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_nested_bold_and_italics() {
    let tokens = vec![
      Token::Bold,
      Token::Text("bold ".to_string()),
      Token::Italics,
      Token::Text("and italic".to_string()),
      Token::Italics,
      Token::Text(" text".to_string()),
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![
        text("bold "),
        TreeElement::Italics(vec![text("and italic")]),
        text(" text"),
      ]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_sequential_bold_and_italics() {
    let tokens = vec![
      Token::Bold,
      Token::Text("bold".to_string()),
      Token::Bold,
      Token::Italics,
      Token::Text("italic".to_string()),
      Token::Italics,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![text("bold")]),
      TreeElement::Italics(vec![text("italic")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_color_by_name() {
    let tokens = vec![
      Token::ColoredBeginColorName("red".to_string()),
      Token::Text("red text".to_string()),
      Token::ColoredEnd,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Colored {
        red: 0xFF,
        green: 0x00,
        blue: 0x00,
        children: vec![text("red text")],
      },
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_color_by_code() {
    let tokens = vec![
      Token::ColoredBeginColorCode("FF0000".to_string()),
      Token::Text("red text".to_string()),
      Token::ColoredEnd,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Colored {
        red: 0xFF,
        green: 0x00,
        blue: 0x00,
        children: vec![text("red text")],
      },
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_color_lowercase_code() {
    let tokens = vec![
      Token::ColoredBeginColorCode("0000ff".to_string()),
      Token::Text("blue".to_string()),
      Token::ColoredEnd,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Colored {
        red: 0x00,
        green: 0x00,
        blue: 0xFF,
        children: vec![text("blue")],
      },
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_all_named_colors() {
    let colors = vec![
      ("aqua", (0x00u8, 0xFF, 0xFF)),
      ("black", (0x00, 0x00, 0x00)),
      ("blue", (0x00, 0x00, 0xFF)),
      ("fuchsia", (0xFF, 0x00, 0xFF)),
      ("grey", (0x80, 0x80, 0x80)),
      ("green", (0x00, 0x80, 0x00)),
      ("lime", (0x00, 0xFF, 0x00)),
      ("maroon", (0x80, 0x00, 0x00)),
      ("navy", (0x00, 0x00, 0x80)),
      ("olive", (0x80, 0x80, 0x00)),
      ("purple", (0x80, 0x00, 0x80)),
      ("red", (0xFF, 0x00, 0x00)),
      ("silver", (0xC0, 0xC0, 0xC0)),
      ("teal", (0x00, 0x80, 0x80)),
      ("white", (0xFF, 0xFF, 0xFF)),
      ("yellow", (0xFF, 0xFF, 0x00)),
    ];

    for (name, (r, g, b)) in colors {
      let tokens = vec![
        Token::ColoredBeginColorName(name.to_string()),
        Token::Text("text".to_string()),
        Token::ColoredEnd,
      ];
      let result = parse_inline(tokens);
      let expected = make_paragraph(vec![
        TreeElement::Colored {
          red: r,
          green: g,
          blue: b,
          children: vec![text("text")],
        },
      ]);
      assert_eq!(result, expected, "Failed for color: {}", name);
    }
  }

  #[test]
  fn test_named_link() {
    let tokens = vec![
      Token::NamedLink {
        link: "https://example.com".to_string(),
        name: "click here".to_string(),
      },
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Link {
        href: crate::ast::Url("https://example.com".to_string()),
        open_in_new_tab: false,
        name: "click here".to_string(),
      },
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_page_link() {
    let tokens = vec![
      Token::PageLink {
        link: "about/author".to_string(),
        name: "author page".to_string(),
      },
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::InternalLink {
        href: "about/author".to_string(),
        open_in_new_tab: false,
        name: "author page".to_string(),
      },
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_newline() {
    let tokens = vec![
      Token::Text("line1".to_string()),
      Token::NewLine,
      Token::Text("line2".to_string()),
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      text("line1"),
      TreeElement::NewLine,
      text("line2"),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_multiple_newlines() {
    let tokens = vec![
      Token::Text("a".to_string()),
      Token::NewLine,
      Token::NewLine,
      Token::Text("b".to_string()),
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      text("a"),
      TreeElement::NewLine,
      TreeElement::NewLine,
      text("b"),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_bold_and_colored() {
    let tokens = vec![
      Token::Bold,
      Token::ColoredBeginColorName("blue".to_string()),
      Token::Text("blue bold".to_string()),
      Token::ColoredEnd,
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![
        TreeElement::Colored {
          red: 0x00,
          green: 0x00,
          blue: 0xFF,
          children: vec![text("blue bold")],
        },
      ]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_complex_nesting() {
    let tokens = vec![
      Token::Bold,
      Token::Text("b".to_string()),
      Token::Italics,
      Token::Text("bi".to_string()),
      Token::MonospacedOpen,
      Token::Text("bim".to_string()),
      Token::MonospacedClose,
      Token::Text("bi2".to_string()),
      Token::Italics,
      Token::Text("b2".to_string()),
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![
        text("b"),
        TreeElement::Italics(vec![
          text("bi"),
          TreeElement::Monospaced(vec![text("bim")]),
          text("bi2"),
        ]),
        text("b2"),
      ]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_empty_formatting() {
    let tokens = vec![
      Token::Bold,
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_superscript_nested_in_bold() {
    let tokens = vec![
      Token::Bold,
      Token::Text("E".to_string()),
      Token::Superscript,
      Token::Text("2".to_string()),
      Token::Superscript,
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![
        text("E"),
        TreeElement::Superscript(vec![text("2")]),
      ]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_subscript_nested_in_bold() {
    let tokens = vec![
      Token::Bold,
      Token::Text("H".to_string()),
      Token::Subscript,
      Token::Text("2".to_string()),
      Token::Subscript,
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![
        text("H"),
        TreeElement::Subscript(vec![text("2")]),
      ]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_color_with_bold_and_italics() {
    let tokens = vec![
      Token::ColoredBeginColorName("green".to_string()),
      Token::Bold,
      Token::Text("green".to_string()),
      Token::Italics,
      Token::Text("text".to_string()),
      Token::Italics,
      Token::Bold,
      Token::ColoredEnd,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Colored {
        red: 0x00,
        green: 0x80,
        blue: 0x00,
        children: vec![
          TreeElement::Bold(vec![
            text("green"),
            TreeElement::Italics(vec![text("text")]),
          ]),
        ],
      },
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_monospaced_complex() {
    let tokens = vec![
      Token::Text("prefix ".to_string()),
      Token::MonospacedOpen,
      Token::Text("mono ".to_string()),
      Token::Bold,
      Token::Text("bold in mono".to_string()),
      Token::Bold,
      Token::Text(" more mono".to_string()),
      Token::MonospacedClose,
      Token::Text(" suffix".to_string()),
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      text("prefix "),
      TreeElement::Monospaced(vec![
        text("mono "),
        TreeElement::Bold(vec![text("bold in mono")]),
        text(" more mono"),
      ]),
      text(" suffix"),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_consecutive_links() {
    let tokens = vec![
      Token::NamedLink {
        link: "https://example.com".to_string(),
        name: "link1".to_string(),
      },
      Token::Text(" ".to_string()),
      Token::PageLink {
        link: "page2".to_string(),
        name: "link2".to_string(),
      },
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Link {
        href: crate::ast::Url("https://example.com".to_string()),
        open_in_new_tab: false,
        name: "link1".to_string(),
      },
      text(" "),
      TreeElement::InternalLink {
        href: "page2".to_string(),
        open_in_new_tab: false,
        name: "link2".to_string(),
      },
    ]);
    assert_eq!(result, expected);
  }

  // Tests for wrongly-layered code (allowed but still tested)
  #[test]
  fn test_unclosed_bold() {
    let tokens = vec![
      Token::Bold,
      Token::Text("unclosed bold".to_string()),
    ];
    let result = parse_inline(tokens);
    // Unclosed elements remain open in the paragraph
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![text("unclosed bold")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_unopened_closing_token() {
    let tokens = vec![
      Token::Text("text".to_string()),
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    // Unopened closing token creates an empty element
    let expected = make_paragraph(vec![
      text("text"),
      TreeElement::Bold(vec![]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_interleaved_formatting_bold_italics() {
    // This represents **a//b**c//d (wrongly-layered)
    let tokens = vec![
      Token::Bold,
      Token::Text("a".to_string()),
      Token::Italics,
      Token::Text("b".to_string()),
      Token::Bold,
      Token::Text("c".to_string()),
      Token::Italics,
    ];
    let result = parse_inline(tokens);
    // When closing Bold (which is inside Italics), it closes Italics too, then reopens it
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![
        text("a"),
        TreeElement::Italics(vec![text("b")]),
      ]),
      TreeElement::Italics(vec![text("c")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_interleaved_colors_and_bold() {
    let tokens = vec![
      Token::ColoredBeginColorName("red".to_string()),
      Token::Bold,
      Token::Text("text".to_string()),
      Token::ColoredEnd,
      Token::Text("more".to_string()),
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    // When closing Colored (which contains Bold), Bold gets reopened after
    let expected = make_paragraph(vec![
      TreeElement::Colored {
        red: 0xFF,
        green: 0x00,
        blue: 0x00,
        children: vec![
          TreeElement::Bold(vec![text("text")]),
        ],
      },
      TreeElement::Bold(vec![text("more")]),
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_multiple_colors_sequential() {
    let tokens = vec![
      Token::ColoredBeginColorName("red".to_string()),
      Token::Text("red".to_string()),
      Token::ColoredEnd,
      Token::ColoredBeginColorName("blue".to_string()),
      Token::Text("blue".to_string()),
      Token::ColoredEnd,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Colored {
        red: 0xFF,
        green: 0x00,
        blue: 0x00,
        children: vec![text("red")],
      },
      TreeElement::Colored {
        red: 0x00,
        green: 0x00,
        blue: 0xFF,
        children: vec![text("blue")],
      },
    ]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_bold_italics_toggle() {
    // **a**b//c//d**e**
    let tokens = vec![
      Token::Bold,
      Token::Text("a".to_string()),
      Token::Bold,
      Token::Text("b".to_string()),
      Token::Italics,
      Token::Text("c".to_string()),
      Token::Italics,
      Token::Text("d".to_string()),
      Token::Bold,
      Token::Text("e".to_string()),
      Token::Bold,
    ];
    let result = parse_inline(tokens);
    let expected = make_paragraph(vec![
      TreeElement::Bold(vec![text("a")]),
      text("b"),
      TreeElement::Italics(vec![text("c")]),
      text("d"),
      TreeElement::Bold(vec![text("e")]),
    ]);
    assert_eq!(result, expected);
  }
}