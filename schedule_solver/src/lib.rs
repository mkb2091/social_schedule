#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod scheduler;
mod util;
mod word;
pub use scheduler::*;
