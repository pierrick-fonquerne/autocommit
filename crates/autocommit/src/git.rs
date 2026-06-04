//! Staged diff reading for autocommit.
//!
//! Invokes git to obtain the staged changes and parses them into a
//! [`StagedDiff`]. Parsing is separated from process execution so it can be
//! tested on fixture strings without a real repository.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The staged diff: every changed file with its hunks and counters.
#[derive(Debug, Clone, Default)]
pub struct StagedDiff {
    /// Changed files, in the order git reports them.
    pub files: Vec<FileDiff>,
}

/// A single changed file within the staged diff.
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// Path of the file, relative to the repository root.
    pub path: PathBuf,
    /// How the file changed.
    pub change: ChangeKind,
    /// Contiguous blocks of changed lines (empty for binary files).
    pub hunks: Vec<Hunk>,
    /// Number of added lines.
    pub added: usize,
    /// Number of removed lines.
    pub removed: usize,
    /// Whether git reported the file as binary.
    pub binary: bool,
}

/// How a file changed between the index and HEAD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    /// Newly added file.
    Added,
    /// Modified file.
    Modified,
    /// Deleted file.
    Deleted,
    /// Renamed file, carrying the previous path.
    Renamed {
        /// Previous path before the rename.
        from: PathBuf,
    },
}

/// A contiguous block of changed lines from a unified diff.
#[derive(Debug, Clone)]
pub struct Hunk {
    /// The `@@ ... @@` header line.
    pub header: String,
    /// Raw hunk lines, including the leading ` `, `+`, or `-`.
    pub lines: Vec<String>,
}

/// Errors that can occur while reading the staged diff.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// The git invocation failed or git is not available on `PATH`.
    #[error("git command failed: {0}")]
    Command(String),
    /// The current directory is not inside a git repository.
    #[error("not a git repository")]
    NotARepository,
}

/// Reads the staged diff by invoking git, returning an empty [`StagedDiff`]
/// when nothing is staged.
pub fn read_staged() -> Result<StagedDiff, GitError> {
    let root = repo_root()?;
    let name_status = run_git(&root, &["diff", "--cached", "--name-status"])?;
    if name_status.trim().is_empty() {
        return Ok(StagedDiff::default());
    }
    let unified = run_git(&root, &["diff", "--cached"])?;
    Ok(build_staged_diff(&name_status, &unified))
}

/// Returns the repository root via `git rev-parse --show-toplevel`.
pub fn repo_root() -> Result<PathBuf, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| GitError::Command(format!("cannot run git: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitError::NotARepository);
        }
        return Err(GitError::Command(format!("rev-parse failed: {stderr}")));
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok(PathBuf::from(path))
}

/// Builds a [`StagedDiff`] from the raw outputs of
/// `git diff --cached --name-status` and `git diff --cached`.
///
/// This is the testable core: it takes command outputs as strings so the
/// parsing logic runs without a real repository.
#[must_use]
fn build_staged_diff(name_status: &str, unified: &str) -> StagedDiff {
    let entries = parse_name_status(name_status);
    let mut hunk_map = parse_unified_diff(unified);

    let files = entries
        .into_iter()
        .map(|(path, change)| {
            let diff_data = hunk_map.remove(&path).unwrap_or_default();
            FileDiff {
                path,
                change,
                hunks: diff_data.hunks,
                added: diff_data.added,
                removed: diff_data.removed,
                binary: diff_data.binary,
            }
        })
        .collect();

    StagedDiff { files }
}

/// Intermediate per-file data accumulated while walking the unified diff.
#[derive(Default)]
struct DiffData {
    hunks: Vec<Hunk>,
    added: usize,
    removed: usize,
    binary: bool,
}

