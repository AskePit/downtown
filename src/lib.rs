mod code_highlighter;
mod configurator;
mod frontmatter_parser;
mod toml_parser;
mod utils;

use crate::code_highlighter::highlight_code;
use crate::configurator::Configurator;
use crate::frontmatter_parser::Frontmatter;
use crate::utils::StrUtils;
use std::cmp::PartialEq;
use std::ops::Range;
use std::sync::Arc;
use std::thread;

type Line = Arc<str>;
type LinesSlice = Arc<[Line]>;
type Block = LinesSlice;

type Level = u8;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum UnitType {
    Header(Level),
    Text,
    List,
    Image,
    Latex,
    Code,
    Blockquote,
    HorizontalLine,
    LocalLink,
    RawText, // e.x. for html tags
}

struct ParsedData {
    units: Vec<Block>,
    unit_types: Vec<UnitType>,
    frontmatter: Option<Frontmatter>,
    title: String,
}

pub struct Markdown2Html {
    parsed_data: ParsedData,
    number_of_threads: u8,
    configurator: Configurator,
}

impl Markdown2Html {
    pub fn new(input: String) -> Markdown2Html {
        let (frontmatter, markdown) = Frontmatter::load(&input);
        let markdown: Vec<_> = markdown.lines().map(|x| Arc::from(x.trim_end())).collect();

        let parse_context = Markdown2Html::parse(markdown, frontmatter);

        Markdown2Html {
            parsed_data: parse_context,
            number_of_threads: 0,
            configurator: Default::default(),
        }
    }

    pub fn new_with_config(
        input: String,
        number_of_threads: u8,
        config_toml: Option<String>,
    ) -> Markdown2Html {
        let (frontmatter, markdown) = Frontmatter::load(&input);
        let markdown: Vec<_> = markdown.lines().map(|x| Arc::from(x.trim_end())).collect();

        let parse_context = Markdown2Html::parse(markdown, frontmatter);

        Markdown2Html {
            parsed_data: parse_context,
            number_of_threads,
            configurator: if let Some(config_toml) = config_toml {
                Configurator::new(config_toml)
            } else {
                Configurator::default()
            },
        }
    }

    pub fn set_number_of_threads(&mut self, number_of_threads: u8) {
        self.number_of_threads = number_of_threads;
    }

    pub fn set_configuration(&mut self, toml_file_content: String) {
        self.configurator = Configurator::new(toml_file_content);
    }

    pub fn generate_html(&self) -> String {
        if self.number_of_threads == 0 {
            // default behaviour
            self.generate_html_multi_threaded(self.number_of_threads)
        } else if self.number_of_threads == 1 {
            self.generate_html_single_threaded()
        } else {
            self.generate_html_multi_threaded(self.number_of_threads)
        }
    }

    fn generate_html_single_threaded(&self) -> String {
        let parse_units = &self.parsed_data.units;
        let unit_types = &self.parsed_data.unit_types;
        let units_size = parse_units.len();

        let mut output_vec = vec!["".to_owned(); units_size];

        for i in 0..units_size {
            let parse_unit = parse_units[i].clone();
            let unit_type = unit_types[i];

            output_vec[i] = process_unit(parse_unit, unit_type, &self.configurator);
        }

        let html_body = output_vec.join("\n");
        self.configurator
            .frame_page(&self.parsed_data.frontmatter, html_body)
    }

    fn generate_html_multi_threaded(&self, number_of_threads: u8) -> String {
        const DEFAULT_NUMBER_OF_THREADS: usize = 4;

        let number_of_threads = if number_of_threads == 0 {
            DEFAULT_NUMBER_OF_THREADS
        } else {
            number_of_threads as usize
        };

        let parse_units = &self.parsed_data.units;
        let unit_types = &self.parsed_data.unit_types;
        let units_size = parse_units.len();

        let chunk_size = (units_size + number_of_threads - 1) / number_of_threads; // Calculate chunk size

        let mut output_vec: Vec<String> = (0..units_size).map(|_| String::new()).collect();

        thread::scope(|scope| {
            let unit_chunks = parse_units.chunks(chunk_size);
            let type_chunks = unit_types.chunks(chunk_size);
            let output_chunks = output_vec.chunks_mut(chunk_size);

            for (unit_chunk, type_chunk, output_chunk) in unit_chunks
                .zip(type_chunks)
                .zip(output_chunks)
                .map(|((a, b), c)| (a, b, c))
            {
                let configurator = self.configurator.clone();

                scope.spawn(move || {
                    for (item, (unit_type, output)) in unit_chunk
                        .iter()
                        .zip(type_chunk.iter().zip(output_chunk.iter_mut()))
                    {
                        *output = process_unit(item.clone(), *unit_type, &configurator);
                    }
                });
            }
        });

        let html_body = output_vec.join("\n");
        self.configurator
            .frame_page(&self.parsed_data.frontmatter, html_body)
    }

