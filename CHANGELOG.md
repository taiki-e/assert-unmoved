# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

## [0.1.6] - 2024-05-08

- Add `#[must_use]` to constructor and getters.

## [0.1.5] - 2023-08-25

- Track location that `AssertUnmoved` being first pinned-mutably accessed.

- Fix build error from dependency when built with `-Z minimal-versions`.

- Restore compatibility with Rust 1.46 by replacing `pin-project` with `pin-project-lite`.

## [0.1.4] - 2022-02-06

- Use `#[track_caller]`.

  **Note:** This raises the minimum supported Rust version of this crate from Rust 1.37 to Rust 1.46.

- Detect misuse of `AssertUnmoved::get_mut`.

## [0.1.3] - 2021-04-06

- [Apply `doc(cfg(...))` on feature gated APIs.](https://github.com/taiki-e/assert-unmoved/pull/3)

## [0.1.2] - 2021-01-05

- Exclude unneeded files from crates.io.

## [0.1.1] - 2020-12-23

- [Add support for tokio v1.](https://github.com/taiki-e/assert-unmoved/pull/2)

## [0.1.0] - 2020-11-09

Initial release

[Unreleased]: https://github.com/taiki-e/assert-unmoved/compare/v0.1.6...HEAD
[0.1.6]: https://github.com/taiki-e/assert-unmoved/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/taiki-e/assert-unmoved/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/taiki-e/assert-unmoved/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/taiki-e/assert-unmoved/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/taiki-e/assert-unmoved/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/taiki-e/assert-unmoved/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/taiki-e/assert-unmoved/releases/tag/v0.1.0
