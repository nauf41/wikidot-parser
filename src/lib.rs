mod tokenizer;
mod block;
mod inline;
mod ast;
mod renderer;
mod constants;

pub fn parse(s: String) -> String {
  // get LF string
  let s = s.replace("\r\n", "\n"); // CRLF -> LF
  let s = s.replace("\r", "\n"); // CR -> LF

  let token = tokenizer::tokenize(s);
  //println!("{:#?}", token);
  let block_tree = block::parse(token);
  //println!("{:#?}", block_tree);
  let ast = inline::parse(block_tree);
  //println!("{:#?}", ast);
  let html = renderer::xhtml::render(ast);

  html
}
