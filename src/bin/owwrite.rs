//! **owwrite** -- _Rust version_
//!
//! ## Write a value to owserver ( to a specific 1-wire device file )
//! 
//! **owwrite** is a tool in the 1-wire file system **OWFS**
//!
//! This Rust version of **owwrite** is part of **owrust** -- the _Rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## SYNTAX
//! ```
//! owwrite [OPTIONS] PATH VALUE
//! ```
//!
//! ## OPTIONS
//! * `-s IP:port` (default `localhost:4304`)
//! * `--hex       read the value in hexidecimal
//! * `--size n    write only n bytes
//! * `--offset m  start writing at byte m
//! * -h           for full list of options
//!
//! ## PATH
//! * 1-wire path to a file
//! * No Default
//! 
//! ## VALUE
//! * Text (a byte string)
//! * Hexidecimal bytes ( e.g. `03A3FF` is a 3 byte value )
//!   * upper and lower case a-f allowed
//!   * no 0x prefix should be used
//!   * no spaces between bytes
//!
//! ### More than one PATH / VALUE pair allowed
//!
//! **owwrite** only works on files.
//!
//! ## USAGE
//! * owserver must be running in a network-accessible location
//! * `owwrite` is a command line program
//! * errors to stderr
//! 
//! ## EXAMPLE
//! Read a temperature
//! ```
//! owread /10.67C6697351FF/temperature
//!     85.7961 
//! ```
//! Read temperature in hex
//! ```
//! owread /10.67C6697351FF/temperature --hex
//! 20 20 20 20 20 37 36 2E 31 35 38 35
//! ```
//! {c} 2025 Paul H Alfille -- MIT Licence

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use owrust::parse_args ;

fn main() {
	let mut owserver = owrust::new() ; // create structure for owserver communication

	// configure and get paths
	match parse_args::command_line( &mut owserver ) {
		Ok( paths ) => {
			if paths.is_empty() {
				// No path
				eprintln!( "Not enough arguments" ) ;
			} else if ! paths.len().is_multiple_of(2) {
				eprintln!("Path and value not paired") ;
			} else {
				// for each path/value pair in command line
				for chunk in paths.chunks(2) {
					from_path( &owserver, &chunk[0], &chunk[1] ) ;
				}
			}
		}
		Err(_e) => {
			eprintln!("owread trouble");
		},
	}
}

// print 1-wire file contents (e.g. a sensor reading)
fn from_path( owserver: &owrust::OwClient, path: &String, value: &String ) {
	match owserver.write( path, value.as_bytes() ) {
		Ok(_) => (),
		Err(_) => {
			eprintln!("Trouble with write -- path {} value {}",path, value);
		},
	}
}	
