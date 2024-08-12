# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2024-08-12

### Changed

- `RangeList` is now provided by a separate crate, `rangelist`.
  This crate now no longer depends on `flatzinc-serde`.

## [0.1.0] - 2024-06-21

### Added

- Add initial definition of structs to (de)serialize XCSP3-core (XML) format, where `Instance` is the root struct.

[unreleased]: https://github.com/shackle-rs/shackle/releases/compare/xcsp3-serde-v0.1.1......HEAD
[0.2.0]: https://github.com/shackle-rs/shackle/releases/compare/xcsp3-serde-v0.1.0...xcsp3-serde-v0.1.1
[0.1.0]: https://github.com/shackle-rs/shackle/releases/tag/xcsp3-serde-v0.1.0
