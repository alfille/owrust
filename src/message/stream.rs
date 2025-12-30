//! **owrust** Rust library interfaces with owserver to use 1-wire devices
//!
//! This is a tool in the 1-wire file system **OWFS**
//!
//! This library is the central part of **owrust** -- the _rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## PURPOSE
//! Stream encapsulates the connection to an owserver
//! * handles the persistent connection request (where the Tcp connection is reused if possible for efficiency)
//! * holds a target adress
//!
//! ## EXAMPLES
//! * New connection
//! ```
//! use owrust::error::OwError;
//! use owrust::message::stream ;
//!
//! let mut stream_one_time = owrust::message::stream::Stream::new() ;
//! stream_one_time.set_persistence(false);
//! stream_one_time.set_target("locaalhost:4304");
//! match stream_one_time.connect() {
//!   Ok(_) => (), // connected ok
//!   Err(_) => (), // connection failure
//! }
//!   ```

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

pub use crate::error::{OwEResult, OwError};

/// ### Stream
/// manage the Tcp connections including timeouts and persistance
#[derive(Debug)]
pub struct Stream {
    stream: Option<TcpStream>,
    persist: bool,
    target: String,
}

/// Clone Stream object
/// Creates Stream with same persistance and target but closed connection
impl Clone for Stream {
    fn clone(&self) -> Self {
        Stream {
            stream: None,
            persist: self.persist,
            target: self.target.clone(),
        }
    }
}
/// Default Stream
impl Default for Stream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream {
    /// ### new
    /// Create a new Stream object with minimal properties
    /// * Persistance defaults false
    /// * stream set to None
    /// * target set to None
    pub fn new() -> Self {
        Stream {
            stream: None,
            persist: false,
            target: "localhost:4304".to_string(),
        }
    }

    /// ### set_timeout
    /// Set a 5 second timeout for getting response
    /// * used for connections to an owserver
    /// * ping message should be received as a "keep alive" to show still thinking
    fn set_timeout(&self) -> OwEResult<()> {
        if let Some(stream) = &self.stream {
            stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        }
        Ok(())
    }

    /// ### connect
    /// Connect (via tcp network protocol) to a remote target
    /// * Tests if persistence is on
    ///   * test if connection still works
    /// * returns TcpStream errors or ()
    pub fn connect(&mut self) -> OwEResult<()> {
        if self.stream.is_none() || !self.persist || !self.test() {
            self.stream = None;
            let stream = TcpStream::connect(&self.target)?;
            self.stream = Some(stream);
            self.set_timeout()
        } else {
            Ok(())
        }
    }

    /// ### Set_persistence
    /// Set persistence flag and clear stream for safety
    /// Does not alter target
    pub fn set_persistence(&mut self, persist: bool) {
        self.persist = persist;
    }

    /// ### Set_target
    /// Set target address and clear stream for safety
    /// Does not alter persistence state
    pub fn set_target(&mut self, target: &str) {
        //println!("Setting target: {}", target);
        self.target = target.to_string();
        self.stream = None;
    }

    /// ### get
    /// Get the actual stream for communication
    pub fn get(&mut self) -> Option<&mut TcpStream> {
        self.stream.as_mut()
    }

    /// ### get_persistence
    /// get persistence state for marking message flag
    pub fn get_persistence(&self) -> bool {
        self.persist
    }

    // test the connection (for persistent connctions to see if still valid)
    fn test(&mut self) -> bool {
        match self.stream.as_mut() {
            Some(s) => matches!(s.write_all(&[]), Ok(())), // test existing connection
            _ => false,
        }
    }
}
