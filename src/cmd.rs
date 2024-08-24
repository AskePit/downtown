use std::env::args;
use std::path::{Path, PathBuf};

pub fn has_tag(short_tag: &str, long_tag: &str) -> bool {
    args()
        .position(|x| x == short_tag)
        .or_else(|| args().position(|x| x == long_tag))
        .is_some()
}

pub fn get_string_by_tag(short_tag: &str, long_tag: &str) -> Option<String> {
    args()
        .position(|x| x == short_tag)
        .or_else(|| args().position(|x| x == long_tag))
        .map(|x| x + 1)
        .map(|x| args().nth(x))
        .flatten()
}

pub fn get_path_by_tag(short_tag: &str, long_tag: &str) -> Option<PathBuf> {
    get_string_by_tag(short_tag, long_tag).map(|x| Path::new(x.as_str()).to_owned())
}

pub fn get_number_by_tag(short_tag: &str, long_tag: &str) -> Option<u8> {
    get_string_by_tag(short_tag, long_tag).map(|x| x.parse::<u8>().unwrap_or(0))
}
