#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub source: SourceId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Compiler-generated nodes get distinct analysis keys while diagnostics
    /// retain the originating source location. Low bits identify generated roles.
    pub fn synthetic(self, role: usize) -> Self {
        let origin = self.source_origin();
        let start = origin.start.saturating_mul(65536).saturating_add(role);
        Self {
            source: SourceId(usize::MAX / 2 + origin.source.0),
            start,
            end: start.saturating_add(1),
        }
    }
    pub fn source_origin(self) -> Self {
        if self.source.0 >= usize::MAX / 2 {
            let start = self.start / 65536;
            Self {
                source: SourceId(self.source.0 - usize::MAX / 2),
                start,
                end: start.saturating_add(1),
            }
        } else {
            self
        }
    }

    pub fn join(self, other: Self) -> Self {
        assert_eq!(self.source, other.source);
        Self {
            source: self.source,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub notes: Vec<DiagnosticNote>,
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticNote {
    pub span: Span,
    pub message: String,
}
impl Diagnostic {
    pub fn with_note(mut self, span: Span, message: impl Into<String>) -> Self {
        self.notes.push(DiagnosticNote {
            span,
            message: message.into(),
        });
        self
    }
    pub fn render(&self, sources: &[Source]) -> String {
        let origin = self.span.source_origin();
        let mut rendered = sources.get(origin.source.0).map_or_else(
            || format!("{}: {}", self.code, self.message),
            |source| source.render(self),
        );
        for note in &self.notes {
            let span = note.span.source_origin();
            let location = sources
                .get(span.source.0)
                .map(|source| {
                    let (line, column) = source.location(span.start).unwrap_or((1, 1));
                    format!("{}:{line}:{column}", source.name)
                })
                .unwrap_or_else(|| format!("source {}:{}", span.source.0, span.start));
            rendered.push_str(&format!("\n{location}: note: {}", note.message));
        }
        rendered
    }
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            notes: Vec::new(),
            code,
            message: message.into(),
            span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Source {
    pub name: String,
    pub text: String,
    line_starts: Vec<usize>,
}

impl Source {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut starts = vec![0];
        let mut chars = text.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c == '\r' {
                let end = if chars.peek().is_some_and(|(_, next)| *next == '\n') {
                    chars.next();
                    i + 2
                } else {
                    i + 1
                };
                starts.push(end);
            } else if c == '\n' {
                starts.push(i + 1);
            }
        }
        Self {
            name: name.into(),
            text,
            line_starts: starts,
        }
    }

    pub fn location(&self, byte: usize) -> Option<(usize, usize)> {
        if byte > self.text.len() || !self.text.is_char_boundary(byte) {
            return None;
        }
        let line = self.line_starts.partition_point(|start| *start <= byte) - 1;
        Some((
            line + 1,
            self.text[self.line_starts[line]..byte].chars().count() + 1,
        ))
    }

    pub fn render(&self, diagnostic: &Diagnostic) -> String {
        let (line, column) = self
            .location(diagnostic.span.source_origin().start)
            .unwrap_or((1, 1));
        format!(
            "{}:{line}:{column}: error[{}]: {}",
            self.name, diagnostic.code, diagnostic.message
        )
    }
}
