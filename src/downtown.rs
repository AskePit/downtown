use std::sync::Arc;

type Block = Arc<str>;
type BlocksSlice = Arc<[Block]>;
type ParseUnit = BlocksSlice;

pub struct Markdown2Html {
    parse_context: ParseContext,
}

type Level = u8;

#[derive(Debug, Clone, Copy)]
enum UnitType {
    Header(Level),
    Text,
    List,
    Image,
    Latex,
    Code,
    Callout,
}

struct ParseContext {
    parse_units: Vec<ParseUnit>,
    unit_types: Vec<UnitType>,
}

impl Markdown2Html {
    pub fn new(input: String) -> Markdown2Html {
        let input: Vec<_> = input
            .split('\n')
            .filter(|x| !x.trim().is_empty())
            .map(|x| Arc::from(x.trim_end()))
            .collect();

        let parse_context = Markdown2Html::analyze_input(&input);

        Markdown2Html { parse_context }
    }

    pub fn generate_html(&self) -> String {
        let mut output_vec = vec!["".to_owned(); self.parse_context.parse_units.len()];

        for i in 0..self.parse_context.parse_units.len() {
            let parse_unit = self.parse_context.parse_units[i].clone();
            let unit_type = self.parse_context.unit_types[i];
            let output = &mut output_vec[i];

            process_unit(parse_unit, unit_type, output);
        }

        output_vec.join("\n")
    }

    fn analyze_input(input: &Vec<Block>) -> ParseContext {
        let mut context = ParseContext {
            parse_units: vec![],
            unit_types: vec![],
        };

        let mut multiline_state = false;
        let mut multiline_counter: usize = 0;

        let mut block_start: usize = 0;

        for (i, block) in input.iter().enumerate() {
            if multiline_state {
                let state_type = context.unit_types.last().unwrap();
                match state_type {
                    UnitType::List => {
                        if block.trim_start().starts_with("- ") {
                            multiline_counter += 1;
                            continue;
                        } else {
                            context.parse_units.push(Arc::from(
                                &input[block_start..block_start + multiline_counter],
                            ));
                            multiline_state = false;
                        }
                    }
                    UnitType::Callout => {
                        if block.starts_with(">") {
                            multiline_counter += 1;
                            continue;
                        } else {
                            context.parse_units.push(Arc::from(
                                &input[block_start..block_start + multiline_counter],
                            ));
                            multiline_state = false;
                        }
                    }
                    UnitType::Latex => {
                        multiline_counter += 1;

                        if block.starts_with("$$") {
                            context.parse_units.push(Arc::from(
                                &input[block_start..block_start + multiline_counter],
                            ));
                            multiline_state = false;
                        }
                        continue;
                    }
                    UnitType::Code => {
                        multiline_counter += 1;

                        if block.starts_with("```") {
                            context.parse_units.push(Arc::from(
                                &input[block_start..block_start + multiline_counter],
                            ));
                            multiline_state = false;
                        }
                        continue;
                    }

                    _ => {}
                }
            }

            if block.starts_with("# ") {
                context.unit_types.push(UnitType::Header(1));
                context.parse_units.push(Arc::from(&input[i..i + 1]));
            } else if block.starts_with("## ") {
                context.unit_types.push(UnitType::Header(2));
                context.parse_units.push(Arc::from(&input[i..i + 1]));
            } else if block.starts_with("### ") {
                context.unit_types.push(UnitType::Header(3));
                context.parse_units.push(Arc::from(&input[i..i + 1]));
            } else if block.starts_with("#### ") {
                context.unit_types.push(UnitType::Header(4));
                context.parse_units.push(Arc::from(&input[i..i + 1]));
            } else if block.starts_with("![") {
                context.unit_types.push(UnitType::Image);
                context.parse_units.push(Arc::from(&input[i..i + 1]));
            } else if block.starts_with("- ") {
                context.unit_types.push(UnitType::List);
                multiline_state = true;
                multiline_counter = 1;
                block_start = i;
            } else if block.starts_with("$$") {
                context.unit_types.push(UnitType::Latex);
                multiline_state = true;
                multiline_counter = 1;
                block_start = i;
            } else if block.starts_with("```") {
                context.unit_types.push(UnitType::Code);
                multiline_state = true;
                multiline_counter = 1;
                block_start = i;
            } else if block.starts_with(">") {
                context.unit_types.push(UnitType::Callout);
                multiline_state = true;
                multiline_counter = 1;
                block_start = i;
            } else {
                context.unit_types.push(UnitType::Text);
                context.parse_units.push(Arc::from(&input[i..i + 1]));
            }
        }

        if multiline_state {
            let state_type = context.unit_types.last().unwrap();
            match state_type {
                UnitType::List | UnitType::Callout => {
                    context.parse_units.push(Arc::from(
                        &input[block_start..block_start + multiline_counter],
                    ));
                }
                _ => {}
            }
        }

        context
    }
}

fn process_unit(markdown_unit: ParseUnit, unit_type: UnitType, output: &mut String) {
    if let UnitType::Header(level) = unit_type {
        process_header(level, markdown_unit, output);
        return;
    }

    let f = match unit_type {
        UnitType::Text => process_text,
        UnitType::List => process_list,
        UnitType::Image => process_image,
        UnitType::Latex => process_latex,
        UnitType::Code => process_code,
        UnitType::Callout => process_callout,
        _ => process_text,
    };

    f(markdown_unit, output);
}

