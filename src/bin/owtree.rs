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
    let paths = match parse_args::command_line( &mut owserver ) {
        Ok( paths ) => {
            if paths.is_empty() {
                vec!("/".to_string())
            } else {
                paths
            }
        },
        Err(_e) => vec!("/".to_string()),
    } ;
    
    // add slash and persistence
    match parse_args::temporary_client( &owserver, vec!("--dir","--persist")) {
        Ok( new_server ) => {
            for path in paths.into_iter() {
                from_path( &new_server, path ) ;
            }
        },
        Err(_e) => {
            eprintln!("Could not set persistence and directory signal");
        },
    } ;
}

fn from_path ( owserver: &owrust::OwClient, path: String ) {
    let root = File::root(path) ;
    root.print( owserver, &"".to_string(), true ) ;
}

#[derive(Debug,Clone)]
struct Dir {
    contents: Vec<File>,
}
impl Dir {
    fn new( owserver: &owrust::OwClient, path: String ) -> Self {
        let dir_u8 = match owserver.dirall( &path ) {
            Ok(x) => x,
            _ => {
                eprintln!("Trouble getting directory of {}",&path ) ;
                return Dir::null_dir() ;
            },
        };
        let dirlist = match String::from_utf8(dir_u8) {
            Ok(d) => d,
            _ => {
                eprintln!("Bad characters in directory of {}",&path ) ;
                return Dir::null_dir() ;
            },
        };
        // directory
        Dir {
            contents: dirlist
                .split(',')
                .map( |f| File::new( f.to_string() ) )
                .collect(),
        }
    }
    fn null_dir() -> Self {
        Dir {
            contents: vec!(),
        }
    }
    fn print( &self, owserver: &owrust::OwClient, prefix: &String ) {
        let len = self.contents.len() ;
        let prefix_mid = format!("{}{}",prefix,RGT);
        let prefix_end = format!("{}{}",prefix,TAB);
        for (i,f) in self.contents.iter().enumerate() {
            if i < len-1 {
                f.print( owserver, &prefix_mid, false )
            } else {
                f.print( owserver, &prefix_end, true )
            }
        }
    }
} 

#[derive(Debug,Clone)]
struct File {
    path: String,
    name: String,
    dir: bool,
}
impl File {
    fn new( path: String ) -> Self {
        let parts: Vec<String> = path
            .split('/')
            .map( String::from )
            .collect() ;
        let len = parts.len() ;
        if len == 0 {
            File {
                path,
                name: "No name".to_string(),
                dir: false
            }
        } else if len==1 {
            File {
                path,
                name: parts[0].clone(),
                dir: false
            }
        } else if parts[len-1].is_empty() {
            // directory since null last element
            File {
                path,
                name: parts[len-2].clone(),
                dir: true,
            }
        }
        else {
            // regular file
            File {
                path,
                name: parts[len-1].clone(),
                dir: false,
            }
        }
    }
    fn root ( path: String ) -> Self {
        File {
            path: path.clone(),
            name: path.clone(),
            dir: true,
        }
    }
    fn print( &self, owserver: &owrust::OwClient, prefix: &String, last: bool ) {
        // File
        if last {
            println!("{}{}{}",prefix,END,self.name);
        } else {
            println!("{}{}{}",prefix,NEXT,self.name);
        }
        if self.dir {
            let prefix: String = match last {
                true => format!("{}{}",prefix,TAB),
                false => format!("{}{}",prefix,RGT),
            } ;
            let dir = Dir::new( owserver, self.path.clone() ) ;
            dir.print(owserver, &prefix) ;
        }
    }
}

const END:  &str = "└── ";
const RGT:  &str = "│   ";
const NEXT: &str = "├── ";
const TAB:  &str = "    ";
