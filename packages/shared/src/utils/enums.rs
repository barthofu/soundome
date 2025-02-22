pub enum Match<T> {
    Exact(T),
    Partial(T),
    None
}
