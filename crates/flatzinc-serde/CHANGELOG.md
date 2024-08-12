# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2024-08-12

### Changed

- The `RangeList` type has been moved to a separate crate called `rangelist`. The `RangeList` type is now re-exported from the `flatzinc-serde` crate.

### Fixed

- Add the missing generic type for the `Annotation` variant of `AnnotationLiteral`.

## [0.3.0] - 2024-05-14

### Changed

- The type `Call` has been split into `AnnotationCall` and `Constraint` to allow the more recursive structure of annotation arguments.

### Added

- Add a `RangeList::iter` when its elements implement the `Copy` trait.
  This method provides an iterator over the intervals of the `RangeList` where the yielded elements are copied.

## [0.2.0] - 2024-04-11

### Changed

- The type for identifiers can now be chosen by the user using a generic type, to allow optimizations like interning.
  The chosen type must implement the `Serialize` and `Deserialize` traits to recursive keep these traits available.
  The default type for identifiers is `String`.

### Added

- All structures now implement `Display` and output the structures in the traditional FlatZinc format.

## [0.1.0] - 2024-01-19

### Added

- Add initial defintion of structs to represent FlatZinc JSON format, where `FlatZinc` is the root struct.

[unreleased]: https://github.com/shackle-rs/shackle/releases/compare/flatzinc-serde-v0.3.0......HEAD
[0.3.0]: https://github.com/shackle-rs/shackle/releases/compare/flatzinc-serde-v0.2.0...flatzinc-serde-v0.3.0
[0.2.0]: https://github.com/shackle-rs/shackle/releases/compare/flatzinc-serde-v0.1.0...flatzinc-serde-v0.2.0
[0.1.0]: https://github.com/shackle-rs/shackle/releases/tag/flatzinc-serde-v0.1.0
