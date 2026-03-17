# FlatZinc Serde

`flatzinc-serde` is a Rust library that provides serialization and deserialization support for FlatZinc.
FlatZinc is a representation used to represent decision and optimization problems to solvers of these types of problems.
This format is used by the [MiniZinc](https://www.minizinc.org/) constraint modelling language.

This library supports both the FlatZinc JSON format and the older `.fzn` file format.
The JSON format uses the [`serde`](https://serde.rs/) library, and can (de)serialized using [`serde_json`](https://crates.io/crates/serde_json).
The `serde` dependency can be avoided by disabling the default `serde` feature.
A custom [`winnow`](https://crates.io/crates/winnow) parser is used for the `.fzn` format.
This parser can be used by enabling the `fzn` feature.

## Getting Started

Install the library using `cargo`:

```bash
cargo install flatzinc-serde
```

Read the [documentation](https://docs.rs/flatzinc-serde) for more information about how to use the library.

## License

This project is licensed under the MPL-2.0 License.
See the LICENSE file for details.
