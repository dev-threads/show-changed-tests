//! Common utility functions for tests

use std::{fs::File, io::Write, process::Command};

use git2::Repository;
use tempfile::TempDir;

pub struct TestRepository {
    git_repo: Repository,
    location: TempDir,
}

impl TestRepository {
    pub fn new() -> Self {
        let location = TempDir::with_prefix("scenario-number-test-").unwrap();
        let git_repo = Repository::init(&location).unwrap();

        Self { git_repo, location }
    }

    /// Create a file in the repository.
    ///
    /// The contents of the file are at the same time a diff,
    /// lines starting with `-` are removed in the current working directory
    /// and lines that start with `+` are new.
    ///
    /// All other lines are commited and unchanged.
    pub fn add_file(&mut self, name: &str, contents: &str) {
        let lines_before: String = contents
            .lines()
            .filter_map(|line| filter_diff(line, DiffKind::Old))
            .map(|line| format!("{line}\n"))
            .collect();
        let lines_after: String = contents
            .lines()
            .filter_map(|line| filter_diff(line, DiffKind::New))
            .map(|line| format!("{line}\n"))
            .collect();
        let mut file = File::create(self.location.path().join(name)).unwrap();
        file.write_all(lines_before.as_bytes()).unwrap();

        self.git(&["add", name]);
        self.git(&["commit", "-m", "Create file", "--no-verify", "--", name]);

        let mut file = File::create(self.location.path().join(name)).unwrap();
        file.write_all(lines_after.as_bytes()).unwrap();

        self.git(&["add", name]);
    }

    pub fn git_repo(&self) -> &Repository {
        &self.git_repo
    }

    /// Run a git command
    pub fn git(&self, cmd: &[&str]) {
        let output = Command::new("git")
            .args(["-c", "commit.gpgsign=false"])
            .args(cmd)
            .current_dir(&self.location)
            .output()
            .unwrap();
        if !output.status.success() {
            panic!(
                "Git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

enum DiffKind {
    Old,
    New,
}

fn filter_diff(line: &str, kind: DiffKind) -> Option<String> {
    let trimmed_line = line.trim_start();
    let (expected, forbidden) = match kind {
        DiffKind::Old => ('-', '+'),
        DiffKind::New => ('+', '-'),
    };

    if trimmed_line.starts_with(forbidden) {
        None
    } else if trimmed_line.starts_with(expected) {
        Some(line.replacen(expected, "", 1))
    } else {
        Some(line.to_owned())
    }
}
