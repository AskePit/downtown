pub trait StrUtils {
    // It's a better version of String::replace since here we create the String with predefined capacity
    fn better_replace(&self, from: &str, to: &str) -> String;
}

impl<T: AsRef<str>> StrUtils for T {
    fn better_replace(&self, from: &str, to: &str) -> String {
        let original = self.as_ref();
        let pattern_growth: isize = to.len() as isize - from.len() as isize;
        let indices = original.match_indices(from).collect::<Vec<_>>();
        let total_growth = pattern_growth * indices.len() as isize;

        let new_size = (original.len() as isize + total_growth) as usize;

        let mut result = String::with_capacity(new_size);

        let mut last_end = 0;
        for (start, part) in indices {
            result.push_str(unsafe { original.get_unchecked(last_end..start) });
            result.push_str(to);
            last_end = start + part.len();
        }
        result.push_str(unsafe { original.get_unchecked(last_end..original.len()) });
        result
    }
}
