pub struct Builder {
  data: String,
  st: Vec<String>,
}

impl Builder {
  pub fn new() -> Self {
    Self {
      data: String::new(),
      st: vec![],
    }
  }

  pub fn escape_chars(s: &str) -> String {
    s
      .replace("&", "&amp;")
      .replace("<", "&lt;")
      .replace(">", "&gt;")
      .replace(r#"""#, "&quot;")
      .replace("'", "&apos;")
  }

  /// Open a new tag with given attributes.
  pub fn open(&mut self, key: String, values: Vec<(&str, &str)>) {
    let mut st = String::from("<");
    st += &key;
    let is_values_empty = values.is_empty();
    if !is_values_empty {
      st += " ";
    }

    for (k, v) in values {
      st += &Self::escape_chars(k);
      st += r#"=""#;
      st += &Self::escape_chars(v);
      st += r#"" "#;
    }

    if !is_values_empty {st.pop().unwrap();}
    st += ">";

    self.data += &st;
    self.st.push(key);
  }

  /// Inserts a self-closing element.
  pub fn insert(&mut self, key: String, values: Vec<(&str, &str)>) {
    let mut st = String::from("<");
    st += &key;
    let is_values_empty = values.is_empty();
    if !is_values_empty {
      st += " ";
    }

    for (k, v) in values {
      st += &Self::escape_chars(k);
      st += r#"=""#;
      st += &Self::escape_chars(v);
      st += r#"" "#;
    }

    if !is_values_empty {st.pop().unwrap();}
    st += " />";

    self.data += &st;
  }

  pub fn close(&mut self) {
    self.data += "</";
    self.data += &self.st.pop().unwrap_or(String::new());
    self.data += ">";
  }

  pub fn write(&mut self, text: &str) {
    self.data += &Self::escape_chars(text);
  }
}

impl From<Builder> for String {
  fn from(value: Builder) -> Self {
    value.data
  }
}