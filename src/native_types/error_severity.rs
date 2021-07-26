#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]

/// Indicates how to proceed depending on
/// the severity of the error
pub enum ErrorSeverity {
    Comunicate,
    CloseClient,
    ShutdownServer,
}
