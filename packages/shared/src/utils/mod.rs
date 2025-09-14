pub mod date;
pub mod enums;
pub mod fs;
pub mod json;
pub mod string;
pub mod logs;

pub fn with_default<T>(optional_data: Option<T>, default: T) -> T {
    optional_data.unwrap_or(default)
}
