mod cmd;

use downtown::Markdown2Html;
use std::io::Write;
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    if cmd::has_tag("-h", "--help") {
        println!(
            r#"
downtown.exe -i <input> [-j <number>] [-o <output>]

-i, --input      Input *.md file
-o, --output     Output *.html file. If not specified, a file with same name as <input> file will be
                 created but with .html extension
-j, --threads    Number of threads to run. Default is 4"#
        );
        return Ok(());
    }

    let input_path = cmd::get_path_by_tag("-i", "--input");

    if input_path.is_none() {
        return Err("input markdown file is not specified!")?;
    }

    let input_path = input_path.unwrap();

    if !input_path.exists() {
        return Err("specified markdown file does not exist!")?;
    }

    let output_path =
        cmd::get_path_by_tag("-o", "--output").unwrap_or_else(|| input_path.with_extension("html"));

    let output_path = if let None = output_path.extension() {
        output_path.with_extension("html")
    } else {
        output_path
    };

    let number_of_threads = cmd::get_number_by_tag("-j", "--threads").unwrap_or(0);

    let input = std::fs::read_to_string(input_path)?;
    let mut parser = Markdown2Html::new(input);
    parser.set_number_of_threads(number_of_threads);
    let res = parser.generate_html();

    let mut f = fs::File::create(output_path)?;
    f.write(res.as_bytes())?;

    Ok(())
}
