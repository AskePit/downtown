use downtown::Markdown2Html;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env::args, error::Error, fs};

fn get_path_by_tag(short_tag: &str, long_tag: &str) -> Option<PathBuf> {
    args()
        .position(|x| x == short_tag)
        .or_else(|| args().position(|x| x == long_tag))
        .map(|x| x + 1)
        .map(|x| args().nth(x))
        .flatten()
        .map(|x| Path::new(x.as_str()).to_owned())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path =
        get_path_by_tag("-i", "--input");

    if input_path.is_none() {
        return Err("input markdown file is not specified!")?;
    }

    let input_path = input_path.unwrap();

    if !input_path.exists() {
        return Err("specified markdown file does not exist!")?;
    }

    let output_path =
        get_path_by_tag("-o", "--output").unwrap_or_else(|| input_path.with_extension("html"));

    let input = std::fs::read_to_string(input_path)?;
    let parser = Markdown2Html::new(input);
    let res = parser.generate_html();

    let mut f = fs::File::create(output_path)?;
    f.write(res.as_bytes())?;

    Ok(())
}
