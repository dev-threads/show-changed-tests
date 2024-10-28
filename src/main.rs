use git2::Repository;
use show_changed_tests::{changed_test_numbers, print_issue_references};

fn main() {
    let repo = Repository::open_from_env().unwrap();

    let numbers = changed_test_numbers(&repo);

    print_issue_references(&numbers, 72, "Work-Items: ");
}