/// Parses `git diff --cached --name-status` output into (path, change kind) pairs.
#[must_use]
fn parse_name_status(text: &str) -> Vec<(PathBuf, ChangeKind)> {
    text.lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\t');
            let status = parts.next()?;
            let first = parts.next()?;

            // R<score> = Rename, C<score> = Copy — both have a second path field.
            if status.starts_with('R') {
                let second = parts.next()?;
                Some((
                    PathBuf::from(second),
                    ChangeKind::Renamed {
                        from: PathBuf::from(first),
                    },
                ))
            } else if status.starts_with('C') {
                // Treat copy target as a newly added file.
                let second = parts.next()?;
                Some((PathBuf::from(second), ChangeKind::Added))
            } else {
                let change = match status {
                    "A" => ChangeKind::Added,
                    "D" => ChangeKind::Deleted,
                    _ => ChangeKind::Modified,
                };
                Some((PathBuf::from(first), change))
            }
        })
        .collect()
}

/// Parses `git diff --cached` unified diff output into a map of path → [`DiffData`].
#[must_use]
fn parse_unified_diff(text: &str) -> HashMap<PathBuf, DiffData> {
    let mut map: HashMap<PathBuf, DiffData> = HashMap::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_data = DiffData::default();
    let mut current_hunk: Option<Hunk> = None;

    for line in text.lines() {
        if line.starts_with("diff --git ") {
            if let Some(hunk) = current_hunk.take() {
                current_data.hunks.push(hunk);
            }
            if let Some(path) = current_path.take() {
                map.insert(path, std::mem::take(&mut current_data));
            }
            current_path = parse_diff_git_header(line);
        } else if line.starts_with("Binary files ") && current_path.is_some() {
            current_data.binary = true;
        } else if line.starts_with("@@ ") {
            if let Some(hunk) = current_hunk.take() {
                current_data.hunks.push(hunk);
            }
            current_hunk = Some(Hunk {
                header: line.to_owned(),
                lines: Vec::new(),
            });
        } else if let Some(ref mut hunk) = current_hunk {
            if line.starts_with('+') && !line.starts_with("+++") {
                current_data.added += 1;
                hunk.lines.push(line.to_owned());
            } else if line.starts_with('-') && !line.starts_with("---") {
                current_data.removed += 1;
                hunk.lines.push(line.to_owned());
            } else if !line.starts_with('\\') {
                // Context lines (` ` prefix); skip `\ No newline at end of file`.
                hunk.lines.push(line.to_owned());
            }
        }
    }
    // Flush the last file.
    if let Some(hunk) = current_hunk {
        current_data.hunks.push(hunk);
    }
    if let Some(path) = current_path {
        map.insert(path, current_data);
    }

    map
}

/// Extracts the b-side path from a `diff --git a/<path> b/<path>` header.
///
/// Uses `rfind` so that paths containing the literal string ` b/` are handled
/// correctly (the b-side is always the last such occurrence).
#[must_use]
fn parse_diff_git_header(line: &str) -> Option<PathBuf> {
    let rest = line.strip_prefix("diff --git ")?;
    let b_pos = rest.rfind(" b/")?;
    Some(PathBuf::from(&rest[b_pos + 3..]))
}

