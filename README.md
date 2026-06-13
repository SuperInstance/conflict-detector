# Conflict Detector — Git Merge Conflict Scanner

**A conflict detector** scans text for Git merge conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`) and extracts structured information about each conflict region — the "ours" and "theirs" sides, line numbers, and separator position. It's a lightweight parser for the output of a failed `git merge` or `git rebase`.

## Why It Matters

Merge conflicts are the daily reality of collaborative software development. Tools that help resolve them — IDE integrations, PR bots, merge queue processors, and CI checks — all need to detect and parse conflict regions programmatically. Rather than shelling out to `git diff --check` and parsing unstructured output, this crate provides a direct text scanner that returns structured `Conflict` objects. It's useful for pre-commit hooks (block commits with unresolved conflicts), automated merge resolution bots, and developer tooling that highlights conflict regions in editors.

## How It Works

### Conflict Marker Format

Git uses seven-character delimiters:

```
<<<<<<< ours
foo
=======
bar
>>>>>>> theirs
```

| Marker | Meaning |
|---|---|
| `<<<<<<< name` | Start of conflict; `name` = current branch/ref |
| `=======` | Separator between "ours" (above) and "theirs" (below) |
| `>>>>>>> name` | End of conflict; `name` = incoming branch/ref |

### State Machine Parser

The scanner is a 3-state finite automaton:

```
State 0 (idle) ──── sees "<<<<<<<" ────► State 1 (in_ours)
State 1 (in_ours) ── sees "=======" ──► State 2 (in_theirs)
State 2 (in_theirs) ── sees ">>>>>>>" ──► State 0 (idle) + emit Conflict
```

Lines in state 1 are accumulated into `ours_buf`. Lines in state 2 (between separator and end marker) are collected into `theirs`. On reaching the end marker, a `Conflict` is emitted with:

- `start_line`: index of `<<<<<<<`
- `separator_line`: index of `=======`
- `end_line`: index of `>>>>>>>`
- `ours`: text between start and separator
- `theirs`: text between separator and end

**Complexity**: `O(L)` where `L` = total lines in the input text. Single pass, no backtracking. Each line is compared against at most one marker prefix per state.

### Edge Cases

- **Nested markers**: Git doesn't produce nested conflicts, but the parser handles them by treating inner `<<<<<<<` as part of the current `ours` section.
- **Malformed conflicts**: If `>>>>>>>` is missing, the parser stays in state 2 until EOF and does not emit a partial conflict — matching Git's own behavior.
- **No conflict markers**: Returns `ScanResult { has_conflicts: false, conflicts: [] }`.

## Quick Start

```rust
use conflict_detector::{scan, count_conflicts};

let text = r#"
before
<<<<<<< ours
fn foo() { 1 }
=======
fn foo() { 2 }
>>>>>>> theirs
after
"#;

let result = scan(text);
assert!(result.has_conflicts);
assert_eq!(result.conflicts.len(), 1);
assert_eq!(result.conflicts[0].ours, "fn foo() { 1 }");
assert_eq!(result.conflicts[0].theirs, "fn foo() { 2 }");

// Quick check
assert_eq!(count_conflicts(text), 1);
```

## API

| Function / Type | Description |
|---|---|
| `scan(text)` | Parse text for conflict markers → `ScanResult`. `O(L)`. |
| `count_conflicts(text)` | Count conflict regions → `usize`. |
| `ScanResult` | `{ conflicts: Vec<Conflict>, has_conflicts: bool }`. |
| `Conflict` | `{ start_line, separator_line, end_line, ours: String, theirs: String }`. |

## Architecture Notes

Conflict detection is a development tool in the SuperInstance ecosystem, serving the γ (generation) side of γ + η = C. It automates merge quality checks in CI pipelines, preventing broken merges from reaching production. See [SuperInstance Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. Git Documentation. *Git Tools - Advanced Merging*. <https://git-scm.com/book/en/v2/Git-Tools-Advanced-Merging>
2. Loeliger, J. & McCullough, M. (2012). *Version Control with Git* (2nd ed.). O'Reilly.
3. Aho, A. et al. (2006). *Compilers: Principles, Techniques, and Tools* (2nd ed.), Ch. 3: Lexical Analysis. — State-machine parsing.

## License

MIT
