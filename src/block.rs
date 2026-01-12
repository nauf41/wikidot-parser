use crate::tokenizer::Token;
use crate::ast::table_cell;

mod parse_table;

#[derive(PartialEq, Eq, Debug)]
pub enum BlockLevelAttribute {
  BlockQuote(Vec<BlockLevelAttribute>),
  Table(Vec<Vec<table_cell::Cell>>), // Inline以外中には入らないようにする必要がある.

  Inline(Vec<crate::tokenizer::Token>), // トップレベルのInlineは段落を示す.
}

pub struct DataBuilder {
  data: Vec<Vec<BlockLevelAttribute>>, // BlockQuoteの深さでインデックス。0番は深さ0(=BlockQuoteなし)で1番は深さ1(=`> Lorem ipsum`)
}

impl DataBuilder {
  pub fn new() -> Self {
    Self {
      data: vec![vec![]], // the size of this must keep 1 or above, or Rust may panics
    }
  }

  fn change_stack_quote_level_to(&mut self, depth: usize) {
    let target_data_size: usize = depth + 1;
    if self.data.len() > target_data_size {
      while self.data.len() > target_data_size { // length is 2 or above
        let attrs = self.data.pop().unwrap();
        self.data.last_mut().unwrap().push(BlockLevelAttribute::BlockQuote(attrs));
      }
    } else if self.data.len() < target_data_size {
      while self.data.len() < target_data_size {
        self.data.push(vec![]);
      }
    }
  }

  // 深さ0はBlockQuoteなし
  pub fn add_block(&mut self, mut buf: Vec<crate::tokenizer::Token>, depth: usize) {
    self.change_stack_quote_level_to(depth);

    while let Some(Token::NewLine) = buf.last() {
      buf.pop().unwrap();
    }

    for v in parse_table::parse_table(&mut buf) {
      self.data.last_mut().unwrap().push(v); // data array never be empty
    }
  }

  pub fn get(mut self) -> Vec<BlockLevelAttribute> {
    self.change_stack_quote_level_to(0);
    std::mem::take(&mut self.data[0])
  }
}


pub fn parse(tokens: Vec<crate::tokenizer::Token>) -> Vec<BlockLevelAttribute> {
  let mut db = DataBuilder::new();

  let mut buf: Vec<crate::tokenizer::Token> = vec![];
  let mut last_depth: usize = 0;
  let mut was_prev_newline = false; // token列で単純に直前がNewLineであるかを保存.

  for token in tokens {
    match token {
      Token::BlockQuote(level) => {
        let level: usize = level.get();
        if last_depth != level {
          // end of the block
          db.add_block(std::mem::take(&mut buf), last_depth);
        } else {
          // continue; nothing to do
        }

        last_depth = level;
        was_prev_newline = false;
      }

      Token::NewLine => {
        if was_prev_newline {
          // End of the block
          db.add_block(std::mem::take(&mut buf), last_depth);
        }
        if !buf.is_empty() {
          if let Some(Token::NewLine) = buf.last() {
            // skip
          } else {
            buf.push(Token::NewLine);
          }
        } else {
          // skip
        }

        was_prev_newline = true;
      }

      _ => {
        if was_prev_newline { // Newlineの直後にBlockQuoteでない要素が来た == BlockQuote全体の終わり
          db.add_block(std::mem::take(&mut buf), last_depth);
          last_depth = 0;
        }
        buf.push(token);

        was_prev_newline = false;
      }
    }
  }

  db.add_block(buf, last_depth);

  db.get()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn nz(v: usize) -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::try_from(v).unwrap()
  }

  #[test]
  fn test_empty() {
    assert_eq!(parse(vec![]), vec![]);
  }

  #[test]
  fn test_blockquote() {
    use crate::tokenizer::Token;
    assert_eq!(parse(vec![
      Token::BlockQuote(nz(1)), Token::Text(String::from("Hello,")), Token::NewLine,
      Token::BlockQuote(nz(2)), Token::Text(String::from("World!")), Token::NewLine
    ]), vec![
      BlockLevelAttribute::BlockQuote(vec![
        BlockLevelAttribute::Inline(vec![Token::Text(String::from("Hello,"))]),
        BlockLevelAttribute::BlockQuote(vec![
          BlockLevelAttribute::Inline(vec![Token::Text(String::from("World!"))])
        ]),
      ])
    ]);
  }

  // AI-generated tests are below

  #[test]
  fn test_table_in_blocks() {
    use crate::tokenizer::tokenize;
    use crate::tokenizer::Token;
    // "a\n|| a || b ||\nc"
    let tokens = tokenize(String::from("a\n|| a || b ||\nc"));
    let parsed = parse(tokens);

    assert_eq!(parsed, vec![
      BlockLevelAttribute::Inline(vec![Token::Text(String::from("a"))]),
      BlockLevelAttribute::Table(vec![
        vec![
          crate::block::table_cell::Cell { val: vec![Token::Text(String::from(" a "))], style: None, spanning: nz(1) },
          crate::block::table_cell::Cell { val: vec![Token::Text(String::from(" b "))], style: None, spanning: nz(1) },
        ]
      ]),
      BlockLevelAttribute::Inline(vec![Token::Text(String::from("c"))]),
    ]);
  }

  #[test]
  fn test_nested_closings() {
    use crate::tokenizer::Token;
    // depth 1 -> 2 -> 3 then close to 1 and continue
    let tokens = vec![
      Token::BlockQuote(nz(1)), Token::Text(String::from("L1")), Token::NewLine,
      Token::BlockQuote(nz(2)), Token::Text(String::from("L2")), Token::NewLine,
      Token::BlockQuote(nz(3)), Token::Text(String::from("L3")), Token::NewLine,
      Token::BlockQuote(nz(1)), Token::Text(String::from("After")), Token::NewLine,
    ];

    let parsed = parse(tokens);

    assert_eq!(parsed, vec![
      BlockLevelAttribute::BlockQuote(vec![
        BlockLevelAttribute::Inline(vec![Token::Text(String::from("L1"))]),
        BlockLevelAttribute::BlockQuote(vec![
          BlockLevelAttribute::Inline(vec![Token::Text(String::from("L2"))]),
          BlockLevelAttribute::BlockQuote(vec![
            BlockLevelAttribute::Inline(vec![Token::Text(String::from("L3"))])
          ]),
        ]),
        BlockLevelAttribute::Inline(vec![Token::Text(String::from("After"))]),
      ])
    ]);
  }
}