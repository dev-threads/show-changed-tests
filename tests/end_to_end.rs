mod common;
use common::*;
use show_changed_tests::changed_test_numbers;

#[test]
fn test_single_change_in_scenario() {
    let mut repo = TestRepository::new();
    repo.add_file(
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
    );

    assert_eq!(changed_test_numbers(repo.git_repo()), vec![12345]);
}

#[test]
fn test_multiple_changes_in_scenario() {
    let mut repo = TestRepository::new();
    repo.add_file(
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
    );

    assert_eq!(changed_test_numbers(repo.git_repo()), vec![12345]);
}

#[test]
fn test_step_is_added() {
    let mut repo = TestRepository::new();
    repo.add_file(
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
    );

    assert_eq!(changed_test_numbers(repo.git_repo()), vec![12345]);
}

#[test]
fn test_step_is_removed() {
    let mut repo = TestRepository::new();
    repo.add_file(
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
    );

    assert_eq!(changed_test_numbers(repo.git_repo()), vec![12345]);
}

#[test]
fn test_single_change_in_multiple_scenarios() {
    let mut repo = TestRepository::new();
    repo.add_file(
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
    );

    assert_eq!(changed_test_numbers(repo.git_repo()), vec![111, 222]);
}

#[test]
fn test_unchanged_scenario_is_not_listed() {
    let mut repo = TestRepository::new();
    repo.add_file(
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
    );

    assert_eq!(changed_test_numbers(repo.git_repo()), vec![222]);
}

#[test]
fn test_background_change_affects_all_scenarios() {
    let mut repo = TestRepository::new();
    repo.add_file(
        "SimpleChange.feature",
        "
        Feature: Don't list unchanged scenario

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
    );

    assert_eq!(changed_test_numbers(repo.git_repo()), vec![111, 222, 333]);
}

#[test]
fn test_scenario_number_changes() {
    let mut repo = TestRepository::new();
    repo.add_file(
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
    );

    assert_eq!(changed_test_numbers(repo.git_repo()), vec![12345, 56789]);
}
