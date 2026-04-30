//! Prospectus Graphicus — library entry point.
//!
//! *Instrumentum Lineae Iussorum ad Epistulas Prospecti per Graphum Tractandas.*
//!
//! The binary (`prospectus`) is a thin wrapper over this library. Integration
//! tests exercise the same entry points the binary uses.

pub mod auctoritas;
pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod graphus;
pub mod output;

pub use error::{Error, Result};
