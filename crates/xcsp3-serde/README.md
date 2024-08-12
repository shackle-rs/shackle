# XCSP3 Serde

`xcsp3-serde` is a Rust library that provides serialization and deserialization support for the XCSP3 (XML) format using the [`serde`](https://serde.rs/) library.
XCSP3 is a representation used to represent decision and optimization problems.
This format can be used by some solvers of these types of problems, or used to further process the problem.

## Getting Started

Install the library using `cargo`:

```bash
cargo install xcsp3-serde
```

Read the [documentation](https://docs.rs/xcsp3-serde) for more information about how to use the library.

## Limitations

This library currently only supports a subset of the XCSP3 format.
All features of the XCSP3-core format are supported, but many features outside the core format are not yet supported.
It is the aim of this library to support the full XCSP3 format in the future.

## License

This project is licensed under the MPL-2.0 License.
See the LICENSE file for details.
