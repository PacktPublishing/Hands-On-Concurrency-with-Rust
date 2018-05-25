extern crate rand;
#[macro_use]
extern crate rand_derive;
extern crate libc;
// extern crate tempdir;

pub mod individual;
pub mod instruction;
pub mod mars;

#[cfg(test)]
extern crate quickcheck;
