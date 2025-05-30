mod common;

use common::*;
use show_changed_tests::changed_test_numbers;

fn check(files: &[(&'static str, &'static str)], numbers: &[u32]) {
    let mut repo = TestRepository::new();

    for (name, content) in files {
        repo.add_file(name, content);
    }

    assert_eq!(
        &changed_test_numbers(repo.git_repo(), &Default::default()).unwrap(),
        numbers
    );
}

#[test]
fn test_single_change_in_scenario() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: Detect single change in scenario

        @tc:12345
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 12345
          -When a line is changed
          +When this line is changed
          Then 12345 is in the output
        ",
        )],
        &[12345],
    );
}

#[test]
fn change_in_last_line_is_detected() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: Detect single change in scenario

        @tc:12345
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 12345
          When a line is changed
          Then 12345 is in the output
          + And nothing explodes",
        )],
        &[12345],
    );
}

#[test]
fn test_multiple_changes_in_scenario() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: Detect multiple changes in scenario

        @tc:12345
        Scenario: Line in scenario is changed
          -Given a test scenario
          +Given a simple test scenario with number 12345
          -When a line is changed
          +When this line is changed
          Then 12345 is in the output
        ",
        )],
        &[12345],
    );
}

#[test]
fn test_step_is_added() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: A new step is added in a scenario

        @tc:12345
        Scenario: Line in scenario is added
          Given a simple test scenario with number 12345
          When this line is untouched
          +And this line is new
          Then 12345 is in the output
        ",
        )],
        &[12345],
    );
}

#[test]
fn test_step_is_removed() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: An old step is removed in a scenario

        @tc:12345
        Scenario: Line in scenario is removed
          Given a simple test scenario with number 12345
          When this line is untouched
          -And this line is removed
          Then 12345 is in the output
        ",
        )],
        &[12345],
    );
}

#[test]
fn test_single_change_in_multiple_scenarios() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: Detect multiple changes in scenario

        @tc:111
        Scenario: Line in scenario is changed
          -Given a test scenario
          +Given a simple test scenario with number 111
          -When a line is changed
          +When this line is changed
          Then 111 is in the output

        @tc:222
        Scenario: Line in scenario is changed
          -Given a test scenario
          +Given a simple test scenario with number 222
          -When a line is changed
          +When this line is changed
          Then 222 is in the output
        ",
        )],
        &[111, 222],
    );
}

#[test]
fn test_change_in_single_file() {
    check(
        &[
            (
                "Changed.feature",
                "
        Feature: Detect single change in scenario

        @tc:111
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 111
          -When a line is changed
          +When this line is changed
          Then 111 is in the output
        ",
            ),
            (
                "Unchanged.feature",
                "
        Feature: Ignore file that was not changed

        @tc:222
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 222
          When nothing is changed
          Then 222 is not in the output
        ",
            ),
        ],
        &[111],
    );
}

#[test]
fn unstaged_changes_are_not_included() {
    let mut repo = TestRepository::new();
    repo.add_file(
        "Changed.feature",
        "
        Feature: Detect single change in scenario

        @tc:111
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 111
          -When a line is changed
          +When this line is changed
          Then 111 is in the output
        ",
    );
    repo.add_file(
        "Unstaged.feature",
        "
        Feature: Ignore changes not staged for commit

        @tc:222
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 222
          When a line is changed
          +And the line is unstaged
          Then 222 is not in the output
        ",
    );
    repo.git(&["reset", "Unstaged.feature"]);

    assert_eq!(
        changed_test_numbers(repo.git_repo(), &Default::default()).unwrap(),
        vec![111]
    );
}

#[test]
fn test_unchanged_scenario_is_not_listed() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: Don't list unchanged scenario

        @tc:111
        Scenario: Scenario is untouched
          Given a simple test scenario with number 111
          When it is not touched
          Then 111 is not in the output

        @tc:222
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 222
          -When a line is changed
          +When this line is changed
          Then 222 is in the output

        @tc:333
        Scenario: Scenario is untouched
          Given a simple test scenario with number 333
          When it is not touched
          Then 333 is not in the output
        ",
        )],
        &[222],
    );
}

#[test]
fn test_background_change_affects_all_scenarios() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: Background change affects all scenarios

        Background:
            - Given a line in the background changed
            + Given this line in the background changed

        @tc:111
        Scenario: Scenario is untouched
          Given a simple test scenario with number 111
          When it is not touched
          And the background has changed
          Then 111 is in the output

        @tc:222
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 222
          When it is not touched
          And the background has changed
          Then 222 is in the output

        @tc:333
        Scenario: Scenario is untouched
          Given a simple test scenario with number 333
          When it is not touched
          And the background has changed
          Then 333 is in the output
        ",
        )],
        &[111, 222, 333],
    );
}

#[test]
fn gracefully_handle_parse_errors() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: Gracefully handle parse errors

        Background:
            Given a test scenario with number 12345

        @tc:12345
        Scenario: Parse error is fixed
          -And it has a parse error
          +Given the parse error is fixed
          When the gherkin is fixed
          Then 12345 is in the output
        ",
        )],
        &[12345],
    );
}

/// Known issue, the span information does not include the tags at the beginning of the scenario.
#[test]
#[should_panic]
fn test_scenario_number_changes() {
    check(
        &[(
            "SimpleChange.feature",
            "
        Feature: Detect simple changes in scenario

        +@tc:56789
        -@tc:12345
        Scenario: Line in scenario is changed
          Given a simple test scenario with number 12345
          When the number is changed
          Then 12345 and 56789 are in the output
        ",
        )],
        &[12345, 56789],
    );
}

/// This test reproduces a bug in the gherkin parsing library.
#[test]
#[should_panic]
fn repro_span_issue() {
    let text = "
        Feature: Spans are correct

        @Tag1
        @Tag2
        Scenario: I parse a scenario
            Given this scenario
            When I extract then span
            Then the text referenced by the span is this scenario
        ";

    let feature = gherkin::Feature::parse(&text, Default::default()).unwrap();
    let span = feature.scenarios[0].span;

    let scenario = "\
        @Tag1
        @Tag2
        Scenario: I parse a scenario
            Given this scenario
            When I extract then span
            Then the text referenced by the span is this scenario
        ";

    assert_eq!(&text[span.start..span.end], scenario);
}
