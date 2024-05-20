pub(crate) fn uppercase_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod item_tests {
    use super::*;
    #[test]
    fn zero_works() {
        assert_eq!(uppercase_first(""), "");
    }
    #[test]
    fn one_works() {
        assert_eq!(uppercase_first("a"), "A");
    }
    #[test]
    fn two_works() {
        assert_eq!(uppercase_first("ab"), "Ab");
    }
}
