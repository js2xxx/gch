#![no_std]

#[cfg(feature = "basic")]
extern crate alloc;

mod common;
pub use self::common::*;

#[cfg(feature = "basic")]
pub mod basic;

#[cfg(feature = "flume")]
mod flume;
#[cfg(feature = "flume")]
pub use self::flume::Flume;

#[cfg(feature = "crossbeam")]
mod crossbeam;
#[cfg(feature = "crossbeam")]
pub use self::crossbeam::Crossbeam;
