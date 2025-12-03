//! **owdir** -- _Rust version_
//!
//! ## Read a directory from owserver
//! 
//! **owdir** a tool in the 1-wire file system **OWFS**
//!
//! This Rust version of **owdir** is part of **owrust** -- the _Rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## SYNTAX
//! ```
//! owdir [OPTIONS] PATH
//! ```
//! ## PURPOSE
//! __owdir__ shows a 1-wire "directory" via owserver
//! * Root (/) directories show devices and informational entries (like `statistics`)
//! * Device (/10.132542312) directories show device entries. Information and properties (like `temperature`) 
//! * All entries are shown from the root of owserver. There is no "current directory"
//!
//! ## OPTIONS
//! * `-s IP:port` (default `localhost:4304`)
//! * `--dir`      Add trailing **/** for directory elements
//! * `--bare`     Suppress non-device entries
//! * `--prune`    Even more spare output suppressing convenience files like `id` and `crc` 
//! * -h           for full list of options
//!
//! ## PATH
//! * 1-wire path
//! * default is root **/**
//! * more than one path can be given
//!
//! ## USAGE
//! * owserver must be running in a network-accessible location
//! * `owdir` is a command line program
//! * output to stdout
//! * errors to stderr
//! 
//! ## EXAMPLE
//! Read root 1-wire directory
//! ```
//! owdir -s localhost:4304 /
//! ```
//! ```text
//! /10.67C6697351FF
//! /05.4AEC29CDBAAB
//! /bus.0
//! /uncached
//! /settings
//! /system
//! /statistics
//! /structure
//! /simultaneous
//! /alarm
//! ```
//! Read the root directory, but dont'show non-devices
//! ```
//! owdir -s localhost:4304 --bare /
//! ```
//! ```text
//! /10.67C6697351FF
//! /05.4AEC29CDBAAB
//! ```
//! Read a device directory
//! ```
//! owdir -s localhost:4304 /10.67C6697351FF
//! ```
//! ```text
//! /10.67C6697351FF/address
//! /10.67C6697351FF/alias
//! /10.67C6697351FF/crc8
//! /10.67C6697351FF/errata
//! /10.67C6697351FF/family
//! /10.67C6697351FF/id
//! /10.67C6697351FF/latesttemp
//! /10.67C6697351FF/locator
//! /10.67C6697351FF/power
//! /10.67C6697351FF/r_address
//! /10.67C6697351FF/r_id
//! /10.67C6697351FF/r_locator
//! /10.67C6697351FF/scratchpad
//! /10.67C6697351FF/temperature
//! /10.67C6697351FF/temphigh
//! /10.67C6697351FF/templow
//! /10.67C6697351FF/type
//! ``` 
//! Read a device directory "pruning out" the convenience entries
//! ```
//! owdir -s localhost:4304 --prune /10.67C6697351FF
//! ```
//! ```text
//! /10.67C6697351FF/alias
//! /10.67C6697351FF/errata
//! /10.67C6697351FF/latesttemp
//! /10.67C6697351FF/power
//! /10.67C6697351FF/scratchpad
//! /10.67C6697351FF/temperature
//! /10.67C6697351FF/temphigh
//! /10.67C6697351FF/templow
//! ``` 
//! ### {c} 2025 Paul H Alfille -- MIT Licence

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"

use owrust::parse_args ;

fn main() {
    let mut owserver = owrust::new() ; // create structure for owserver communication

    // configure and get paths
    match parse_args::command_line( &mut owserver ) {
        Ok( paths ) => {
            if paths.is_empty() {
                // No path -- assume root
                from_path( &mut owserver, "/".to_string() ) ;
            } else {
                // for each path in command line
                for path in paths.into_iter() {
                    from_path( &mut owserver, path ) ;
                }
            }
        },
        Err(e) => {
            eprintln!("owdir trouble {}",e);
        }
    }
}

// print 1-wire directory contents
fn from_path( owserver: &mut owrust::OwClient, path: String ) {
    match owserver.dirall(&path) {
        Ok(files) => println!("{}",files.join("\n")),
        Err(e) => eprintln!("Trouble with path {} Error {}",path,e),
    }
}   
