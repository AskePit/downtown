use std::sync::Arc;

pub struct Markdown2Html {
    input: Vec<Arc<str>>,
}

struct ParseContext {
    parse_units: Vec<u16>, // how much input elements do occupy one logical parse unit
}

struct GenerationContext<'gen> {
    input: &'gen Vec<Arc<str>>,
    output: Vec<String>,
    cursor: u32
}

impl Markdown2Html {
    pub fn new(input: String) -> Markdown2Html {
        let input: Vec<_> = input.split('\n')
            .filter(|x| !x.trim().is_empty() )
            .map(|x| Arc::from(x))
            .collect();

        Markdown2Html {
            input,
        }
    }

    pub fn generate_html(&self) -> String {
        let mut output = String::new();        

        todo!();

        return output;
    }

    fn analyze_input(&self) -> ParseContext {
        let mut context = ParseContext { parse_units: vec![] };
        for block in &self.input {
            todo!();
        }

        context
    }

    fn parse_paragraph(&mut self) {
        todo!();
    }
}
