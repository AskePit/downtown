use std::collections::HashSet;

#[derive(PartialEq)]
enum HighlightClass {
    Keyword,
    Literal,
    Comment,
    Call,
    DiffAdd,
    DiffRemove,
}

struct MultilineCommentDesc {
    start: &'static str,
    end: &'static str,
}

struct CommentsDesc {
    oneline: Vec<&'static str>,
    multiline: Vec<MultilineCommentDesc>,
}

impl From<HighlightClass> for &str {
    fn from(val: HighlightClass) -> Self {
        match val {
            HighlightClass::Keyword => "<span class=\"code-keyword\">",
            HighlightClass::Literal => "<span class=\"code-literal\">",
            HighlightClass::Comment => "<span class=\"code-comment\">",
            HighlightClass::Call => "<span class=\"code-call\">",
            HighlightClass::DiffAdd => "<span class=\"code-diff-add\">",
            HighlightClass::DiffRemove => "<span class=\"code-diff-remove\">",
        }
    }
}

struct HighlightData {
    highlight_class: HighlightClass,
    start: usize,
    end: usize,
    dead: bool,
}

impl HighlightData {
    fn new(highlight_class: HighlightClass, start: usize, end: usize) -> Self {
        HighlightData {
            highlight_class,
            start,
            end,
            ..Default::default()
        }
    }
}

impl Default for HighlightData {
    fn default() -> Self {
        HighlightData {
            highlight_class: HighlightClass::Keyword,
            start: 0,
            end: 0,
            dead: false,
        }
    }
}

fn cplusplus_keywords() -> HashSet<&'static str> {
    let keywords = [
        "alignas",
        "alignof",
        "and",
        "and_eq",
        "asm",
        "atomic_cancel",
        "atomic_commit",
        "atomic_noexcept",
        "auto",
        "bitand",
        "bitor",
        "bool",
        "break",
        "case",
        "catch",
        "char",
        "char8_t",
        "char16_t",
        "char32_t",
        "class",
        "compl",
        "concept",
        "const",
        "consteval",
        "constexpr",
        "const_cast",
        "continue",
        "co_await",
        "co_return",
        "co_yield",
        "decltype",
        "default",
        "delete",
        "do",
        "double",
        "dynamic_cast",
        "else",
        "enum",
        "explicit",
        "export",
        "extern",
        "false",
        "float",
        "for",
        "friend",
        "goto",
        "if",
        "inline",
        "int",
        "long",
        "mutable",
        "namespace",
        "new",
        "noexcept",
        "not",
        "not_eq",
        "nullptr",
        "operator",
        "or",
        "or_eq",
        "private",
        "protected",
        "public",
        "register",
        "reinterpret_cast",
        "requires",
        "return",
        "short",
        "signed",
        "sizeof",
        "static",
        "static_assert",
        "static_cast",
        "struct",
        "switch",
        "template",
        "this",
        "thread_local",
        "throw",
        "true",
        "try",
        "typedef",
        "typeid",
        "typename",
        "union",
        "unsigned",
        "using",
        "virtual",
        "void",
        "volatile",
        "wchar_t",
        "while",
        "xor",
        "xor_eq",
        "define",    // Defines a macro
        "undef",     // Undefines a macro
        "include",   // Includes a file
        "if",        // Starts a conditional directive
        "ifdef",     // Checks if a macro is defined
        "ifndef",    // Checks if a macro is not defined
        "else",      // Provides an alternative for #if
        "elif",      // Else-if condition for #if
        "endif",     // Ends a conditional directive
        "error",     // Generates a compile-time error
        "pragma",    // Special compiler instructions
        "line",      // Changes the current line number
        "warning",   // Generates a compile-time warning
        "region",    // Marks the start of a region (non-standard)
        "endregion", // Marks the end of a region (non-standard)
    ];
    HashSet::from(keywords)
}

fn rust_keywords() -> HashSet<&'static str> {
    let keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn",
    ];
    HashSet::from(keywords)
}

fn javascript_keywords() -> HashSet<&'static str> {
    let keywords = [
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "let",
        "new",
        "null",
        "return",
        "super",
        "switch",
        "this",
        "throw",
        "true",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "yield",
    ];
    HashSet::from(keywords)
}

fn python_keywords() -> HashSet<&'static str> {
    let keywords = [
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield",
    ];
    HashSet::from(keywords)
}

// Function to calculate the union of all keyword sets
fn all_keywords() -> HashSet<&'static str> {
    let mut all_keywords = cplusplus_keywords();
    all_keywords.extend(rust_keywords());
    all_keywords.extend(javascript_keywords());
    all_keywords.extend(python_keywords());
    all_keywords
}

type Lang = String;
type Code = String;

pub fn highlight_code(lang: &str, text: &str) -> (Lang, Code) {
    let (lang, diff_mode) = if let Some(new_lang) = lang.strip_suffix(" diff") {
        (new_lang, true)
    } else {
        (lang, false)
    };

    let mut corrected_lang = lang.to_lowercase();
    let mut result = text.to_owned();

    if lang == "js" {
        corrected_lang = "javascript".to_owned();
    } else if lang == "py" {
        corrected_lang = "python".to_owned();
    }

    let code_indices = get_highlight_indices(&corrected_lang, &result);
    apply_highlighting(code_indices, &mut result);

    if diff_mode {
        let diff_indices = get_diff_indices(&result);
        apply_highlighting(diff_indices, &mut result);
    }

    (corrected_lang, result)
}

