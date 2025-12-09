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
//! The main struct is OwClient which holds all the configuration information.
//! Typically it is populated by the command line or configuration files
//!
//! ## EXAMPLES
//! ```
//! use owrust ; // basic library
//! use owrust::parse_args ; // configure from command line, file or OsString
//!
//! let mut owserver = owrust::new() ; // create an OwClient struct
//!   // configure from command line and get 1-wire paths
//! let paths = parse_args::command_line( &mut owserver ) ;
//!   // Call any of the OwClient functions like dir, read, write,...
//!   ```
  
// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use std::io::{Read,Write} ;
use std::net::TcpStream ;
use std::time::Duration ;
use std::str ;

mod token ;
use token::make_token ;

pub use crate::error::{OwError,OwEResult};

#[derive(Debug,PartialEq,Clone)]
/// ### Temperature scale
/// sent to owserver in the flag parameter since only the original 1-wire 
/// program in the chain knows the type of value being sought
pub enum Temperature {
    CELSIUS,
    FARENHEIT,
    KELVIN,
    RANKINE,
    DEFAULT,
}

#[derive(Debug,PartialEq,Clone)]
/// ### Pressure scale
/// sent to owserver in the flag parameter since only the original 1-wire 
/// program in the chain knows the type of value being sought
pub enum Pressure {
    MMHG,
    INHG,
    PA,
    PSI,
    ATM,
    MBAR,
    DEFAULT,
}

#[derive(Debug,PartialEq,Clone)]
/// ### 1-wire ID format
/// has components:
///  F family code (1 byte)
///  I unique serial number (6 bytes)
///  C checksum (1-byte)
pub enum Format {
    FI,
    FdI,
    FIC,
    FIdC,
    FdIC,
    FdIdC,
    DEFAULT,
}

#[derive(Debug)]
struct Stream {
    stream: Option<TcpStream>,
}
impl Clone for Stream {
    fn clone( &self ) -> Self {
        Stream{ stream: None }
    }
}

#[derive(Debug,Clone)]
/// ### OwClient
/// structure that manages the connection to owserver
/// * Stores configuration settings
/// * has public fuction for each message type to owserver
///   * read
///   * write
///   * dir
/// * convenience functions for printing results
/// ### Creation
/// ```
/// let mut owserver = owrust::new() ;
/// ```
pub struct OwServer {
    owserver:    String,
    temperature: Temperature,
    pressure:    Pressure,
    format:      Format,
    size:        u32,
    offset:      u32,
    slash:       bool,
    hex:         bool,
    bare:        bool,
    prune:       bool,
    persistence: bool,
    stream:      Stream,
    debug:       u32,
    flags:       u32,
}

