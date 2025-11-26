//! **owtree** -- _Rust version_
//!
//! ## show the directory structure for owserver
//! 
//! **owtree** a tool in the 1-wire file system **OWFS**
//!
//! This Rust version of **owtree** is part of **owrust** -- the _Rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## SYNTAX
//! ```
//! owtee [OPTIONS] PATH
//! ```
//!
//! ## OPTIONS
//! * `-s IP:port` (default `localhost:4304`)
//! * `--dir`      Add trailing **/** for directory elements
//! * `--bare`     Suppress non-device entries
//! *                and non-unique device entries 
//! * -h           for full list of options
//!
//! ## PATH
//! * 1-wire path
//! * default is root **/**
//! * more than one path can be given
//!
//! ## USAGE
//! * owserver must be running in a network-accessible location
//! * `owtree` is a command line program
//! * output to stdout
//! * errors to stderr
//! 
//! ## EXAMPLE
//! Read root 1-wire directory
//! ```
//! owdir -s localhost:4304 /
//! ```
//! ```text
//! /10.67C6697351FF,/05.4AEC29CDBAAB,/bus.0,/uncached,/settings,/system,/statistics,/structure,/simultaneous,/alarm
//! ```
//! Read the root directory, dont'show non-devices and split entries to separate lines
//! ```
//! owdir -s localhost:4304 --bare / | tr ',' '\n'
//! ```
//! ```text
//! /10.67C6697351FF
//! /05.4AEC29CDBAAB
//! ```
//! Read a device directory and split entries to separate lines
//! ```
//! owdir -s localhost:4304 /10.67C6697351FF | tr ',' '\n'
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
//! {c} 2025 Paul H Alfille -- MIT Licence

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
                from_path( &owserver, "/".to_string() ) ;
            } else {
                // for each path in command line
                for path in paths.into_iter() {
                    from_path( &owserver, path ) ;
                }
            }
        },
        Err(e) => {
            eprintln!("owdir trouble {}",e);
        }
    }
}

// Split path into parts
// prune final "/"
// prune initial "/"
fn parse_path( fullpath: String ) -> Vec<String> {
	fullpath.trim_matches('/')
		.split('/')
		.map( |s| s.to_string() )
		.collect()
}

fn parse_diff( first: Vec<String>, second: Vec<String> ) -> usize {
	let len = std::cmp::min( first.len(), second.len() ) ;
	for i in (0..len) {
		if first[i] != second[i] {
			return i;
		}
	}
	len
}


// print 1-wire directory contents
fn from_path( owserver: &owrust::OwClient, path: String ) {
    match owserver.dirall(&path) {
        Ok(files) => {
            match owserver.show_text(files) {
                Ok(t) => {
                    println!("{}",t);
                },
                Err(e) => {
                    eprintln!("Trouble displaying directory {}",e);
                }
            } ;
        },
        Err(e) => {
            eprintln!("Trouble with path {} Error {}",path,e);
        }
    }
}   

#[cfg(test)]
mod tests {
    use super::*;
    
    fn s2v( v: Vec<&str> ) -> Vec<String> {
		v
		.iter()
		.map( |s| s.to_string() )
		.collect()
	}

    #[test]
    fn p_path1() {
        let path = "/".to_string();
        let v = parse_path( path ) ;
        let q = s2v(vec![]);
        assert!( q.is_empty()) ;
    }
    #[test]
    fn p_path2() {
        let path = "/10.1232/temperature" .to_string() ;
        let v = parse_path( path ) ;
        let q = s2v(vec!["10.1232","temperature"]);
        assert_eq!( v,q) ;
    }
    #[test]
    fn p_path3() {
        let path = "/10.1232/temperature/" .to_string() ;
        let v = parse_path( path ) ;
        let q = s2v(vec!["10.1232","temperature"]);
        assert_eq!( v,q) ;
    }
    #[test]
    fn p_diff1() {
        let f = "/10.1232/temperature/" .to_string() ;
        let s = "/10.1232/temperture/" .to_string() ;
        let pf = parse_path(f) ;
        let ps = parse_path(s) ;
        assert_eq!( parse_diff(pf,ps), 1 ) ;
    }
    #[test]
    fn p_diff2() {
        let f = "/10.1232/temperature/" .to_string() ;
        let s = "/10.1232/temperature/wertyw" .to_string() ;
        let pf = parse_path(f) ;
        let ps = parse_path(s) ;
        assert_eq!( parse_diff(pf,ps), 2 ) ;
    }
    #[test]
    fn p_diff3() {
        let f = "/10.1232/temperature/" .to_string() ;
        let s = "/10.1232/temperature" .to_string() ;
        let pf = parse_path(f) ;
        let ps = parse_path(s) ;
        assert_eq!( parse_diff(pf,ps), 2 ) ;
    }
}
