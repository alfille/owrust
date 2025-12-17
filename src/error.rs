//! **owrust** Rust library interfaces with owserver to use 1-wire devices
//!
//! This is a tool in the 1-wire file system **OWFS**
//!
//! This library is the central part of **owrust** -- the _rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## PURPOSE
//! lib.rs is the library code that actually performs the **owserver protocol**.
//! Communication with **owserver** is over TCP/IP (network) using an efficient well-documented protocol.
//!
//! Supported operations are read, write, dir, present and size, with some variations
//!
//! The main struct is OwMessage which holds all the configuration information.
//! Typically it is populated by the command line or configuration files
//!
//! ## EXAMPLES
//! ```
//! use owrust ; // basic library
//! use owrust::parse_args ; // configure from command line, file or OsString
//!
//! let mut owserver = owrust::new() ; // create an OwMessage struct
//!   // configure from command line and get 1-wire paths
//! let paths = parse_args::command_line( &mut owserver ) ;
//!   // Call any of the OwMessage functions like dir, read, write,...
//!   ```

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use std::fmt;

/// ### OwEResult
///
/// type alias for Result<_,OwError> to reduce boilerplate
/// `OwEResult<String>` is equivalent to `Result<String,OwError>`
pub type OwEResult<T> = std::result::Result<T, OwError>;

#[derive(Debug)]
/// ### OwError
/// the **owrust**-specific error type
///
/// details field is a String with error details
pub enum OwError {
    General(String),
    Input(String),
    Output(String),
    Io(std::io::Error),
    Args(pico_args::Error),
    Numeric(String),
    Text(String),
}

impl fmt::Display for OwError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OwError::General(e) => write!(f, "An error: {}", e),
            OwError::Input(e) => write!(f, "Input error: {}", e),
            OwError::Output(e) => write!(f, "Output error: {}", e),
            OwError::Io(e) => write!(f, "IO error: {}", e),
            OwError::Args(e) => write!(f, "Args error: {}", e),
            OwError::Text(e) => write!(f, "Text conversion error: {}", e),
            OwError::Numeric(e) => write!(f, "Non-numeric characters: {}", e),
        }
    }
}
impl std::error::Error for OwError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OwError::Io(e) => Some(e),
            OwError::Args(e) => Some(e),
            _ => None,
        }
    }
}

use std::convert::From;
use std::io;
impl From<OwError> for io::Error {
    fn from(error: OwError) -> Self {
        io::Error::other(error.to_string())
    }
}
impl From<std::io::Error> for OwError {
    fn from(e: std::io::Error) -> Self {
        OwError::Io(e)
    }
}
impl From<pico_args::Error> for OwError {
    fn from(e: pico_args::Error) -> Self {
        OwError::Args(e)
    }
}
impl From<std::str::Utf8Error> for OwError {
    fn from(_e: std::str::Utf8Error) -> Self {
        OwError::Text("Utf8 Error".into())
    }
}
impl From<std::string::FromUtf8Error> for OwError {
    fn from(_e: std::string::FromUtf8Error) -> Self {
        OwError::Text("FromUTF8Error".into())
    }
}
impl From<std::ffi::NulError> for OwError {
    fn from(_e: std::ffi::NulError) -> Self {
        OwError::Text("Nul Error".into())
    }
}
