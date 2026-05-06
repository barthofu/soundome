pub mod date;
pub mod enums;
pub mod fs;
pub mod json;
pub mod logs;
pub mod string;

pub fn with_default<T>(optional_data: Option<T>, default: T) -> T {
    optional_data.unwrap_or(default)
}
