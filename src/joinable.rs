use crate::native_types::ErrorStruct;
/// This trait implementa a safe way to close
/// structures that have active threads and/or
/// channels which connect the structure with
/// other threads.
pub trait Joinable<T> {
    /// Does the safe close
    fn join(&mut self) -> Result<T, ErrorStruct>;
}
