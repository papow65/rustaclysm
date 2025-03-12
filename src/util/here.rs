// This is exported to crate::here
#[macro_export]
/// Returns the location as a `String` of where this macro is used
macro_rules! here {
    // Based on https://doc.rust-lang.org/src/std/macros.rs.html#352-374
    () => {
        format!("[{}:{}:{}]", file!(), line!(), column!())
    };
}
