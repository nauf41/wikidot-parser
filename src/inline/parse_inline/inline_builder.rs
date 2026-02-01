use crate::ast::{self, TreeElement};

pub struct InlineBuilder {
  root: Vec<ast::TreeElement>,
  data: Vec<(ast::ParseFrame, Vec<ast::TreeElement>)>,
  footnotes: Vec<Vec<ast::TreeElement>>,
  footnoteoffset: usize,
}

impl InlineBuilder {
  pub fn new() -> Self {
    Self {
      root: vec![],
      data: vec![],
      footnotes: vec![],
      footnoteoffset: 0,
    }
  }

  pub fn pop_and_merge(&mut self) -> Option<ast::ParseFrame> {
    if let Some((frame, container)) = self.data.pop() {
      if let Some((_, parent_container)) = self.data.last_mut() {
        parent_container.push(frame.clone().into_tree_element(container));
      } else {
        self.root.push(frame.clone().into_tree_element(container));
      }

      Some(frame)
    } else {
      None
    }
  }

  pub fn close_element(&mut self, kind: ast::ParseFrameKind) -> bool {
    let mut frame_to_reopen: Vec<ast::ParseFrame> = vec![]; // Frames that need to be reopened

    let mut reached = false;
    while let Some(frame) = self.pop_and_merge() {
      if frame.get_kind() == kind {
        reached = true;
        break;
      } else {
        frame_to_reopen.push(frame);
      }
    }

    for frame in frame_to_reopen {
      self.push(frame);
    }

    reached
  }

  pub fn switch_element(&mut self, param_frame: ast::ParseFrame) {
    for (frame, _) in &self.data {
      if frame.get_kind() == param_frame.get_kind() {
        self.close_element(param_frame.get_kind()); // if found: open
        return;
      }
    }

    self.push(param_frame); // if not found: close
  }

  pub fn push(&mut self, frame: ast::ParseFrame) {
    self.data.push((frame, vec![]));
  }

  pub fn add(&mut self, element: ast::TreeElement) {
    if let Some((_, container)) = self.data.last_mut() {
      container.push(element);
    } else {
      self.root.push(element);
    }
  }

  pub fn register_footnote(&mut self) -> std::num::NonZeroUsize {
    self.footnotes.push(vec![]);
    std::num::NonZeroUsize::try_from(self.footnotes.len()).unwrap() // the length won't be zero unless unsafe parallelization
  }

  pub fn edit_footnote(&mut self, elements: Vec<ast::TreeElement>) {
    if let Some(_) = self.footnotes.pop() {
      self.footnotes.push(elements);
    }
  }

  pub fn get_now_children(&mut self) -> Vec<TreeElement> {
    if let Some(v) = self.data.pop() {
      v.1
    } else {
      panic!()
    }
  }

  pub fn insert_footnote_block(&mut self) {
    let v = std::mem::take(&mut self.footnotes);
    if !v.is_empty() {
      let offset = self.footnoteoffset;
      self.footnoteoffset += v.len();
      let mut result_vec = vec![];
      for (idx, val) in v.into_iter().enumerate() {
        result_vec.push((offset+idx, val));
      }
      self.add(ast::TreeElement::FootnoteTarget(result_vec));
    }
  }
}

impl From<InlineBuilder> for Vec<ast::TreeElement> {
  fn from(mut builder: InlineBuilder) -> Vec<ast::TreeElement> {
    while let Some(_) = builder.pop_and_merge() {}
    builder.insert_footnote_block();
    builder.root
  }
}