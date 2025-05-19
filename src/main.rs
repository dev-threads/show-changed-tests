use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use clap::Parser;
use git2::Repository;
use show_changed_tests::{changed_test_numbers, format_issue_references, Options};

fn main() {
    let cli = Cli::parse();

    let repo = Repository::open_from_env().unwrap();

    let numbers = changed_test_numbers(&repo, &cli.clone().into());
    let trailer = format_issue_references(&numbers, 72, &format!("{}: ", cli.trailer));

    let Some(message_file) = cli.message_file else {
        // if called without args, assume cli usage and print the trailer
        print!("{trailer}");
        return;
    };

    if !cli
        .source
        .as_ref()
        .is_none_or(|src| src == "template" || src == "message")
    {
        return;
    }

    let mut msg_file = File::options()
        .read(true)
        .write(true)
        .open(&message_file)
        .unwrap();
    let mut message = String::new();
    msg_file.read_to_string(&mut message).unwrap();

    let mut all_lines: Vec<_> = message.lines().collect();
    let (last_empty, _) = all_lines
        .iter()
        .enumerate()
        .rfind(|(_, line)| line.trim().is_empty())
        .unwrap();

    all_lines.insert(last_empty, &trailer);

    let new_contents = all_lines
        .into_iter()
        .fold(String::new(), |mut contents, line| {
            contents += &format!("{line}\n");
            contents
        });

    msg_file.seek(SeekFrom::Start(0)).unwrap();
    msg_file.set_len(0).unwrap();
    msg_file.write_all(new_contents.as_bytes()).unwrap();
}

#[derive(Debug, Parser, Clone)]
struct Cli {
    #[clap(long, default_value = "tc:")]
    prefix: String,

    #[clap(long, default_value = "Tests")]
    trailer: String,

    message_file: Option<PathBuf>,

    source: Option<String>,

    hash: Option<String>,
}

impl From<Cli> for Options {
    fn from(value: Cli) -> Self {
        Self {
            test_prefix: value.prefix,
        }
    }
}