fn process_text(markdown_unit: ParseUnit, output: &mut String) {
    assert_eq!(markdown_unit.len(), 1);

    let text = markdown_unit.first().unwrap().trim();
    let text = process_inline_formatting(text);
    *output = format!("<p>{}</p>", text);
}

fn process_header(level: Level, markdown_unit: ParseUnit, output: &mut String) {
    assert_eq!(markdown_unit.len(), 1);

    let text = markdown_unit
        .first()
        .unwrap()
        .trim_start_matches('#')
        .trim();
    let text = process_inline_formatting(text);
    *output = format!("<h{}>{}</h{}>", level, text, level);
}

fn process_list(markdown_unit: ParseUnit, output: &mut String) {
    *output = "<ul>\n".to_owned();
    for el in markdown_unit.iter() {
        let text = el.trim().trim_start_matches('-').trim();
        let text = process_inline_formatting(text);
        *output += format!("\t<li>{}</li>\n", text).as_str();
    }
    *output += "</ul>";
}

fn process_image(markdown_unit: ParseUnit, output: &mut String) {
    assert_eq!(markdown_unit.len(), 1);

    let text = markdown_unit.first().unwrap().trim();
    let caption = &text[2..text.find("](").unwrap()];
    let caption = process_inline_formatting(caption);
    let src = &text[text.find("](").unwrap() + 2..text.len() - 1];

    *output = format!(
        r#"<div class="picture">
    <figure>
        <img src="{src}" alt="{caption}">
        <figcaption>{caption}</figcaption>
    </figure>
</div>"#
    );
}

fn process_latex(markdown_unit: ParseUnit, output: &mut String) {
    *output = format!("<p class=\"latex\">{}</p>", markdown_unit.join("\n"));
}

fn process_code(markdown_unit: ParseUnit, output: &mut String) {
    assert!(markdown_unit.len() >= 2);

    let lang = markdown_unit
        .first()
        .unwrap()
        .trim_start_matches('`')
        .trim();
    *output = format!(
        "<pre><code class=\"language-{}\">{}</code></pre>",
        lang,
        markdown_unit[1..markdown_unit.len() - 1].join("\n")
    );
}

fn process_callout(markdown_unit: ParseUnit, output: &mut String) {
    *output = "<div class=\"callout\">\n".to_owned();
    for el in markdown_unit.iter() {
        let text = el.trim().trim_start_matches('-').trim();
        let text = process_inline_formatting(text);
        *output += format!("\t<p>{}</p>\n", text[1..].trim()).as_str();
    }
    *output += "</div>";
}

fn process_inline_formatting(s: impl Into<String>) -> String {
    let mut res = s.into();

    let mut f =
        |markdown_pattern: &str, open_tag: &str, close_tag: &str, check_on_identifiers: bool| {
            let indices = res.match_indices(markdown_pattern).map(|x| x.0);

            let indices = if check_on_identifiers {
                let mut head = false;
                let mut head_failed = false;
                let char_indices = res.char_indices().map(|x| x.0).collect::<Vec<_>>();

                indices
                    .filter(|x| {
                        head = !head;

                        if !head && head_failed {
                            head_failed = false;
                            return false;
                        }

                        let pattern_position =
                            char_indices.iter().position(|val| val == x).unwrap();

                        let determine_index = if head {
                            if pattern_position == 0 {
                                None
                            } else {
                                char_indices.get(pattern_position - 1)
                            }
                        } else {
                            char_indices.get(pattern_position + markdown_pattern.len())
                        };

                        if determine_index.is_none() {
                            return true;
                        }

                        let determine_index = determine_index.unwrap();
                        let determine_char_index = char_indices
                            .iter()
                            .position(|val| val == determine_index)
                            .unwrap();

                        let pass = !res
                            .chars()
                            .nth(determine_char_index)
                            .unwrap()
                            .is_alphanumeric();

                        if head && !pass {
                            head_failed = true
                        }

                        pass
                    })
                    .collect::<Vec<_>>()
            } else {
                indices.collect::<Vec<_>>()
            };

            let mut offset_accum: usize = 0;

            for chunk in indices.chunks_exact(2) {
                res.replace_range(
                    chunk[0] + offset_accum..chunk[0] + offset_accum + markdown_pattern.len(),
                    open_tag,
                );
                offset_accum += open_tag.len() - markdown_pattern.len();
                res.replace_range(
                    chunk[1] + offset_accum..chunk[1] + offset_accum + markdown_pattern.len(),
                    close_tag,
                );
                offset_accum += close_tag.len() - markdown_pattern.len();
            }
        };

    f("***", "<b><i>", "</i></b>", false);
    f("___", "<b><i>", "</i></b>", true);
    f("**", "<b>", "</b>", false);
    f("__", "<b>", "</b>", true);
    f("*", "<i>", "</i>", false);
    f("_", "<i>", "</i>", true);
    f("`", "<code>", "</code>", false);
    f("~~", "<s>", "</s>", false);

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_input() {
        let input = std::fs::read_to_string("sample_data/small_test_input.md").unwrap();
        let generator = Markdown2Html::new(input);
        let res = generator.generate_html();
        println!("{}", res);
    }
}
