use crate::native_types::ErrorStruct;

pub enum StatusAnswer {
    Continue(Vec<String>),
    Done(Result<String, ErrorStruct>),
    Break(ErrorStruct),
}