fn get_highlight_indices(lang: &str, text: &String) -> Vec<HighlightData> {
    let keywords = match lang {
        "cpp" | "c" => cplusplus_keywords(),
        "javascript" => javascript_keywords(),
        "python" => python_keywords(),
        "rust" => rust_keywords(),
        _ => all_keywords(),
    };

    let comments_desc: CommentsDesc = match lang {
        "python" => CommentsDesc {
            oneline: vec!["#"],
            multiline: vec![MultilineCommentDesc {
                start: "'''",
                end: "'''",
            }],
        },
        _ => CommentsDesc {
            oneline: vec!["//"],
            multiline: vec![MultilineCommentDesc {
                start: "/*",
                end: "*/",
            }],
        },
    };

    parse_code(text, keywords, comments_desc)
}

// DISCLAIMER:
// ChatGPT-4 written function. Potentially can be optimized
fn parse_code(
    code: &String,
    keywords: HashSet<&str>,
    comments_desc: CommentsDesc,
) -> Vec<HighlightData> {
    let mut highlights = Vec::new();
    let mut i = 0;

    while i < code.len() {
        let remainder = &code[i..];

        // Check for one-line comments
        if comments_desc
            .oneline
            .iter()
            .find_map(|&marker| remainder.strip_prefix(marker))
            .is_some()
        {
            if let Some(end) = remainder.find('\n') {
                highlights.push(HighlightData::new(HighlightClass::Comment, i, i + end));
                i += end;
            } else {
                highlights.push(HighlightData::new(HighlightClass::Comment, i, code.len()));
                break;
            }
        }
        // Check for multiline comments
        else if let Some(end_marker) = comments_desc
            .multiline
            .iter()
            .find_map(|desc| remainder.strip_prefix(desc.start).map(|_| desc.end))
        {
            if let Some(end) = remainder.find(end_marker) {
                highlights.push(HighlightData::new(
                    HighlightClass::Comment,
                    i,
                    i + end + end_marker.len(),
                ));
                i += end + end_marker.len();
            } else {
                highlights.push(HighlightData::new(HighlightClass::Comment, i, code.len()));
                break;
            }
        }
        // Check for literals (strings, chars, numbers)
        else if let Some(stripped) = remainder.strip_prefix('"') {
            if let Some(end) = stripped.find('"') {
                highlights.push(HighlightData::new(
                    HighlightClass::Literal,
                    i,
                    i + end + 2, // Account for the closing quote
                ));
                i += end + 2;
            } else {
                highlights.push(HighlightData::new(HighlightClass::Literal, i, code.len()));
                break;
            }
        } else if let Some(stripped) = remainder.strip_prefix('\'') {
            if let Some(end) = stripped.find('\'') {
                highlights.push(HighlightData::new(
                    HighlightClass::Literal,
                    i,
                    i + end + 2, // Account for the closing quote
                ));
                i += end + 2;
            } else {
                highlights.push(HighlightData::new(HighlightClass::Literal, i, code.len()));
                break;
            }
        } else if remainder.starts_with(|c: char| c.is_ascii_digit()) {
            // Numbers (simple detection of integers and floats)
            let mut end = 0;
            while end < remainder.len()
                && remainder[end..].starts_with(|c: char| c.is_ascii_digit() || c == '.')
            {
                end += 1;
            }
            highlights.push(HighlightData::new(HighlightClass::Literal, i, i + end));
            i += end;
        }
        // Check for keywords
        else if remainder.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
            let mut end = 0;
            while end < remainder.len()
                && remainder[end..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
            {
                end += 1;
            }
            let word = &remainder[..end];
            if keywords.contains(word) {
                highlights.push(HighlightData::new(HighlightClass::Keyword, i, i + end));
            } else if remainder[end..].starts_with('(') {
                // Function call
                highlights.push(HighlightData::new(HighlightClass::Call, i, i + end));
            }
            i += end;
        } else {
            // Advance character by character to avoid breaking UTF-8 sequences
            let next_char = remainder.chars().next().unwrap();
            i += next_char.len_utf8();
        }
    }

    for i in 1..highlights.len() {
        let (left, right) = highlights.split_at_mut(i);
        let left = &mut left[0];
        let right = &mut right[0];

        if left.highlight_class == right.highlight_class && left.end + 1 == right.start {
            left.end = right.end;
            right.dead = true;
        }
    }

    highlights
        .into_iter()
        .filter(|x| !x.dead)
        .collect::<Vec<_>>()
}

fn get_diff_indices(text: &str) -> Vec<HighlightData> {
    let mut highlights = Vec::new();
    let mut line_start = 0;

    for line in text.lines() {
        let line_length = line.len();

        if line.starts_with('+') {
            let line_end = line_start + line_length;
            highlights.push(HighlightData::new(
                HighlightClass::DiffAdd,
                line_start,
                line_end,
            ));
        }

        if line.starts_with('-') {
            let line_end = line_start + line_length;
            highlights.push(HighlightData::new(
                HighlightClass::DiffRemove,
                line_start,
                line_end,
            ));
        }

        // Update the start index for the next line
        line_start += line_length + 1; // +1 for the '\n' character
    }

    highlights
}

fn apply_highlighting(indices: Vec<HighlightData>, text: &mut String) {
    let mut offset_accum: usize = 0;
    for data in indices {
        let start_tag = data.highlight_class.into();
        const END_TAG: &str = "</span>";

        text.insert_str(data.start + offset_accum, start_tag);
        offset_accum += start_tag.len();
        text.insert_str(data.end + offset_accum, END_TAG);
        offset_accum += END_TAG.len();
    }
}
