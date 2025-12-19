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

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub use crate::error::{OwEResult, OwError};

#[derive(Debug)]
struct Stream {
    stream: Option<TcpStream>,
    persist: bool,
}

impl Clone for Stream {
    fn clone(&self) -> Self {
        Stream { 
            stream: None,
            persist: self.persist,
        }
    }
}

implr Stream {
    fn new{ persist: bool ) -> Self {
        Stream{
            stream: None,
            persist,
        }
    }
    
    fn set_timeout ( &self ) -> OwEResult<()> {
        match self.stream {
            Some(s) => {
                s.set_read_timeout( Some(Duration::from_secs(5))) ? ;
            },
            None => {
                return Err(OwError::General("No Tcp stream defined".to_string()));
            },
        }
    }
}

    fn get_msg_single(&mut self) -> OwEResult<OwResponse> {
        // Set timeout
        self.set_timeout()?;
        let stream = match self.stream.stream.as_mut() {
            Some(s) => s,
            None => {
                return Err(OwError::General("No Tcp stream defined".to_string()));
            }
        };
        let rcv = OwResponse::get(stream)?;
        Ok(rcv)
    }

    fn set_timeout(&mut self) -> OwEResult<()> {
        match self.stream.stream.as_mut() {
            Some(s) => {
                // Set timeout
                s.set_read_timeout(Some(Duration::from_secs(5)))?;
            }
            None => {
                return Err(OwError::General("No Tcp stream defined".to_string()));
            }
        }
        Ok(())
    }

    // Loop through getting packets until payload empty
    // for directories
    fn get_msg_many(&mut self) -> OwEResult<OwResponse> {
        // Set timeout
        self.set_timeout()?;

        let stream = match self.stream.stream.as_mut() {
            Some(s) => s,
            None => {
                return Err(OwError::General("No Tcp stream defined".to_string()));
            }
        };
        let mut full_rcv = OwResponse::get(stream)?;

        if full_rcv.payload == 0 {
            return Ok(full_rcv);
        }

        loop {
            // get more packets and add content to first one, adjusting payload size
            let mut rcv = OwResponse::get(stream)?;
            if self.debug > 0 {
                eprintln!("Another packet");
            }
            if rcv.payload == 0 {
                return Ok(full_rcv);
            }
            full_rcv.content[(full_rcv.payload - 1) as usize] = b','; // trailing null -> comma
            full_rcv.content.append(&mut rcv.content); // add this packet's info
            full_rcv.payload += rcv.payload;
        }
    }

    fn connect(&mut self) -> OwEResult<()> {
        let stream = TcpStream::connect(&self.owserver)?;
        self.stream.stream = Some(stream);
        Ok(())
    }

    fn send_packet(&mut self, mut msg: OwQuery) -> OwEResult<()> {
        // Write to network
        if self.debug > 1 {
            eprintln!("about to connect");
        }

        // Create or reuse a connection
        if self.persistence {
            match self.stream.stream.as_mut() {
                None => {
                    // need initial connection
                    self.connect()?;
                }
                Some(s) => {
                    // test existing connection
                    match s.write_all(&[]) {
                        Ok(()) => {}
                        Err(_) => {
                            // recreate
                            self.connect()?;
                        }
                    }
                }
            };
        } else {
            // No persistence, make new connection
            self.connect()?;
        }
        if let Some(stream) = &mut self.stream.stream {
            msg.send(stream)?;
        }
        Ok(())
    }

