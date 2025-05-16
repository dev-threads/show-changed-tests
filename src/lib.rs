use std::{collections::BTreeSet, fs::File, io::Read, ops::Range, path::PathBuf};

use gherkin::{Feature, Span};
use git2::{Diff, DiffOptions, Repository};

/// User configuration to affect the behaviour.
pub struct Options {
    /// Prefix used in tags to link the test case to an item.
    pub test_prefix: String,
}

pub fn changed_test_numbers(repo: &Repository, opts: &Options) -> Vec<u32> {
    let mut diff_opts = DiffOptions::default();
    diff_opts.patience(true).context_lines(0);

    let head = repo
        .resolve_reference_from_short_name("HEAD")
        .unwrap()
        .peel_to_commit()
        .unwrap();
    let tree = head.tree().unwrap();

    let diff = repo
        .diff_tree_to_index(Some(&tree), None, Some(&mut diff_opts))
        .unwrap();

    let changes = changes_in_tests(diff);

    let mut numbers = Vec::new();

    for change in &changes {
        let text = if change.version == Version::Old {
            let blob = tree
                .get_path(&change.path)
                .unwrap()
                .to_object(repo)
                .unwrap()
                .peel_to_blob()
                .unwrap();
            let text = String::from_utf8_lossy(blob.content());
            text.to_string()
        } else {
            let full_path = repo.path().parent().unwrap().join(&change.path);
            let mut text = String::new();
            File::open(&full_path)
                .unwrap()
                .read_to_string(&mut text)
                .unwrap();
            text
        };

        let feature = Feature::parse(&text, Default::default()).unwrap();

        let offsets = calculate_line_spans(&text);
        let changed_line = line_to_byte_offset(offsets.clone(), change.line);

        // Check scenarios
        let scenario = feature
            .scenarios
            .iter()
            .find(|s| s.span.intersects(&changed_line));
        if let Some(scenario) = scenario {
            let testcase = scenario
                .tags
                .iter()
                .find_map(|tag| parse_testcase_number(tag, &opts.test_prefix));
            if let Some(num) = testcase {
                numbers.push(num);
            }
        }

        // Check background
        if let Some(background) = feature.background {
            if background.span.intersects(&changed_line) {
                numbers.extend(feature.scenarios.iter().filter_map(|s| {
                    s.tags
                        .iter()
                        .find_map(|tag| parse_testcase_number(tag, &opts.test_prefix))
                }));
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

    assert!(prefix.len() < width);

    let delimiter = ", ";

    let mut print_delimiter = false;
    let mut current_line = prefix.to_owned();
    for num in numbers {
        let ref_text = format!("#{num}");

        let extra_width = ref_text.len() + if print_delimiter { delimiter.len() } else { 0 };

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
    /// Line number where the change happened, 1 based
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
    fn intersects(&self, other: &Range<usize>) -> bool {
        self.contains(other.start) || self.contains(other.end)
    }
}

impl SpanExt for Span {
    fn contains(&self, val: usize) -> bool {
        self.start <= val && val < self.end
    }
}

type LineOffsets = Vec<Range<usize>>;
fn line_to_byte_offset(offsets: LineOffsets, line: u32) -> Range<usize> {
    offsets[(line - 1) as usize].clone()
}

fn calculate_line_spans(text: &str) -> LineOffsets {
    let mut ptr = 0;
    text.split_inclusive('\n')
        .fold(vec![], |mut offsets, line| {
            let end = ptr + line.len() + 1;
            offsets.push(ptr..end);
            ptr = end;
            offsets
        })
}

fn parse_testcase_number(tag: &str, prefix: &str) -> Option<u32> {
    tag.strip_prefix(prefix)?.parse().ok()
}

impl Default for Options {
    fn default() -> Self {
        Self {
            test_prefix: "tc:".into(),
        }
    }
}
