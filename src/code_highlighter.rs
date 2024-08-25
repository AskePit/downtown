use std::collections::HashSet;

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
    let mut corrected_lang = lang.to_lowercase();
    let mut result = text.to_owned();

    if lang == "js" {
        corrected_lang = "javascript".to_owned();
    } else if lang == "py" {
        corrected_lang = "python".to_owned();
    }

    let keywords = match corrected_lang.as_str() {
        "cpp" | "c" => cplusplus_keywords(),
        "javascript" => javascript_keywords(),
        "python" => python_keywords(),
        "rust" => rust_keywords(),
        _ => all_keywords(),
    };

    highlight_keywords(&keywords, &mut result);

    (corrected_lang, result)
}

fn highlight_keywords(keywords: &HashSet<&str>, text: &mut String) {
    let all_keywords_indices = keywords
        .iter()
        .flat_map(|&keyword| text.match_indices(keyword))
        .collect::<Vec<_>>();

    println!("{:?}", all_keywords_indices);
}
