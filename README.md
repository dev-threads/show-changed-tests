# show-changed-tests

Git hook to automatically include tracking numbers of changed [gherkin](https://cucumber.io/) test cases in the commit message.


## What it does

When a test scenario is tagged with a tracking number,
and this case is edited, then the number is added as a trailer to the commit message.

For example, imagine this diff:

```diff
Feature: Withdrawing cash

  @tc:1001
  Scenario: Successful withdrawal within balance
    Given Alice has 234.56 in their account
    When Alice tries to withdraw 200.00
    Then the withdrawal is successful

  @tc:1002
  Scenario: Declined withdrawal in excess of balance
    Given Hamza has 198.76 in their account
+   And Hamzas account overdraft limit is 0
    When Hamza tries to withdraw 200.00
    Then the withdrawal is declined
```

When you then run `git commit`, the editor window looks like this:

``` 

Tests: #1002

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
# ...
```

## Usage & Installation

`show-changed-tests` is intended to run as a `prepare-commit-msg` git hook.
It also provides a hook to be used with [pre-commit](https://pre-commit.com). 

Alternatively `show-changed-tests` can also be run directly on the CLI,
it then prints the trailer to stdout.

### Standalone hook installation

Put `show-changed-tests` into a location on path,
either by downloading a prebuild binary,
or by installing from source via `cargo install --git https://github.com/dev-threads/show-changed-tests`.

Then create a shell script inside your repository at `.git/hooks/prepare-commit-msg`:

```bash
#! /bin/sh
show-changed-tests -- "$@"
```

### Integration via pre-commit

`show-changed-tests` can also be installed via pre-commit:

```yaml
default_install_hook_types: [pre-commit, prepare-commit-msg]
repos:
-   repo: https://github.com/dev-threads/show-changed-tests
    rev: v1.0.1
    hooks:
    -   id: show-changed-tests

```

It is important to include the `default_install_hook_types` line.
Normally `pre-commit` only installs the git pre-commit hook,
but show-changed-tests operates via the `prepare-commit-msg` hook.


## Configuration

`show-changed-tests` is exclusively configured via command line arguments.
For a full list run `show-changed-tests --help`.

In the standalone case, configuration should be performed via the shell wrapper script, e.g.:

```bash
#! /bin/sh
show-changed-tests --trailer="Issues" --prefix="test:" -- "$@"
```

If it is installed via pre-commit, use the `args` key:

```yaml
default_install_hook_types: [pre-commit, prepare-commit-msg]
repos:
-   repo: https://github.com/dev-threads/show-changed-tests
    rev: v1.0.1
    hooks:
    -   id: show-changed-tests
        args: ["--trailer=Issues", "--prefix=test:"]
```

## Known issues

Changes inside the tag list of a test scenario are not detected.
For example if a test is tagged with `@tc:123`
and this is changed to `@tc:456`,
then neither number will appear in the commit message.

## Troubleshooting

### Installation via pre-commit fails

When installing `show-changed-tests` via pre-commit, it is build from source.
pre-commit will use the system Rust installation, if one is available.
If this installation is outdated, then `show-changed-tests` may fail to build,
Rust version 1.82 is required.

In case of an old installation, run `rustup update`.
