use std::collections::BTreeSet;

use crate::ast::ParseFrameKind;

struct ParseFrames {
  pub values: Vec<Vec<crate::ast::TreeElement>>,
  pub frames: Vec<crate::ast::ParseFrame>,
  pub now_scoped_values: BTreeSet<ParseFrameKind>
}

impl ParseFrames {
  pub fn new() -> Self {
    Self {
      values: vec![vec![]], // must not be empty
      frames: vec![],
      now_scoped_values: BTreeSet::new()
    }
  }

  pub fn enter_frame(&mut self, frame: crate::ast::ParseFrame) {
    self.now_scoped_values.insert(crate::ast::ParseFrameKind::from(&frame));

    self.values.push(vec![]);
    self.frames.push(frame);
  }

  /// @warning 指定フレームがスタックに存在しない場合、すべての要素が途切れる。
  pub fn exit_frame(&mut self, frame: crate::ast::ParseFrame) {
    let mut stack_cache: Vec<crate::ast::ParseFrame> = vec![]; // 該当しないアイテムをcacheする

    while let Some(fr) = self.frames.pop() {
      let children = self.values.pop().unwrap();
      if fr == frame { // 目当てのものであれば完了
        self.values.last_mut().unwrap().push(fr.into_tree_element(children));
        break;
      } else {
        self.values.last_mut().unwrap().push(fr.clone().into_tree_element(children));
        stack_cache.push(fr);
      }
    }

    // TODO frameと一致する値が出るまでstackをキャッシング&要素終了しながら巻き戻し、出たらそれをpopして、キャッシングした要素を再度開始する。
  }

  pub fn exit_frame_kind(&mut self, frame: crate::ast::ParseFrameKind) {
    // TODO frameの種類の値出るまでstackをキャッシング&要素終了しながら巻き戻し、出たらそれをpopして、キャッシングした要素を再度開始する。
  }

  pub fn switch_frame(&mut self, frame: crate::ast::ParseFrame) {
    if self.now_scoped_values.contains(&crate::ast::ParseFrameKind::from(&frame)) {
      self.exit_frame(frame);
    } else {
      self.enter_frame(frame);
    }
  }
}

pub fn parse(block_tree: Vec<crate::block::BlockLevelAttribute>) -> Vec<crate::ast::TreeElement> {
  let frame = ParseFrames::new();
  vec![]
}