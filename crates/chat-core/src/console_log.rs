#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsoleLogKind {
    Input,
    Output,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsoleLogRecord {
    pub kind: ConsoleLogKind,
    pub content: String,
}

impl ConsoleLogRecord {
    pub fn new(kind: ConsoleLogKind, content: impl Into<String>) -> Self {
        Self {
            kind,
            content: content.into(),
        }
    }
}
