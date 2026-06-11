//! conflict-detector — Detects merge conflicts in text and provides diagnostics.
//!
/// Scans content for standard conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
/// and reports structured conflict regions.

/// A single conflict region found in text.
#[derive(Debug, Clone)]
pub struct Conflict {
    /// 0-based line index where `<<<<<<<` appears.
    pub start_line: usize,
    /// 0-based line index where separator `=======` appears.
    pub separator_line: usize,
    /// 0-based line index where `>>>>>>>` appears.
    pub end_line: usize,
    /// Content of the "ours" side.
    pub ours: String,
    /// Content of the "theirs" side.
    pub theirs: String,
}

/// Result of scanning for conflicts.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub conflicts: Vec<Conflict>,
    pub has_conflicts: bool,
}

/// Scans text for conflict markers.
pub fn scan(text: &str) -> ScanResult {
    let mut conflicts = Vec::new();
    let mut state = 0usize; // 0=idle, 1=in_ours, 2=in_theirs
    let mut start = 0usize;
    let mut sep = 0usize;
    let mut ours_buf = String::new();

    for (i, line) in text.lines().enumerate() {
        if state == 0 && line.starts_with("<<<<<<< ") {
            state = 1;
            start = i;
            ours_buf.clear();
        } else if state == 1 && line.trim() == "=======" {
            state = 2;
            sep = i;
        } else if state == 2 && line.starts_with(">>>>>>> ") {
            let ours = ours_buf.trim_end().to_string();
            // Collect theirs lines between sep+1 and i
            let theirs_lines: Vec<&str> = text.lines().skip(sep + 1).take(i - sep - 1).collect();
            let theirs = theirs_lines.join("\n");
            conflicts.push(Conflict {
                start_line: start,
                separator_line: sep,
                end_line: i,
                ours,
                theirs,
            });
            state = 0;
        } else if state == 1 {
            ours_buf.push_str(line);
            ours_buf.push('\n');
        }
    }

    let has_conflicts = !conflicts.is_empty();
    ScanResult { conflicts, has_conflicts }
}

/// Counts conflicts in text.
pub fn count_conflicts(text: &str) -> usize {
    scan(text).conflicts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_conflicts() {
        let result = scan("hello\nworld\n");
        assert!(!result.has_conflicts);
    }

    #[test]
    fn one_conflict() {
        let text = "before\n<<<<<<< ours\nfoo\n=======\nbar\n>>>>>>> theirs\nafter\n";
        let result = scan(text);
        assert!(result.has_conflicts);
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].ours, "foo");
        assert_eq!(result.conflicts[0].theirs, "bar");
    }
}
