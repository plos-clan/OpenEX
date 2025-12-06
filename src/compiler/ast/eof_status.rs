
pub enum EofStatus<T> {
    Next(T),
    Eof(T),
}
