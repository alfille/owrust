//! **owpresent** -- _Rust version_
//!
//! ## Does a file exiss (devise exists) on owserver
//! 
//! **owpresent** is a tool in the 1-wire file system **OWFS**
//!
//! This Rust version of **owpresent** is part of **owrust** -- the _Rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## SYNTAX
//! ```
//! owpresent [OPTIONS] PATH
//! ```
//!
//! ## OPTIONS
//! * `-s IP:port` (default `localhost:4304`)
//! * -h           for full list of options
//!
//! ## PATH
//! * 1-wire path to a file
//! * No Default
//! * More than one path can be given
//!
//! **owpresent** works on files and directories.
//!
//! ## USAGE
//! * owserver must be running in a network-accessible location
//! * `owpresent` is a command line program
//! * output to stdout
//!   * `1` if present
//!   * `0` if not present
//! * errors to stderr
//! 
//! ## EXAMPLE
//! Test presence of a device
//! ```
//! owpresent /10.67C6697351FF
//! 1
//! ```
//! Test a file
//! ```
//! owpresent /10.67C6697351FF/temperature
//! 1
//! ```
//! Test a device that isn't there
//! ```
//! owpresent /10.FFFFFFFFFFFF
//! 0
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
				// No path -- assume root
				from_path( &owserver, "/".to_string() ) ;
			} else {
				// for each pathon command line
				for path in paths.into_iter() {
					from_path( &owserver, path ) ;
				}
			}
		}
		Err(_e) => {
			eprintln!("owread trouble");
		},
	}
}

// print 1-wire file contents (e.g. a sensor reading)
fn from_path( owserver: &owrust::OwClient, path: String ) {
	match owserver.present(&path) {
		Ok(values) => {
			if values {
				println!("1");
			} else {
				println!("0");
			}
		}
		Err(_e) => {
			eprintln!("Trouble with path {}",path);
		}
	}
}	