    fn parse(input: Vec<Line>, frontmatter: Option<Frontmatter>) -> ParsedData {
        let mut context = ParsedData {
            units: vec![],
            unit_types: vec![],
            title: if let Some(frontmatter) = &frontmatter {
                frontmatter.get_string("title")
            } else {
                String::new()
            },
            frontmatter,
        };

        let mut h1_counter: usize = 0;

        let mut multiline_state = false;
        let mut multiline_counter: usize = 0;
        let mut pattern_size: usize = 0;

        let mut line_start: usize = 0;

        'outer: for (i, line) in input.iter().enumerate() {
            if multiline_state {
                let state_type = context.unit_types.last().unwrap();
                match state_type {
                    UnitType::List => {
                        if line.starts_with("- ") || line.starts_with(' ') || line.trim().is_empty()
                        {
                            multiline_counter += 1;
                            continue;
                        } else {
                            context.units.push(Arc::from(
                                &input[line_start..line_start + multiline_counter],
                            ));
                            multiline_state = false;
                        }
                    }
                    UnitType::Blockquote => {
                        if line.starts_with('>') {
                            multiline_counter += 1;
                            continue;
                        } else {
                            context.units.push(Arc::from(
                                &input[line_start..line_start + multiline_counter],
                            ));
                            multiline_state = false;
                        }
                    }
                    UnitType::Latex => {
                        multiline_counter += 1;

                        if line.starts_with("$$") {
                            context.units.push(Arc::from(
                                &input[line_start..line_start + multiline_counter],
                            ));
                            multiline_state = false;
                        }
                        continue;
                    }
                    UnitType::Code => {
                        multiline_counter += 1;

                        if count_leading_chars(line, '`') == pattern_size {
                            context.units.push(Arc::from(
                                &input[line_start..line_start + multiline_counter],
                            ));
                            multiline_state = false;
                        }
                        continue;
                    }

                    _ => {}
                }
            }

            // multiline patterns
            for (pattern, unit_type) in [
                ("- ", UnitType::List),
                ("$$", UnitType::Latex),
                ("```", UnitType::Code),
                (">", UnitType::Blockquote),
            ] {
                if line.starts_with(pattern) {
                    context.unit_types.push(unit_type);
                    multiline_state = true;
                    multiline_counter = 1;
                    line_start = i;
                    if unit_type == UnitType::Code {
                        pattern_size = count_leading_chars(line, '`');
                    }
                    continue 'outer;
                }
            }

            // one-line patterns
            for (pattern, unit_type) in [
                ("# ", UnitType::Header(1)),
                ("## ", UnitType::Header(2)),
                ("### ", UnitType::Header(3)),
                ("#### ", UnitType::Header(4)),
                ("##### ", UnitType::Header(5)),
                ("###### ", UnitType::Header(6)),
                ("![[", UnitType::LocalLink),
                ("![", UnitType::Image),
                ("---", UnitType::HorizontalLine),
                ("<", UnitType::RawText),
            ] {
                if line.starts_with(pattern) {
                    context.unit_types.push(unit_type);
                    context.units.push(Arc::from(&input[i..i + 1]));

                    if unit_type == UnitType::Header(1) {
                        h1_counter += 1;
                    }
                    continue 'outer;
                }
            }

            if !input[i].is_empty() {
                context.unit_types.push(UnitType::Text);
                context.units.push(Arc::from(&input[i..i + 1]));
            }
        }

        if multiline_state {
            let state_type = *context.unit_types.last().unwrap();
            if state_type == UnitType::List || state_type == UnitType::Blockquote {
                context.units.push(Arc::from(
                    &input[line_start..line_start + multiline_counter],
                ));
            }
        }

