#[derive(Debug, Clone)]
pub struct CompilationError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub source_snippet: String,
}

impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error at {}:{}: {}\nSnippet: {}",
            self.line, self.column, self.message, self.source_snippet
        )
    }
}

impl std::error::Error for CompilationError {}
