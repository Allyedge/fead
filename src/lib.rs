pub trait FormatText {
    fn strip_trailing_newline(&self) -> Self;
}

impl FormatText for String {
    fn strip_trailing_newline(&self) -> String {
        self.strip_suffix("\r\n")
            .or(self.strip_suffix('\n'))
            .unwrap_or(self)
            .to_string()
    }
}
