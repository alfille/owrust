//! **owread** -- _rust version_
//!
//! ## Read a value from owserver ( from a 1-wire device )
//! 
//! This is a tool in the 1-wire file system **OWFS**
//!
//! This version os **owdir** is part of **owrust** -- the _rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## SYNTAX
//! ```
//! owread [OPTIONS] PATH
//! ```
//!
//! ## OPTIONS
//! * `-s IP:port` (default `localhost:4304`)
//! * `--hex       show the value in hexidecimal
//! * `--size n    return only n bytes
//! * `--offset m  start return at byte m
//! * -h           more help
//!
//! ## PATH
//! a 1-wire path (default `/`)
//!
//!
//! ## USAGE
//! * owserver must be running in a network-accessible location
//! * `owread` is a command line program
//! * output to stdout
//! * errors to stderr
//! 
//! ## EXAMPLE
//! Read a temperature
//! ```
//! owread /10.67C6697351FF/temperature
//!     85.7961//! 
//!```
//! Read temperature in hex
//! ```
//! owread /10.67C6697351FF/temperature --hex
//! 20 20 20 20 20 37 36 2E 31 35 38 35//! ```
//! ```
//! {c} 2025 Paul H Alfille -- MIT Licence

// owrust project
// https://github.com/alfille/owrust
//
// This is a rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

// owdir.rs mimics the owdir shell program
// the path is a 1-wire path and the returned entries are 1-wire devices and virtual directories

use owrust ;
use owrust::parse_args ;

fn main() {
	let mut owserver = owrust::new() ;

	match parse_args::command_line( &mut owserver ) {
		Ok( paths ) => {
			if paths.len() == 0 {
				// No path -- assume root
				from_path( &owserver, "/".to_string() ) ;
			} else {
				// for each path entry
				for path in paths.into_iter() {
					from_path( &owserver, path ) ;
				}
			}
		}
		Err(_e) => {
			eprintln!("owdir trouble");
		},
	}
}

fn from_path( owserver: &owrust::OwClient, path: String ) {
	match owserver.read(&path) {
		Ok(files) => {
			println!("{}",owserver.printable(files)) ;
		}
		Err(_e) => {
			eprintln!("Trouble with path {}",path);
		}
	}
}	