        let auto_headers_downgrade = true;
        let auto_insert_header = true;

        if h1_counter > 1 && auto_headers_downgrade {
            context.unit_types = context
                .unit_types
                .into_iter()
                .map(|unit| {
                    if let UnitType::Header(level) = unit {
                        UnitType::Header(level + 1)
                    } else {
                        unit
                    }
                })
                .collect();
        }

        if h1_counter != 1 && auto_insert_header && !context.title.trim().is_empty() {
            context.unit_types.insert(0, UnitType::Header(1));
            context.units.insert(
                0,
                Block::from(vec![Line::from(format!("# {}", context.title))].into_boxed_slice()),
            );
        }

        context
    }
}

fn process_unit(markdown_unit: Block, unit_type: UnitType, configurator: &Configurator) -> String {
    if let UnitType::Header(level) = unit_type {
        return process_header(level, markdown_unit, configurator);
    }

    let f = match unit_type {
        UnitType::Text => process_text,
        UnitType::List => process_list,
        UnitType::Image => process_image,
        UnitType::LocalLink => process_local_link,
        UnitType::Latex => process_latex,
        UnitType::Code => process_code,
        UnitType::Blockquote => process_blockquote,
        UnitType::HorizontalLine => process_horizontal_line,
        UnitType::RawText => process_raw_text,
        _ => process_text,
    };

    f(markdown_unit, configurator)
}

fn process_text(markdown_unit: Block, configurator: &Configurator) -> String {
    assert_eq!(markdown_unit.len(), 1);

    let text = markdown_unit.first().unwrap().trim();
    let text = process_inline_formatting(text, configurator);
    configurator.process_paragraph(&text)
}

fn process_header(level: Level, markdown_unit: Block, configurator: &Configurator) -> String {
    assert_eq!(markdown_unit.len(), 1);

    let text = markdown_unit
        .first()
        .unwrap()
        .trim_start_matches('#')
        .trim();
    let text = process_inline_formatting(text, configurator);
    configurator.process_header(level, &text)
}

fn count_leading_chars(s: &str, c: char) -> usize {
    s.chars().take_while(|&ch| ch == c).count()
}

fn process_list(markdown_unit: Block, configurator: &Configurator) -> String {
    #[derive(PartialEq)]
    enum State {
        NewElementStart,
        FirstLineParsed,
        MultilineElement(usize),
    }

    let mut res = "<ul>\n".to_owned();
    let mut state = State::NewElementStart;
    let mut multiline_range = 0usize..0usize;
    let mut i = 0usize;

    let gen_one_line_html = |multiline_range: &Range<usize>| -> String {
        let text = markdown_unit[multiline_range.start]
            .trim()
            .trim_start_matches('-')
            .trim();
        let text = process_inline_formatting(text, configurator);
        format!("\t<li>{}</li>\n", text)
    };

    let gen_multi_line_html = |lines: &[Line], ident: usize| -> String {
        let sub_doc = lines
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let to_skip = if i == 0 { "- ".len() } else { ident };
                x.chars().skip(to_skip).collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut parser = Markdown2Html::new(sub_doc);
        parser.configurator = configurator.clone();
        parser.configurator.prologue = String::new();
        parser.configurator.epilogue = String::new();

        let html = parser.generate_html_single_threaded();
        format!("\t<li>{}</li>\n", html)
    };

    while i < markdown_unit.len() {
        let line = &markdown_unit[i];
        if line.trim().is_empty() {
            i += 1;
            multiline_range.end += 1;
            continue;
        }

        match state {
            State::NewElementStart => {
                state = State::FirstLineParsed;
                multiline_range.start = i;
            }
            State::FirstLineParsed => {
                if line.starts_with(' ') {
                    let spaces = count_leading_chars(line, ' ');
                    state = State::MultilineElement(spaces);
                    multiline_range.end = i + 1;
                } else {
                    res += &gen_one_line_html(&multiline_range);

                    if line.starts_with("- ") {
                        state = State::NewElementStart;
                        continue; // no `i` advance
                    } else {
                        break; // list end
                    }
                }
            }
            State::MultilineElement(ident) => {
                if line.starts_with(&" ".repeat(ident)) {
                    multiline_range.end += 1;
                } else {
                    // end of multiline
                    res += &gen_multi_line_html(&markdown_unit[multiline_range.clone()], ident);

                    if line.starts_with("- ") {
                        state = State::NewElementStart;
                        continue; // no `i` advance
                    } else {
                        break; // list end
                    }
                }
            }
        }

        i += 1;
    }

    match state {
        State::FirstLineParsed => {
            res += &gen_one_line_html(&multiline_range);
        }
        State::MultilineElement(ident) => {
            res += &gen_multi_line_html(&markdown_unit[multiline_range.clone()], ident);
        }
        _ => {}
    }

    res += "</ul>";
    res
}

