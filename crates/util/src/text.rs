use std::sync::Arc;

#[must_use]
pub fn uppercase_first(s: Arc<str>) -> Arc<str> {
    let mut c = s.chars();
    match c.next() {
        None => s,
        Some(f) if f.is_uppercase() => s,
        Some(f) => (f.to_uppercase().collect::<String>() + c.as_str()).into(),
    }
}

#[cfg(test)]
mod item_tests {
    use super::*;
    #[test]
    fn zero_works() {
        assert_eq!(uppercase_first(Arc::from("")), Arc::from(""));
    }
    #[test]
    fn one_works() {
        assert_eq!(uppercase_first(Arc::from("a")), Arc::from("A"));
    }
    #[test]
    fn two_works() {
        assert_eq!(uppercase_first(Arc::from("ab")), Arc::from("Ab"));
    }
}
