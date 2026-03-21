pub mod builtin;
mod error;
mod value;

pub use value::Value;

pub struct Interpreter {}

#[cfg(test)]
mod tests {}
