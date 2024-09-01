mod cmd;

use downtown::Markdown2Html;
use std::fs::DirEntry;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{error::Error, fs};

fn visit_dirs(
    dir: &Path,
    cb: &dyn Fn(&DirEntry) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

fn process_dir(
    dir: PathBuf,
    output_name: String,
    config_path: Option<PathBuf>,
    number_of_threads: u8,
) -> Result<(), Box<dyn Error>> {
    visit_dirs(&dir, &|entry| {
        let input_path = entry.path();
        if input_path.is_file() {
            if let Some(ext) = input_path.extension() {
                if ext == "md" {
                    let output_path = input_path.parent().unwrap().join(&output_name);

                    process_file(
                        input_path,
                        output_path,
                        config_path.clone(),
                        number_of_threads,
                    )?;
                }
            }
        }
        Ok(())
    })
}

fn process_file(
    input_path: PathBuf,
    output_path: PathBuf,
    config_path: Option<PathBuf>,
    number_of_threads: u8,
) -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string(input_path)?;
    let config = config_path.map(|x| fs::read_to_string(x).ok()).flatten();

    let parser = Markdown2Html::new_with_config(input, number_of_threads, config);
    let res = parser.generate_html();

    let mut f = fs::File::create(output_path)?;
    f.write(res.as_bytes())?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    if cmd::has_tag("-h", "--help") {
        println!(
            r#"
GENERIC USE
===========

downtown.exe -i <input> [-o <output>]

ALL PARAMETERS
==============

-i, --input      Either a path to *.md file or a path to directory which will be recursively
                 traversed and all *.md files will be processed

-o, --output     If <input> is a path to a file then <output> is a path to output *.html file. If
                 file does not exist it'll be created. If file exists it'll be overwritten. If not
                 specified, a file with same name as <input> file will be created but with .html
                 extension.

                 If <input> is a path to a directory then <output> is treated as a filename to be
                 created besides each processed *.md file

-c, --config     A path to the configuration *.toml file, where you can fine-tune generator behaviour

-j, --threads    Number of threads to run. Default is 4

CONFIGURATION FILE SPEC
=======================

TBD
"#
        );
        return Ok(());
    }

    if cmd::has_tag("-hc", "--help-config") {
        // TODO;
        println!("TBD");
        return Ok(());
    }

    let input_path = cmd::get_path_by_tag("-i", "--input");

    if input_path.is_none() {
        return Err("input markdown file is not specified!")?;
    }

    let input_path = input_path.unwrap();

    if !input_path.exists() {
        return Err("specified input path does not exist!")?;
    }

    let config_path = cmd::get_path_by_tag("-c", "--config");

    let number_of_threads = cmd::get_number_by_tag("-j", "--threads").unwrap_or(0);

    if input_path.is_dir() {
        let output_name =
            cmd::get_string_by_tag("-o", "--output").unwrap_or("index.html".to_owned());

        let output_name = if let None = output_name.strip_suffix(".html") {
            output_name + ".html"
        } else {
            output_name
        };

        process_dir(input_path, output_name, config_path, number_of_threads)?;
    } else if input_path.is_file() {
        let output_path = cmd::get_path_by_tag("-o", "--output")
            .unwrap_or_else(|| input_path.with_extension("html"));

        let output_path = if let None = output_path.extension() {
            output_path.with_extension("html")
        } else {
            output_path
        };

        process_file(input_path, output_path, config_path, number_of_threads)?;
    } else {
        return Err("specified input path is neither file nor directory!")?;
    }

    Ok(())
}
