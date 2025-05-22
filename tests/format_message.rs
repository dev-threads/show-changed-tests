use show_changed_tests::{extend_message, format_issue_references};

fn check(message: &str, numbers: &[u32], expected: &str) {
    let trailer = format_issue_references(numbers, 72, "Tests: ");

    assert_eq!(&extend_message(message, &trailer), expected);
}


#[test]
fn empty_message_works() {
    check(
"",
        &[123],
"
Tests: #123
"
    );
}

#[test]
fn default_git_message() {
    check(
"
# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
",
        &[123],
"
Tests: #123

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
"
    );
}

#[test]
fn message_with_template() {
    check(
"<--- short issue description in 50 characters -->

<---   long issue description, keep it at 72 characters per line   --->

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
",
        &[123],
"<--- short issue description in 50 characters -->

<---   long issue description, keep it at 72 characters per line   --->

Tests: #123

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
",
    );
}

#[test]
fn long_message_is_wrapped() {
    check(
"<--- short issue description in 50 characters -->

<---   long issue description, keep it at 72 characters per line   --->

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
",
        &(10000..10025).collect::<Vec<_>>(),
"<--- short issue description in 50 characters -->

<---   long issue description, keep it at 72 characters per line   --->

Tests: #10000, #10001, #10002, #10003, #10004, #10005, #10006, #10007
Tests: #10008, #10009, #10010, #10011, #10012, #10013, #10014, #10015
Tests: #10016, #10017, #10018, #10019, #10020, #10021, #10022, #10023
Tests: #10024

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
",
    );
}
