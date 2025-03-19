pub mod fs;
pub mod string;
pub mod date;
pub mod enums;
pub mod json;

pub fn with_default<T>(optional_data: Option<T>, default: T) -> T {
    optional_data.unwrap_or(default)
}