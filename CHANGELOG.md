# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## Unreleased

### Fixed

- Fixed a crash when the commit message was empty
- Fixed newline formatting. Now the trailer is correctly preceded by an empty line
  instead of having two trailing empty lines.
- Fixed crash when a gherkin file failed to parse.
  Files with parse errors are now logged to stderr and skipped.
- Fixed multiple possible crashes when interacting with git and the file system.
  If an error happens, the error is now logged to stderr and the program does otherwise nothing.
  Even in this error case, `show-changed-tests` still exits with a success error code
  so it is still possible to commit normally.

## [1.0.0] - 2025-05-21

### Added
- Initial version
