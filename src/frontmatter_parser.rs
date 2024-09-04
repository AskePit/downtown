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
        let mut indices = whole_markdown.match_indices("---").map(|x| x.0).take(2);
        let start_index = indices.next();
        let end_index = indices.next();

        if start_index.is_none() || end_index.is_none() {
            return (None, whole_markdown);
        }

        let start_index = start_index.unwrap();
        let end_index = end_index.unwrap();

        let mut res = Self {
            vars: HashMap::new(),
        };

        res.parse(&whole_markdown[start_index + 3..end_index].trim());

        (Some(res), &whole_markdown[end_index + 3..].trim())
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
                Line::ListElement(Rc::from(line.strip_prefix("-").unwrap_or(line).trim()))
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

    pub(crate) fn get(&self, variable_name: &str) -> Option<&FrontmatterVar> {
        self.vars.get(variable_name)
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
        assert_eq!(
            frontmatter.get("id"),
            Some(&FrontmatterVar::String("2".to_string()))
        );

        assert_eq!(
            frontmatter.get("kanban-plugin"),
            Some(&FrontmatterVar::String("basic".to_string()))
        );

        assert_eq!(
            frontmatter.get("urgency"),
            Some(&FrontmatterVar::String("urgent".to_string()))
        );

        assert_eq!(
            frontmatter.get("interest"),
            Some(&FrontmatterVar::String("normal".to_string()))
        );

        assert_eq!(
            frontmatter.get("strategy"),
            Some(&FrontmatterVar::String("urgent".to_string()))
        );

        assert_eq!(
            frontmatter.get("areas"),
            Some(&FrontmatterVar::List(vec![
                "projects".to_string(),
                "career".to_string(),
                "todos".to_string()
            ]))
        );
    }
}
