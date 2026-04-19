use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FrontmatterVar {
    String(String),
    List(Vec<String>),
}

pub(crate) struct Frontmatter {
    pub(crate) vars: HashMap<String, FrontmatterVar>,
}

impl Frontmatter {
    // returns parsed frontmatter and a markdown without frontmatter
    pub(crate) fn load(whole_markdown: &str) -> (Option<Self>, &str) {
        // Frontmatter must start with `---` on the very first line.
        if !whole_markdown.starts_with("---") {
            return (None, whole_markdown);
        }
        let after_opening_delim = &whole_markdown[3..];
        // The char after `---` must be a line terminator (or EOF) — otherwise it's not a fence.
        if let Some(c) = after_opening_delim.chars().next() {
            if c != '\n' && c != '\r' {
                return (None, whole_markdown);
            }
        }

        // Find a closing `---` on its own line.
        let mut search_offset = 0;
        let end_index = loop {
            let rel = match after_opening_delim[search_offset..].find("---") {
                Some(pos) => pos,
                None => return (None, whole_markdown),
            };
            let abs = search_offset + rel;
            let at_line_start = abs == 0
                || after_opening_delim.as_bytes()[abs - 1] == b'\n'
                || after_opening_delim.as_bytes()[abs - 1] == b'\r';
            let after = after_opening_delim.get(abs + 3..abs + 4);
            let at_line_end = after.map_or(true, |s| s == "\n" || s == "\r");
            if at_line_start && at_line_end {
                break abs + 3; // offset within after_opening_delim
            }
            search_offset = abs + 3;
        };

        let mut res = Self {
            vars: HashMap::new(),
        };

        res.parse(after_opening_delim[..end_index - 3].trim());

        (Some(res), after_opening_delim[end_index..].trim())
    }

    // returns end index
    fn parse(&mut self, input: &str) {
        #[derive(PartialEq)]
        enum Line {
            KeyValue((Rc<str>, Rc<str>)),
            ListKey(Rc<str>),
            ListElement(Rc<str>),
        }

        let lines = input.lines().map(|line| {
            if let Some((key, value)) = line.split_once(':') {
                if !value.trim().is_empty() {
                    Line::KeyValue((Rc::from(key.trim()), Rc::from(value.trim())))
                } else {
                    Line::ListKey(Rc::from(key.trim()))
                }
            } else {
                let line = line.trim();
                Line::ListElement(Rc::from(line.strip_prefix('-').unwrap_or(line).trim()))
            }
        });

        let mut temp_key: Rc<str> = Rc::from("");

        for line in lines {
            match line {
                Line::KeyValue((key, value)) => {
                    self.vars
                        .insert(key.to_string(), FrontmatterVar::String(value.to_string()));
                }
                Line::ListKey(key) => {
                    temp_key = key;
                }
                Line::ListElement(element) => {
                    if !self.vars.contains_key(temp_key.as_ref()) {
                        self.vars
                            .insert(temp_key.to_string(), FrontmatterVar::List(vec![]));
                    }

                    let list = self.vars.get_mut(temp_key.as_ref()).unwrap();
                    if let FrontmatterVar::List(list) = list {
                        list.push(element.to_string());
                    }
                }
            }
        }
    }

    pub(crate) fn get_var_names(&self) -> Vec<String> {
        self.vars.keys().cloned().collect()
    }

    pub(crate) fn get_string(&self, variable_name: &str) -> String {
        self.vars
            .get(variable_name)
            .map(|v| match v {
                FrontmatterVar::String(s) => s.clone(),
                FrontmatterVar::List(_) => self.get_list_joined(variable_name, ", "),
            })
            .unwrap_or_default()
    }

    pub(crate) fn get_list(&self, variable_name: &str) -> Vec<String> {
        self.vars
            .get(variable_name)
            .and_then(|v| {
                if let FrontmatterVar::List(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub(crate) fn get_list_joined(&self, variable_name: &str, separator: &str) -> String {
        self.get_list(variable_name).join(separator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter() {
        let markdown = r#"---

id: 2
kanban-plugin: basic
urgency: urgent
strategy: urgent
interest: normal
areas:
  - projects
  - career
  - todos

---

## In Work <!-- id: 0 -->"#;

        let (frontmatter, markdown) = Frontmatter::load(markdown);

        assert!(frontmatter.is_some());
        let frontmatter = frontmatter.unwrap();

        assert_eq!(markdown, "## In Work <!-- id: 0 -->");
        assert_eq!(frontmatter.get_string("id"), "2".to_string());

        assert_eq!(frontmatter.get_string("kanban-plugin"), "basic".to_string());

        assert_eq!(frontmatter.get_string("urgency"), "urgent".to_string());

        assert_eq!(frontmatter.get_string("interest"), "normal".to_string());

        assert_eq!(frontmatter.get_string("strategy"), "urgent".to_string());

        assert_eq!(
            frontmatter.get_list("areas"),
            vec![
                "projects".to_string(),
                "career".to_string(),
                "todos".to_string()
            ]
        );
    }
}