fn process_image(markdown_unit: Block, configurator: &Configurator) -> String {
    assert_eq!(markdown_unit.len(), 1);

    let text = markdown_unit.first().unwrap().trim();
    let caption = &text[2..text.find("](").unwrap()];
    let caption = process_inline_formatting(caption, configurator);
    let src = &text[text.find("](").unwrap() + 2..text.len() - 1];

    configurator.process_image(src, &caption)
}

fn process_local_link(markdown_unit: Block, configurator: &Configurator) -> String {
    assert_eq!(markdown_unit.len(), 1);
    let text = markdown_unit.first().unwrap().trim();
    insert_error_element(text, configurator)
}

fn insert_error_element(error_text: &str, configurator: &Configurator) -> String {
    configurator.process_error(error_text)
}

fn process_latex(markdown_unit: Block, configurator: &Configurator) -> String {
    let text = markdown_unit.join("\n");
    configurator.process_latex(&text)
}

fn process_code(markdown_unit: Block, configurator: &Configurator) -> String {
    assert!(markdown_unit.len() >= 2);

    let lang = markdown_unit
        .first()
        .unwrap()
        .trim_start_matches('`')
        .trim();

    let code = markdown_unit[1..markdown_unit.len() - 1].join("\n");
    let code = escape_characters(code);
    let (lang, code) = highlight_code(lang, code.as_str());

    configurator.process_code(&lang, &code)
}

