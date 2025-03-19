#[derive(Debug, Clone, PartialEq)]
pub enum Match<T> {
    Exact(T),
    Partial(T),
    None,
}
