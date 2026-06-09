pub type FileId = usize;

pub struct Source {
    pub name: String,
    pub text: String,
    pub len: usize,
    pub lines: Vec<usize>,
}

impl Source {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let lines = Self::compute_lines(&text);
        Self {
            name: name.into(),
            len: text.len(),
            text,
            lines,
        }
    }

    fn compute_lines(text: &str) -> Vec<usize> {
        let mut lines = vec![0];
        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                lines.push(i + 1);
            }
        }
        lines
    }

    pub fn get_line(&self, line_num: usize) -> String {
        let end = self.lines.get(line_num + 1).unwrap_or(&self.len);
        self.text[self.lines[line_num]..*end]
            .trim_end_matches("\n")
            .to_string()
    }
}

pub struct SourceMap(pub Vec<Source>);
