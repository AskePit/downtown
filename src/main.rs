use downtown::Markdown2Html;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env::args, error::Error, fs};

fn has_tag(short_tag: &str, long_tag: &str) -> bool {
    args()
        .position(|x| x == short_tag)
        .or_else(|| args().position(|x| x == long_tag))
        .is_some()
}

fn get_string_by_tag(short_tag: &str, long_tag: &str) -> Option<String> {
    args()
        .position(|x| x == short_tag)
        .or_else(|| args().position(|x| x == long_tag))
        .map(|x| x + 1)
        .map(|x| args().nth(x))
        .flatten()
}

fn get_path_by_tag(short_tag: &str, long_tag: &str) -> Option<PathBuf> {
    get_string_by_tag(short_tag, long_tag).map(|x| Path::new(x.as_str()).to_owned())
}

fn get_number_by_tag(short_tag: &str, long_tag: &str) -> Option<u8> {
    get_string_by_tag(short_tag, long_tag).map(|x| x.parse::<u8>().unwrap_or(0))
}

fn main() -> Result<(), Box<dyn Error>> {
    if has_tag("-h", "--help") {
        // TODO: help print
        println!("Heeeeelp!");
        return Ok(());
    }

    let input_path = get_path_by_tag("-i", "--input");

    if input_path.is_none() {
        return Err("input markdown file is not specified!")?;
    }

    let input_path = input_path.unwrap();

    if !input_path.exists() {
        return Err("specified markdown file does not exist!")?;
    }

    let output_path =
        get_path_by_tag("-o", "--output").unwrap_or_else(|| input_path.with_extension("html"));

    let number_of_threads = get_number_by_tag("-j", "--threads").unwrap_or(0);

    let input = std::fs::read_to_string(input_path)?;
    let mut parser = Markdown2Html::new(input);
    parser.set_number_of_threads(number_of_threads);
    let res = parser.generate_html();

    let mut f = fs::File::create(output_path)?;
    f.write(res.as_bytes())?;

    Ok(())
}