fn process_blockquote(markdown_unit: Block, configurator: &Configurator) -> String {
    let sub_doc = markdown_unit
        .iter()
        .map(|x| x.chars().skip(1).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");

    let mut parser = Markdown2Html::new(sub_doc);
    parser.configurator = configurator.clone();
    parser.configurator.prologue = String::new();
    parser.configurator.epilogue = String::new();

    let html = parser.generate_html_single_threaded();

    configurator.process_blockquote(&html)
}

fn process_horizontal_line(markdown_unit: Block, configurator: &Configurator) -> String {
    assert_eq!(markdown_unit.len(), 1);
    configurator.process_horizontal_line()
}

fn process_raw_text(markdown_unit: Block, _configurator: &Configurator) -> String {
    assert_eq!(markdown_unit.len(), 1);
    markdown_unit.first().unwrap().trim().to_string()
}

fn process_inline_formatting(s: impl Into<String>, configurator: &Configurator) -> String {
    let mut res = s.into();

    res = escape_characters(res);
    res = process_symmetric_inline_pattern(&res, "***", false, |text| {
        configurator.process_italic_bold(text)
    });
    res = process_symmetric_inline_pattern(&res, "___", true, |text| {
        configurator.process_italic_bold(text)
    });
    res =
        process_symmetric_inline_pattern(&res, "**", false, |text| configurator.process_bold(text));
    res =
        process_symmetric_inline_pattern(&res, "__", true, |text| configurator.process_bold(text));
    res = process_symmetric_inline_pattern(&res, "*", false, |text| {
        configurator.process_italic(text)
    });
    res =
        process_symmetric_inline_pattern(&res, "_", true, |text| configurator.process_italic(text));
    res = process_symmetric_inline_pattern(&res, "`", false, |text| {
        configurator.process_code_inline(text)
    });
    res = process_symmetric_inline_pattern(&res, "~~", false, |text| {
        configurator.process_strikethrough(text)
    });
    res = process_links(&res, configurator);

    res
}

fn escape_characters(text: String) -> String {
    text.better_replace("<", "&lt;").better_replace(">", "&gt;")
}

fn byte_index_to_char_index(text: &str, byte_index: usize) -> usize {
    // Count the number of characters up to the given byte index
    text[..byte_index].chars().count()
}

fn process_symmetric_inline_pattern(
    text: &str,
    markdown_pattern: &str,
    check_on_identifiers: bool,
    configurator_method: impl Fn(&str) -> String,
) -> String {
    let mut res = text.to_string();

    let pattern_indices = text.match_indices(markdown_pattern).map(|x| x.0);

    let pattern_indices = if check_on_identifiers {
        let mut head = false;

        pattern_indices
            .filter(|&byte_index| {
                head = !head;

                let pattern_position = byte_index_to_char_index(text, byte_index);

                let determine_char_index = if head {
                    if pattern_position == 0 {
                        return true;
                    }
                    pattern_position - 1
                } else {
                    pattern_position + markdown_pattern.len()
                };

                let determine_char = text.chars().nth(determine_char_index);

                let pass = if determine_char.is_some() {
                    let determine_char = determine_char.unwrap();
                    !determine_char.is_alphanumeric() && determine_char != '_'
                } else {
                    true
                };

                if !pass {
                    head = !head;
                }

                pass
            })
            .collect::<Vec<_>>()
    } else {
        pattern_indices.collect::<Vec<_>>()
    };

    let mut offset_accum: usize = 0;

    for chunk in pattern_indices.chunks_exact(2) {
        let outer_range = chunk[0] + offset_accum..chunk[1] + offset_accum + markdown_pattern.len();
        let inner_range = chunk[0] + offset_accum + markdown_pattern.len()..chunk[1] + offset_accum;

        let processed = configurator_method(&res[inner_range]);
        offset_accum += processed.len() - outer_range.len();
        res.replace_range(outer_range, &processed);
    }

    res
}

fn process_links(text: &str, configurator: &Configurator) -> String {
    let mut res = text.to_string();

    #[derive(PartialEq)]
    enum State {
        None,
        CaptionStart(usize),
        CaptionEnd(usize),
        LinkStart(usize, usize),
    }

    let mut state = State::None;

    let mut to_replace: Vec<(Range<usize>, &str, &str)> = vec![];

    for (i, ch) in text.char_indices() {
        match state {
            State::None => {
                if ch == '[' {
                    state = State::CaptionStart(i);
                }
            }
            State::CaptionStart(start) => {
                if ch == ']' {
                    state = State::CaptionEnd(start);
                }
            }
            State::CaptionEnd(start) => {
                if ch == '(' {
                    state = State::LinkStart(start, i);
                } else {
                    state = State::None;
                    if ch == '[' {
                        state = State::CaptionStart(i);
                    }
                }
            }
            State::LinkStart(caption_start, link_start) => {
                if ch == ')' {
                    let link_end = i;

                    let caption = &text[caption_start + 1..link_start - 1];
                    let link = &text[link_start + 1..link_end];

                    to_replace.push((caption_start..link_end + 1, link, caption));
                    state = State::None;
                }
            }
        }
    }

    let mut offset_accum: usize = 0;
    for (r, link, caption) in to_replace {
        let left_index = r.start + offset_accum;
        let right_index = r.end + offset_accum;

        let processed = configurator.process_link(link, caption);

        offset_accum += processed.len() - r.len();
        res.replace_range(left_index..right_index, &processed);
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    #[ignore]
    fn benchmark() {
        let mut total_time = std::time::Duration::default();
        const TIMES: usize = 4000;

        for _ in 0..TIMES {
            let timer_start = SystemTime::now();

            let input = std::fs::read_to_string("sample_data/small_test_input.md").unwrap();
            let generator = Markdown2Html::new(input);
            let _res = generator.generate_html();

            total_time += SystemTime::now().duration_since(timer_start).unwrap();
        }

        let ms = total_time.as_millis();
        println!("  {} ms", ms);
    }

    #[test]
    fn analyze_input() {
        for file in vec![
            "sample_data/small_test_input.md",
            "sample_data/big_test_input.md",
            "sample_data/code_test_input.md",
        ] {
            let input = std::fs::read_to_string(file).unwrap();
            let mut generator = Markdown2Html::new(input);
            generator.set_number_of_threads(16);
            let _res = generator.generate_html();
            println!("{}", _res);
        }
    }
}
