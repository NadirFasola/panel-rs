# Contributing to panel-rs

Thanks for your interest! Please:

1. Fork the repo and create a feature branch:
    ```bash
    git checkout -b feature/<your-feature>
    ```
1. Follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) for commit messages.
1. Run tests & linters before submitting a PR.
    ```bash
    cargo fmr -- --check
    cargo clippy -- -D warnings
    cargo test
    ```
1. Open a PR against `main`. We'll review as soon as possible!
