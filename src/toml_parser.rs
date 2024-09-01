use std::collections::HashMap;

// DISCLAIMER:
// ChatGPT-4 written code. Potentially can be optimized

#[derive(Clone)]
pub(crate) struct TomlTable {
    pub(crate) entries: HashMap<String, String>,
}

#[derive(Clone)]
pub(crate) struct TomlDoc {
    pub(crate) tables: HashMap<String, TomlTable>,
}

impl TomlDoc {
    pub(crate) fn new(input: &str) -> Self {
        let mut res = TomlDoc {
            tables: HashMap::new(),
        };

        res.parse(input);

        res
    }

    fn parse(&mut self, input: &str) {
        let mut current_table: Option<String> = None;
        let mut current_key: Option<String> = None;
        let mut multiline_value = String::new();
        let mut inside_multiline = false;

        for line in input.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            if inside_multiline {
                if line.ends_with("'''") {
                    // End of multiline string
                    multiline_value.push_str(&line[..line.len() - 3]);
                    inside_multiline = false;

                    if let Some(table) = &current_table {
                        if let Some(key) = &current_key {
                            self.tables
                                .get_mut(table)
                                .unwrap()
                                .entries
                                .insert(key.clone(), multiline_value.clone());
                        }
                    }

                    multiline_value.clear();
                    current_key = None;
                } else {
                    // Continue adding to multiline string
                    multiline_value.push_str(line);
                    multiline_value.push('\n');
                }
            } else if line.starts_with('[') && line.ends_with(']') {
                // This is a table header
                let table_name = line[1..line.len() - 1].to_string();
                current_table = Some(table_name.clone());
                self.tables.insert(
                    table_name,
                    TomlTable {
                        entries: HashMap::new(),
                    },
                );
            } else if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim();

                if value.starts_with("'''") {
                    // Start of multiline string
                    inside_multiline = true;
                    current_key = Some(key);
                    multiline_value.push_str(&value[3..]);
                    multiline_value.push('\n');
                } else if value.starts_with('\'') && value.ends_with('\'') {
                    // Single-quoted string
                    let stripped_value = value[1..value.len() - 1].to_string();
                    if let Some(table) = &current_table {
                        self.tables
                            .get_mut(table)
                            .unwrap()
                            .entries
                            .insert(key, stripped_value);
                    }
                }
            }
        }
    }

    pub(crate) fn get(&self, table_name: &str, variable_name: &str) -> Option<&String> {
        self.tables.get(table_name)?.entries.get(variable_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let toml_content = r#"
[page]
prologue = '''
<!DOCTYPE html>
<html>
<head lang="en">
    <meta charset="UTF-8">
    <title>Test page</title>
</head>
<body>
'''

epilogue = '''
</body>
</html>
'''

[tags]
image = '''
<div class="image">
    <figure>
        <img src="{src}" alt="{caption}">
        <figcaption>{caption}</figcaption>
    </figure>
</div>
'''

link = '<b><a href="{src}">{caption}</a></b>'
latex = '<p class="latex">{text}</p>'
code = '<pre><code class="language-{lang}">{text}</code></pre>'
blockquote = '<div class="blockquote">{text}</div>'
horizontal_line = '<hr><hr>'
paragraph = '<p class="paragraph">{text}</p>'
bold = '<strong>{text}</strong>'
italic = '<em>{text}</em>'
strikethrough = '<del>{text}</del>'
code-inline = '<code>{text}</code>'

header = '<div class="h{level}">{text}</div>'
header3 = '<div class="special-header">{text}</div>'

error = '<div class="parse-error">{text}</div>'
    "#;

        let mut parser = TomlDoc::new(toml_content);
        parser.parse(toml_content);

        assert_eq!(
            *parser.get("tags", "paragraph").unwrap(),
            r#"<p class="paragraph">{text}</p>"#
        );
        assert_eq!(
            *parser.get("page", "epilogue").unwrap(),
            "\n</body>\n</html>\n".to_owned()
        );
        assert_eq!(
            *parser.get("tags", "header3").unwrap(),
            r#"<div class="special-header">{text}</div>"#
        );
        assert_eq!(
            *parser.get("tags", "code-inline").unwrap(),
            "<code>{text}</code>".to_owned()
        );

        assert_eq!(parser.get("page", "epilog"), None);
        assert_eq!(parser.get("tags", "epilogue"), None);
    }
}
