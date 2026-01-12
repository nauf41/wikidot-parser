mod tokenizer;
mod block;
mod inline;
mod ast;

fn parse(s: String) -> Result<(), Box<dyn std::error::Error>> {
  // get LF string
  let s = s.replace("\r\n", "\n");
  let s = s.replace("\r", "\n");

  let token = tokenizer::tokenize(s);
  let block_tree = block::parse(token);

  Ok(())
}