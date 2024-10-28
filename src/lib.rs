use std::{collections::BTreeSet, fs::File, io::Read, path::PathBuf};

use gherkin::{Feature, Span};
use git2::{Diff, DiffOptions, Repository};

pub fn changed_test_numbers(repo: &Repository) -> Vec<u32> {
    let mut opts = DiffOptions::default();
    opts.patience(true).context_lines(0);
    let diff = repo.diff_index_to_workdir(None, Some(&mut opts)).unwrap();

    let changes = changes_in_tests(diff);

    let mut numbers = Vec::new();

    for change in &changes {
        if change.version == Version::Old {
            // TODO(Johannes Pieger,2025-05-14): Parse old file
            println!("Skipping {change:?}");
            continue;
        }
        let full_path = repo.path().parent().unwrap().join(&change.path);
        let mut text = String::new();
        File::open(&full_path)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();

        let feature = Feature::parse(&text, Default::default()).unwrap();

        let offsets = calculate_line_offsets(&text);
        let changed_line = line_to_byte_offset(offsets, change.line);

        // Check scenarios
        let scenario = feature
            .scenarios
            .iter()
            .find(|s| s.span.contains(changed_line));
        if let Some(scenario) = scenario {
            let testcase = scenario
                .tags
                .iter()
                .find_map(|tag| parse_testcase_number(&tag));
            if let Some(num) = testcase {
                numbers.push(num);
            }
        }

        // Check background
        if let Some(background) = feature.background {
            if background.span.contains(changed_line) {
                numbers.extend(
                    feature
                        .scenarios
                        .iter()
                        .filter_map(|s| s.tags.iter().find_map(|tag| parse_testcase_number(&tag))),
                );
            }
        }
    }

    // collect into hashset and back into vec to get rid of duplicates
    // This also sorts the numbers
    numbers = numbers
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    numbers
}

pub fn print_issue_references(numbers: &[u32], width: usize, prefix: &str) {
    let mut lines = Vec::new();

    assert!(prefix.len() < width as usize);

    let delimiter = ", ";

    let mut print_delimiter = false;
    let mut current_line = prefix.to_owned();
    for num in numbers {
        let ref_text = format!("#{num}");

        let extra_width = ref_text.len() + print_delimiter.then(|| delimiter.len()).unwrap_or(0);

        if current_line.len() + extra_width > width {
            lines.push(current_line);
            current_line = prefix.to_owned();
            print_delimiter = false;
        }

        if print_delimiter {
            current_line.push_str(delimiter);
        }

        current_line.push_str(&ref_text);
        print_delimiter = true;
    }

    lines.push(current_line);

    for line in lines {
        println!("{line}");
    }
}

#[derive(Debug, Clone)]
struct Change {
    pub line: u32,
    pub path: PathBuf,
    /// Whether to check the previous or changed version of the file.
    /// E.g. pure deletions should be checked in the `Old` version,
    /// pure additions in the `New`.
    pub version: Version,

    // Newly added line, for debugging.
    #[allow(dead_code)]
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Version {
    Old,
    New,
}

fn changes_in_tests(diff: Diff) -> Vec<Change> {
    let mut result = Vec::new();

    let _ = diff.foreach(
        &mut |_, _| true,
        None,
        None,
        Some(&mut |file, _, line| {
            if file
                .new_file()
                .path()
                .is_none_or(|p| p.extension().is_none_or(|e| e != "feature"))
            {
                return true;
            }

            let Some(path) = file.new_file().path().map(ToOwned::to_owned) else {
                return true;
            };

            let text = String::from_utf8_lossy(line.content()).to_string();

            if text.trim().is_empty() {
                return true;
            }

            let change = match (line.old_lineno(), line.new_lineno()) {
                (None, Some(line)) => Change {
                    line,
                    path,
                    version: Version::New,
                    text,
                },
                (Some(_), Some(line)) => Change {
                    line,
                    path,
                    version: Version::New,
                    text,
                },
                (Some(line), None) => Change {
                    line,
                    path,
                    version: Version::Old,
                    text,
                },
                (None, None) => return true,
            };

            result.push(change);

            true
        }),
    );

    result
}

trait SpanExt {
    fn contains(&self, val: usize) -> bool;
}

impl SpanExt for Span {
    fn contains(&self, val: usize) -> bool {
        self.start <= val && val < self.end
    }
}

type LineOffsets = Vec<usize>;
fn line_to_byte_offset(offsets: LineOffsets, line: u32) -> usize {
    offsets[(line) as usize]
}

fn calculate_line_offsets(text: &str) -> LineOffsets {
    text.split_inclusive('\n')
        .fold(vec![0], |mut offsets, line| {
            offsets.push(offsets.last().unwrap() + line.len());
            offsets
        })
}

fn parse_testcase_number(tag: &str) -> Option<u32> {
    tag.strip_prefix("tc:")?.parse().ok()
}
