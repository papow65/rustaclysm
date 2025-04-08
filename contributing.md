# Contributing to Rustaclysm

## How to set up a development environment

See [development setup](readme.md#development-setup)

## How to report bugs

Eplain what you did, what you expected, and what did go wrong.

Please provide as much relevant information as possible when reporting bugs. This may include:

*   Steps to reproduce the bug
*   Error messages
*   Specific game settings
*   System information, especially operating system
*   Application log
*   Save files

## How to submit a pull request

1.  Fork the repository.
2.  Ensure your pull request addresses a single issue or feature. This makes it easier to review and merge.
3.  Make sure to follow these programming guidelines:
  *   **Programming style:**  Please use `cargo clippy` to ensure best practices, and consistency.
  *   **Rust formatting:**  Please use `cargo fmt` with the default settings to ensure consistent code formatting.
  *   **Testing:** Please run all tests to ensure these still succeed:
    * `cargo test --workspace`
    * Or `cargo nextest run --workspace --status-level slow --color always -- --include-ignored`
4.  Submit a pull request.

## Code of conduct

Please follow the [code of conduct](code_of_conduct.md)
