mod builder;

pub fn render(ast: Vec<crate::ast::TreeElement>) -> String {
  use crate::ast::TreeElement;

  let mut res = builder::Builder::new();

  let mut unique_id_counter: usize = 0;

  res.open(String::from("html"), vec![]);
  res.open(String::from("head"), vec![]);
  res.insert(String::from("meta"), vec![("charset", "UTF-8")]);
  res.insert(String::from("meta"), vec![("name", "viewport"), ("content", "width=device-width, initial-scale=1")]);
  res.close(); // </head>
  res.open(String::from("body"), vec![]);

  let mut iters = vec![Some(ast.into_iter())];

  while !iters.is_empty() {
    if let Some(i) = iters.last_mut().unwrap() {
      if let Some(v) = i.next() {
        match v {
          TreeElement::Paragraph(children) => {
            res.open(String::from("p"), vec![]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Text(text) => {
            res.write(&text);
          }

          TreeElement::Bold(children) => {
            res.open(String::from("strong"), vec![]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Italics(children) => {
            res.open(String::from("i"), vec![]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Underline(children) => {
            res.open(String::from("span"), vec![("style", "text-decoration: underline")]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Strikethrough(children) => {
            res.open(String::from("s"), vec![]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Monospaced(children) => {
            res.open(String::from("code"), vec![]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Superscript(children) => {
            res.open(String::from("sup"), vec![]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Subscript(children) => {
            res.open(String::from("sub"), vec![]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Colored{red, green, blue, children} => {
            res.open(String::from("span"), vec![("style", &format!("color: rgb({}, {}, {})", red, green, blue))]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Size { scale, children } => {
            res.open(String::from("span"), vec![("style", &format!("font-size: {}", scale.0.as_str()))]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Link { href, open_in_new_tab, name } => {
            let mut attrs = vec![("href", &href.0 as &str)];
            if open_in_new_tab {
              attrs.push(("target", "_blank"));
              attrs.push(("rel", "noopener noreferrer"));
            }
            res.open(String::from("a"), attrs);
            res.write(&name);
            res.close()
          }

          TreeElement::Collapsible{text_open, text_closed, children} => {
            let open_id = format!("collapsible_open{}", unique_id_counter.to_string());
            let close_id = format!("collapsible_close{}", unique_id_counter.to_string());

            res.open("div".to_string(), vec![]);
              res.open("div".to_string(), vec![("id", &open_id)]);
                res.insert("a".to_string(), vec![("onclick", &format!(r##"()=>{{document.querySelector("#{}").style.display="none"; document.querySelector("#{}").style.display="block";}}"##, &close_id, &open_id))]);
              res.close();
              res.open("div".to_string(), vec![("id", &close_id)]);
                res.insert("a".to_string(), vec![("onclick", &format!(r##"()=>{{document.querySelector("#{}").style.display="none"; document.querySelector("#{}").style.display="block";}}"##, &open_id, &close_id))]);
              iters.push(Some(children.into_iter()));
            iters.push(None);
            unique_id_counter+=1;
          }

          TreeElement::Footnote(id) => {
            res.open("sup".to_string(), vec![]);
              res.open("a".to_string(), vec![("href", &format!("#{}{}", crate::constants::FOOTNOTE_ID_PREFIX, id.get()))]);
              res.close();
            res.close();
          }

          TreeElement::FootnoteTarget(children) => {
            res.open("div".to_string(), vec![("class", "footnoteblock")]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::FootnoteTargetChild { id, children } => {
            res.open("div".to_string(), vec![("id", &format!("{}{}", crate::constants::FOOTNOTE_ID_PREFIX, id.get()))]);
            res.write(&format!("{}. ", id.get())); // e.g. "1. some footnote"
            iters.push(Some(children.into_iter()));
          }

          TreeElement::QuoteBlock(children) => {
            res.open(String::from("blockquote"), vec![]);
            iters.push(Some(children.into_iter()));
          }

          TreeElement::Iframe(raw) => { // TODO size?
            res.open(String::from("iframe"), vec![("srcdoc", &raw)]);
            res.close();
          }

          TreeElement::Tab{title, children} => {
            todo!()
          }

          TreeElement::TabView(_) => {
            todo!()
          }

          TreeElement::Table(rows) => {
            res.open(String::from("table"), vec![]);
            for row in rows {
              res.open(String::from("tr"), vec![]);
              for cell in row {
                todo!();
              }
              res.close();
            }
            res.close();
          }

          TreeElement::NewLine => {
            res.insert(String::from("br"), vec![]);
          }

          TreeElement::HtmlElement { tag, property, children } => {
            let attrs: Vec<(String, String)> = property.iter().map(|(k, v)| (k.clone(), v.replace("\n", "").replace("\"", ""))).collect();
            let attrs_refs: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
            res.open(tag, attrs_refs);
            iters.push(Some(children.into_iter()));
          }
        }
      } else {
        if iters.len() > 1 {res.close()}; // don't close root
        iters.pop();
      }
    } else {
      iters.pop();
    }
  }

  res.close(); // </body>
  res.close(); // </html>

  res.into()
}