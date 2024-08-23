use std::{env::args, error::Error};
use downtown::Markdown2Html;

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = args().nth(1).expect("no input file given!");
    let input = std::fs::read_to_string(input_file)?;

    let parser = Markdown2Html::new(input);
    let _res = parser.generate_html();

    Ok(())
}
