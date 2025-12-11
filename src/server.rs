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
use std::net::{TcpStream,TcpListener} ;
use std::thread ;
use std::time::Duration ;
use std::str ;

mod token ;
use token::make_token ;

pub use crate::error::{OwError,OwEResult};

#[derive(Debug)]
/// ### OwServer
/// structure that manages this owserver
/// ### Creation
/// ```
/// let mut owserver = OwServer::new("localhost.4304".to_string()) ;
/// ```
pub struct OwServer {
	client: crate::OwClient,
    listen_stream: TcpListener,
    token: [u8;16],
}
    
impl OwServer {
    pub fn new( client: crate::OwClient, address: &str ) -> OwEResult<OwServer> {
        Ok(OwServer {
			client: client.clone(),
            listen_stream: TcpListener::bind(address)?,
            token: make_token(),
        })
    }
    pub fn serve(&self) -> OwEResult<()> {
        for stream in self.listen_stream.incoming() {
            match stream {
                Ok(s) => {
                    let instance = OwServerInstance::new( self.client.clone(), s, self.token ) ;
                    thread::spawn( move || {
                        instance.handle_query() ;
                    });
                },
                Err(e)=>{
                    eprintln!("Bad server query {}",e);
                },
            }
        }
        Ok(())
    }
}

struct OwServerInstance {
    client: crate::OwClient,
    stream: TcpStream,
    token: [u8;16],
}
impl OwServerInstance {
    fn new(client: crate::OwClient, stream: TcpStream, token: [u8;16]) -> OwServerInstance {
        OwServerInstance {
			client,
            stream,
            token,
        }
    }
    fn handle_query( &self ) {
        // Set timeout
        match self.stream.set_read_timeout( Some(Duration::from_secs(5))) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Cannot set timeout to server query {}",e);
                return ;
            },
        }
        
        // get header
		static HSIZE: usize = 24 ;
        let mut buffer: [u8; HSIZE ] = [ 0 ; HSIZE ];

    }
}
