#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum ErrorSeverity {
    Comunicate,
    CloseClient,
    ShutdownServer,
}
