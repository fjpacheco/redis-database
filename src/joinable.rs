use crate::native_types::ErrorStruct;

pub trait Joinable<T> {
    fn join(&mut self) -> Result<T, ErrorStruct>;
}
