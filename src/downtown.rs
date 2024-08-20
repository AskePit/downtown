use std::sync::Arc;

pub struct Markdown2Html {
    input: Vec<Arc<str>>,
    parse_context: ParseContext,
}

type Level = u8;

#[derive(Debug)]
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
    parse_units: Vec<u16>, // how much input elements do occupy one logical parse unit
    unit_types: Vec<UnitType>,
}

struct GenerationContext<'gen> {
    parse_units: &'gen Vec<u16>, // how much input elements do occupy one logical parse unit
    unit_types: &'gen Vec<UnitType>,
    input: &'gen Vec<Arc<str>>,
    output: Vec<String>,
    cursor: u32,
}

impl Markdown2Html {
    pub fn new(input: String) -> Markdown2Html {
        let input: Vec<_> = input
            .split('\n')
            .filter(|x| !x.trim().is_empty())
            .map(|x| Arc::from(x))
            .collect();

        let parse_context = Markdown2Html::analyze_input(&input);

        // for (c, u) in parse_context.parse_units.iter().zip(parse_context.unit_types.iter()) {
        //     println!("{} {:?}", c, u);
        // }

        Markdown2Html {
            input,
            parse_context,
        }
    }

    pub fn generate_html(&self) -> String {
        let mut output = String::new();

        let mut context = GenerationContext {
            parse_units: &self.parse_context.parse_units,
            unit_types: &self.parse_context.unit_types,
            input: &self.input,
            output: vec!["".to_owned(); self.parse_context.parse_units.len()],
            cursor: 0,
        };

        todo!();

        return output;
    }

    fn analyze_input(input: &Vec<Arc<str>>) -> ParseContext {
        let mut context = ParseContext {
            parse_units: vec![],
            unit_types: vec![],
        };

        let mut multiline_state = false;
        let mut multiline_counter: u16 = 0;

        for block in input {
            if multiline_state {
                let state_type = context.unit_types.last().unwrap();
                match state_type {
                    UnitType::List => {
                        if block.trim_start().starts_with("- ") {
                            multiline_counter += 1;
                            continue;
                        } else {
                            context.parse_units.push(multiline_counter);
                            multiline_state = false;
                        }
                    }
                    UnitType::Callout => {
                        if block.starts_with("> ") {
                            multiline_counter += 1;
                            continue;
                        } else {
                            context.parse_units.push(multiline_counter);
                            multiline_state = false;
                        }
                    }
                    UnitType::Latex => {
                        multiline_counter += 1;

                        if block.starts_with("$$") {
                            context.parse_units.push(multiline_counter);
                            multiline_state = false;
                        }
                        continue;
                    }
                    UnitType::Code => {
                        multiline_counter += 1;

                        if block.starts_with("```") {
                            context.parse_units.push(multiline_counter);
                            multiline_state = false;
                        }
                        continue;
                    }

                    _ => {}
                }
            }

            if block.starts_with("# ") {
                context.unit_types.push(UnitType::Header(1));
                context.parse_units.push(1);
            } else if block.starts_with("## ") {
                context.unit_types.push(UnitType::Header(2));
                context.parse_units.push(1);
            } else if block.starts_with("### ") {
                context.unit_types.push(UnitType::Header(3));
                context.parse_units.push(1);
            } else if block.starts_with("#### ") {
                context.unit_types.push(UnitType::Header(4));
                context.parse_units.push(1);
            } else if block.starts_with("![") {
                context.unit_types.push(UnitType::Image);
                context.parse_units.push(1);
            } else if block.starts_with("- ") {
                context.unit_types.push(UnitType::List);
                multiline_state = true;
                multiline_counter = 1;
            } else if block.starts_with("$$") {
                context.unit_types.push(UnitType::Latex);
                multiline_state = true;
                multiline_counter = 1;
            } else if block.starts_with("```") {
                context.unit_types.push(UnitType::Code);
                multiline_state = true;
                multiline_counter = 1;
            } else if block.starts_with(">") {
                context.unit_types.push(UnitType::Callout);
                multiline_state = true;
                multiline_counter = 1;
            } else {
                context.unit_types.push(UnitType::Text);
                context.parse_units.push(1);
            }
        }

        if multiline_state {
            let state_type = context.unit_types.last().unwrap();
            match state_type {
                UnitType::List | UnitType::Callout => {
                    context.parse_units.push(multiline_counter);
                }
                _ => {}
            }
        }

        context
    }

    fn parse_paragraph(&mut self) {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_input() {
        let input = std::fs::read_to_string("sample_data/small_test_input.md").unwrap();
        let generator = Markdown2Html::new(input);
    }
}
