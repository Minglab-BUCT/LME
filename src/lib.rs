#![doc = include_str!("../README.md")]

/// Chemistry concept data structure and functions
pub mod chemistry;
/// Functions for calling external programs like openbabel and sed
pub mod external;
/// Atom group management
pub mod group_name;
/// Input/Output utils
pub mod io;
/// Layers for storage molecular modeling process
pub mod layer;
/// Basic data structure for LME molecule
pub mod sparse_molecule;
/// Some simple functions used internally
pub mod utils;
pub mod workflow;
