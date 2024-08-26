mod code_highlighter;

use crate::code_highlighter::highlight_code;
use std::cmp::PartialEq;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::thread;

type Block = Arc<str>;
type BlocksSlice = Arc<[Block]>;
type ParseUnit = BlocksSlice;

pub struct Markdown2Html {
    parse_context: ParseContext,
    number_of_threads: u8,
}

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

        Markdown2Html {
            parse_context,
            number_of_threads: 0,
        }
    }

    pub fn set_number_of_threads(&mut self, number_of_threads: u8) {
        self.number_of_threads = number_of_threads;
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
        let parse_units = &self.parse_context.parse_units;
        let unit_types = &self.parse_context.unit_types;
        let units_size = parse_units.len();

        let mut output_vec = vec!["".to_owned(); units_size];

        for i in 0..units_size {
            let parse_unit = parse_units[i].clone();
            let unit_type = unit_types[i];
            let output = &mut output_vec[i];

            process_unit(parse_unit, unit_type, output);
        }

        output_vec.join("\n")
    }

    fn generate_html_multi_threaded(&self, number_of_threads: u8) -> String {
        const DEFAULT_NUMBER_OF_THREADS: usize = 4;

        let number_of_threads = if number_of_threads == 0 {
            DEFAULT_NUMBER_OF_THREADS
        } else {
            number_of_threads as usize
        };

        let parse_units = &self.parse_context.parse_units;
        let unit_types = &self.parse_context.unit_types;
        let units_size = parse_units.len();

        let chunk_size = (units_size + number_of_threads - 1) / number_of_threads; // Calculate chunk size

        // Wrap each output element in Arc<Mutex<String>> for thread-safe mutability
        let output_vec: Arc<Vec<Arc<Mutex<String>>>> = Arc::new(
            (0..units_size)
                .map(|_| Arc::new(Mutex::new(String::new())))
                .collect(),
        );

        let mut handles = vec![];

        for thread_index in 0..number_of_threads {
            let input_chunk = parse_units
                .iter()
                .skip(thread_index * chunk_size)
                .take(chunk_size)
                .cloned()
                .collect::<Vec<_>>();

            let unit_types_chunk = unit_types
                .iter()
                .skip(thread_index * chunk_size)
                .take(chunk_size)
                .cloned()
                .collect::<Vec<_>>();

            let output_chunk = output_vec
                .iter()
                .skip(thread_index * chunk_size)
                .take(chunk_size)
                .cloned()
                .collect::<Vec<_>>();

            let handle = thread::spawn(move || {
                for (item, (unit_type, output)) in input_chunk
                    .iter()
                    .zip(unit_types_chunk.iter().zip(output_chunk))
                {
                    let mut output = output.lock().unwrap();

                    process_unit(item.clone(), *unit_type, &mut output);
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        let final_output: Vec<String> = Arc::try_unwrap(output_vec)
            .unwrap()
            .into_iter()
            .map(|cell| Arc::try_unwrap(cell).unwrap().into_inner().unwrap().clone())
            .collect();

        final_output.join("\n")
    }

    fn analyze_input(input: &[Block]) -> ParseContext {
        let mut context = ParseContext {
            parse_units: vec![],
            unit_types: vec![],
        };

        let mut multiline_state = false;
        let mut multiline_counter: usize = 0;

        let mut block_start: usize = 0;

        'outer: for (i, block) in input.iter().enumerate() {
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
                    UnitType::Blockquote => {
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

            // multiline patterns
            for (pattern, unit_type) in [
                ("- ", UnitType::List),
                ("$$", UnitType::Latex),
                ("```", UnitType::Code),
                (">", UnitType::Blockquote),
            ] {
                if block.starts_with(pattern) {
                    context.unit_types.push(unit_type);
                    multiline_state = true;
                    multiline_counter = 1;
                    block_start = i;
                    continue 'outer;
                }
            }

            // one-line patterns
            for (pattern, unit_type) in [
                ("# ", UnitType::Header(1)),
                ("## ", UnitType::Header(2)),
                ("### ", UnitType::Header(3)),
                ("#### ", UnitType::Header(4)),
                ("![[", UnitType::LocalLink),
                ("![", UnitType::Image),
                ("---", UnitType::HorizontalLine),
            ] {
                if block.starts_with(pattern) {
                    context.unit_types.push(unit_type);
                    context.parse_units.push(Arc::from(&input[i..i + 1]));
                    continue 'outer;
                }
            }

            {
                context.unit_types.push(UnitType::Text);
                context.parse_units.push(Arc::from(&input[i..i + 1]));
            }
        }

        if multiline_state {
            let state_type = *context.unit_types.last().unwrap();
            if state_type == UnitType::List || state_type == UnitType::Blockquote {
                context.parse_units.push(Arc::from(
                    &input[block_start..block_start + multiline_counter],
                ));
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
        UnitType::LocalLink => process_local_link,
        UnitType::Latex => process_latex,
        UnitType::Code => process_code,
        UnitType::Blockquote => process_blockquote,
        UnitType::HorizontalLine => process_horizontal_line,
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
        r#"<div class="image">
    <figure>
        <img src="{src}" alt="{caption}">
        <figcaption>{caption}</figcaption>
    </figure>
</div>"#
    );
}

fn process_local_link(markdown_unit: ParseUnit, output: &mut String) {
    assert_eq!(markdown_unit.len(), 1);
    let text = markdown_unit.first().unwrap().trim();
    insert_error_element(text, output);
}

fn insert_error_element(error_text: &str, output: &mut String) {
    *output = format!(r#"<div class="parse-error">{error_text}</div>"#);
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

    let code = markdown_unit[1..markdown_unit.len() - 1].join("\n");
    let (lang, code) = highlight_code(lang, code.as_str());

    *output = format!("<pre><code class=\"language-{lang}\">{code}</code></pre>");
}

fn process_blockquote(markdown_unit: ParseUnit, output: &mut String) {
    *output = "<div class=\"blockquote\">\n".to_owned();
    for el in markdown_unit.iter() {
        let text = el.trim().trim_start_matches('-').trim();
        let text = process_inline_formatting(text);
        *output += format!("\t<p>{}</p>\n", text[1..].trim()).as_str();
    }
    *output += "</div>";
}

fn process_horizontal_line(markdown_unit: ParseUnit, output: &mut String) {
    assert_eq!(markdown_unit.len(), 1);
    *output = "<hr>".to_owned();
}

fn process_inline_formatting(s: impl Into<String>) -> String {
    let mut res = s.into();

    process_symmetric_inline_pattern("***", "<b><i>", "</i></b>", false, &mut res);
    process_symmetric_inline_pattern("___", "<b><i>", "</i></b>", true, &mut res);
    process_symmetric_inline_pattern("**", "<b>", "</b>", false, &mut res);
    process_symmetric_inline_pattern("__", "<b>", "</b>", true, &mut res);
    process_symmetric_inline_pattern("*", "<i>", "</i>", false, &mut res);
    process_symmetric_inline_pattern("_", "<i>", "</i>", true, &mut res);
    process_symmetric_inline_pattern("`", "<code>", "</code>", false, &mut res);
    process_symmetric_inline_pattern("~~", "<s>", "</s>", false, &mut res);
    process_links(&mut res);

    res
}

fn byte_index_to_char_index(text: &str, byte_index: usize) -> usize {
    // Count the number of characters up to the given byte index
    text[..byte_index].chars().count()
}

fn process_symmetric_inline_pattern(
    markdown_pattern: &str,
    open_tag: &str,
    close_tag: &str,
    check_on_identifiers: bool,
    text: &mut String,
) {
    let pattern_indices = text.match_indices(markdown_pattern).map(|x| x.0);

    let pattern_indices = if check_on_identifiers {
        let mut head = false;

        pattern_indices
            .filter(|&byte_index| {
                head = !head;

                let pattern_position = byte_index_to_char_index(&text, byte_index);

                let determine_char_index = if head {
                    if pattern_position == 0 {
                        return true;
                    }
                    pattern_position - 1
                } else {
                    pattern_position + markdown_pattern.len()
                };

                let determine_char = text.chars().nth(determine_char_index).unwrap();

                let pass = !determine_char.is_alphanumeric() && determine_char != '_';

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
        let left_index = chunk[0] + offset_accum;
        text.replace_range(left_index..left_index + markdown_pattern.len(), open_tag);
        offset_accum += open_tag.len() - markdown_pattern.len();

        let right_index = chunk[1] + offset_accum;
        text.replace_range(right_index..right_index + markdown_pattern.len(), close_tag);
        offset_accum += close_tag.len() - markdown_pattern.len();
    }
}

fn process_links(text: &mut String) {
    #[derive(PartialEq)]
    enum State {
        None,
        CaptionStart(usize),
        CaptionEnd(usize),
        LinkStart(usize, usize),
    }

    let mut state = State::None;

    let mut to_replace: Vec<(Range<usize>, String)> = vec![];

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

                    to_replace.push((
                        caption_start..link_end + 1,
                        format!("<a href=\"{link}\">{caption}</a>"),
                    ));
                    state = State::None;
                }
            }
        }
    }

    let mut offset_accum: usize = 0;
    for (r, s) in to_replace {
        text.replace_range(r.start + offset_accum..r.end + offset_accum, s.as_str());
        offset_accum += s.len() - r.len()
    }
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

            let input = std::fs::read_to_string("sample_data/big_test_input.md").unwrap();
            let generator = Markdown2Html::new(input);
            let _res = generator.generate_html();

            total_time += SystemTime::now().duration_since(timer_start).unwrap();
        }

        let ms = total_time.as_millis();
        println!("  {} ms", ms);
    }

    #[test]
    fn analyze_input() {
        let input = std::fs::read_to_string("sample_data/code_test_input.md").unwrap();
        let mut generator = Markdown2Html::new(input);
        generator.set_number_of_threads(1);
        let _res = generator.generate_html();
        println!("{}", _res);
    }
}