/// Runs a git command in `repo_root` and returns its stdout as a `String`.
fn run_git(repo_root: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| GitError::Command(format!("cannot run git: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(GitError::Command(format!(
            "git {} failed: {stderr}",
            args.join(" ")
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| GitError::Command(format!("non-UTF-8 git output: {e}")))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ChangeKind, build_staged_diff};

    #[test]
    fn empty_staging_area_yields_empty_diff() {
        let diff = build_staged_diff("", "");
        assert!(diff.files.is_empty());
    }

    #[test]
    fn modified_file_counts_lines_and_hunks() {
        let name_status = "M\tsrc/main.rs\n";
        let unified = concat!(
            "diff --git a/src/main.rs b/src/main.rs\n",
            "index abc..def 100644\n",
            "--- a/src/main.rs\n",
            "+++ b/src/main.rs\n",
            "@@ -1,3 +1,4 @@\n",
            " fn main() {\n",
            "-    println!(\"old\");\n",
            "+    println!(\"new\");\n",
            "+    println!(\"extra\");\n",
            " }\n",
        );
        let diff = build_staged_diff(name_status, unified);
        assert_eq!(diff.files.len(), 1);
        let f = &diff.files[0];
        assert_eq!(f.path, PathBuf::from("src/main.rs"));
        assert_eq!(f.change, ChangeKind::Modified);
        assert_eq!(f.added, 2);
        assert_eq!(f.removed, 1);
        assert!(!f.binary);
        assert_eq!(f.hunks.len(), 1);
    }

    #[test]
    fn added_and_deleted_files_detected() {
        let name_status = "A\tnew.rs\nD\told.rs\n";
        let unified = concat!(
            "diff --git a/new.rs b/new.rs\n",
            "new file mode 100644\n",
            "--- /dev/null\n",
            "+++ b/new.rs\n",
            "@@ -0,0 +1 @@\n",
            "+fn new() {}\n",
            "diff --git a/old.rs b/old.rs\n",
            "deleted file mode 100644\n",
            "--- a/old.rs\n",
            "+++ /dev/null\n",
            "@@ -1 +0,0 @@\n",
            "-fn old() {}\n",
        );
        let diff = build_staged_diff(name_status, unified);
        assert_eq!(diff.files.len(), 2);
        assert_eq!(diff.files[0].change, ChangeKind::Added);
        assert_eq!(diff.files[1].change, ChangeKind::Deleted);
        assert_eq!(diff.files[0].added, 1);
        assert_eq!(diff.files[1].removed, 1);
    }

    #[test]
    fn renamed_file_carries_original_path() {
        let name_status = "R100\told_name.rs\tnew_name.rs\n";
        let unified = concat!(
            "diff --git a/old_name.rs b/new_name.rs\n",
            "similarity index 100%\n",
            "rename from old_name.rs\n",
            "rename to new_name.rs\n",
        );
        let diff = build_staged_diff(name_status, unified);
        assert_eq!(diff.files.len(), 1);
        let f = &diff.files[0];
        assert_eq!(f.path, PathBuf::from("new_name.rs"));
        assert_eq!(
            f.change,
            ChangeKind::Renamed {
                from: PathBuf::from("old_name.rs"),
            }
        );
        assert!(f.hunks.is_empty());
        assert_eq!(f.added, 0);
        assert_eq!(f.removed, 0);
    }

    #[test]
    fn binary_file_is_flagged() {
        let name_status = "A\tlogo.png\n";
        let unified = concat!(
            "diff --git a/logo.png b/logo.png\n",
            "new file mode 100644\n",
            "index 000..abc\n",
            "Binary files /dev/null and b/logo.png differ\n",
        );
        let diff = build_staged_diff(name_status, unified);
        assert_eq!(diff.files.len(), 1);
        assert!(diff.files[0].binary);
        assert!(diff.files[0].hunks.is_empty());
    }

    #[test]
    fn multiple_hunks_per_file_all_counted() {
        let name_status = "M\tsrc/lib.rs\n";
        let unified = concat!(
            "diff --git a/src/lib.rs b/src/lib.rs\n",
            "index abc..def 100644\n",
            "--- a/src/lib.rs\n",
            "+++ b/src/lib.rs\n",
            "@@ -1,3 +1,3 @@\n",
            " line1\n",
            "-line2\n",
            "+line2_new\n",
            " line3\n",
            "@@ -10,3 +10,3 @@\n",
            " line10\n",
            "-line11\n",
            "+line11_new\n",
            " line12\n",
        );
        let diff = build_staged_diff(name_status, unified);
        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].hunks.len(), 2);
        assert_eq!(diff.files[0].added, 2);
        assert_eq!(diff.files[0].removed, 2);
    }
}
