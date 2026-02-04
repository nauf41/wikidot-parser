use crate::{ast::{ParseFrame, TreeElement}, block::BlockLevelAttribute, inline::data_builder::DataBuilder};

mod data_builder;
mod parse_inline;
mod tags;

pub struct ParseState {
  footnotes: Vec<Vec<TreeElement>>,
  reg_footnote_count: usize,
  out_footnote_count: usize,
}

impl ParseState {
  pub fn new() -> Self {
    Self {
      footnotes: vec![],
      reg_footnote_count: 0,
      out_footnote_count: 0,
    }
  }

  pub fn register_footnote(&mut self) -> std::num::NonZeroUsize {
    self.footnotes.push(vec![]);
    self.reg_footnote_count+=1;
    std::num::NonZeroUsize::try_from(self.reg_footnote_count).unwrap() // the length won't be zero unless unsafe parallelization
  }

  pub fn edit_footnote(&mut self, elements: Vec<crate::ast::TreeElement>) {
    if let Some(_) = self.footnotes.pop() {
      self.footnotes.push(elements);
    }
  }

  pub fn insert_footnote_block(&mut self) -> Option<TreeElement> {
    let v = std::mem::take(&mut self.footnotes);
    if !v.is_empty() {
      let mut res = vec![];
      for val in v {
        self.out_footnote_count+=1;
        res.push( crate::ast::TreeElement::FootnoteTargetChild { id: std::num::NonZeroUsize::try_from(self.out_footnote_count).unwrap(), children: val } )
      }
      Some(crate::ast::TreeElement::FootnoteTarget(res))
    } else {
      None
    }
  }
}

pub fn parse(block_tree: Vec<crate::block::BlockLevelAttribute>) -> Vec<crate::ast::TreeElement> {
  let mut db = DataBuilder::new();
  let mut shared = ParseState::new();

  let mut iters = vec![block_tree.into_iter()];

  while !iters.is_empty() {
    if let Some(block) = iters.last_mut().unwrap().next() {
      match block {
        BlockLevelAttribute::BlockQuote(children) => {
          db.push(ParseFrame::QuoteBlock);
          iters.push(children.into_iter());
        }

        BlockLevelAttribute::TabView(children) => {
          db.push(ParseFrame::TabView);
          iters.push(children.into_iter());
        }

        BlockLevelAttribute::Table(table) => {
          let mut res = vec![vec![]];

          for vc in table {
            for item in vc {
              res.last_mut().unwrap().push(crate::ast::table_cell::Cell {
                val: parse_inline::parse_inline(item.val, &mut shared),
                style: item.style,
                spanning: item.spanning,
              })
            }
          }

          db.add(TreeElement::Table(res));
        }

        BlockLevelAttribute::Tab { title, children } => {
          db.push(ParseFrame::Tab(title));
          iters.push(children.into_iter());
        }

        BlockLevelAttribute::Inline(children) => {
          db.add(TreeElement::Paragraph(parse_inline::parse_inline(children, &mut shared)));
        }
      }
    } else {
      db.pop_and_merge();
      iters.pop();
    }
  }

  if let Some(v) = shared.insert_footnote_block() {
    db.add(v);
  }

  db.into()
}

