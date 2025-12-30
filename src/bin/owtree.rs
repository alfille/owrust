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
//! ## PURPOSE
//! Show the 1-wire directory structure and devices. Similar to the unix `tree` program.
//!
//! ## SYNTAX
//! ```
//! owtree [OPTIONS] PATH
//! ```
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
//! * `owtree` is a command line program
//! * output to stdout
//! * errors to stderr
//! * can be "piped" to uther programs like `less` and `grep`
//!
//! ## EXAMPLE
//! ### Read root 1-wire directory
//! ```
//! owtree -s localhost:4304 | head -30
//! ```
//! ```text
//! /
//! ├── 10.67C6697351FF
//! │   ├── address
//! │   ├── alias
//! │   ├── crc8
//! │   ├── errata
//! │   │   ├── die
//! │   │   ├── trim
//! │   │   ├── trimblanket
//! │   │   └── trimvalid
//! │   ├── family
//! │   ├── id
//! │   ├── latesttemp
//! │   ├── locator
//! │   ├── power
//! │   ├── r_address
//! │   ├── r_id
//! │   ├── r_locator
//! │   ├── scratchpad
//! │   ├── temperature
//! │   ├── temphigh
//! │   ├── templow
//! │   └── type
//! ├── 05.4AEC29CDBAAB
//! │   ├── PIO
//! │   ├── address
//! │   ├── alias
//! │   ├── crc8
//! │   ├── family
//! │   ├── id
//! ...
//! ```
//! There is a lot of virtual information included
//! * Everything is mirrored in the bus.x directories
//! * an a mirror in uncached
//! * Total line count `owtree | wc -l` = 6582
//!
//! ### Read __bare__ root 1-wire directory
//! ```
//! owtree -s localhost:4304 --bare | head -30
//! ```
//! ```text
//! /
//! ├── 10.67C6697351FF
//! │   ├── address
//! │   ├── alias
//! │   ├── crc8
//! │   ├── errata
//! │   │   ├── die
//! │   │   ├── trim
//! │   │   ├── trimblanket
//! │   │   └── trimvalid
//! │   ├── family
//! │   ├── id
//! │   ├── latesttemp
//! │   ├── locator
//! │   ├── power
//! │   ├── r_address
//! │   ├── r_id
//! │   ├── r_locator
//! │   ├── scratchpad
//! │   ├── temperature
//! │   ├── temphigh
//! │   ├── templow
//! │   └── type
//! ├── 05.4AEC29CDBAAB
//! │   ├── PIO
//! │   ├── address
//! │   ├── alias
//! │   ├── crc8
//! │   ├── family
//! │   ├── id
//! ...
//! ```
//! * No virtual information included (not apparent in the snippet above)
//! * Total line count `owtree --bare | wc -l` = 36
//!
//! ### Read __pruned__ root 1-wire directory
//! ```
//! owtree -s localhost:4304 --prune | head -30
//! ```
//! ```text
//! /
//! ├── 10.67C6697351FF
//! │   ├── alias
//! │   ├── errata
//! │   │   ├── die
//! │   │   ├── trim
//! │   │   ├── trimblanket
//! │   │   └── trimvalid
//! │   ├── latesttemp
//! │   ├── power
//! │   ├── scratchpad
//! │   ├── temperature
//! │   ├── temphigh
//! │   └── templow
//! └── 05.4AEC29CDBAAB
//!     ├── PIO
//!     ├── alias
//!     └── sensed
//! ```
//! * `--prune` also triggers `--bare` automatically
//! * No virtual information included
//! * Convenience files (e.g. id) are suppressed
//! * Total line count `owtree --bare | wc -l` = 18
//! ### {c} 2025 Paul H Alfille -- MIT Licence

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"

use owrust::console::console_line;
use owrust::parse_args::{OwTree, Parser};

fn main() {
    let mut owserver = owrust::new(); // create structure for owserver communication
    let prog = OwTree;

    // configure and get paths
    match prog.command_line(&mut owserver) {
        Ok(paths) => {
            if paths.is_empty() {
                // No path -- assume root
                from_path(&mut owserver, "/".to_string());
            } else {
                // show tree for each path
                for path in paths.into_iter() {
                    from_path(&mut owserver, path);
                }
            }
        }
        Err(e) => {
            eprintln!("owtree trouble {}", e);
        }
    }
}

// start at path, printing and following directories recursively
fn from_path(owserver: &mut owrust::OwMessage, path: String) {
    let root = File::root(path);
    root.root_print(owserver);
}

#[derive(Debug, Clone)]
// Structure for a directory
struct Dir {
    contents: Vec<File>,
}
impl Dir {
    // directory needs to call dirall to get a list of contents
    fn new(owserver: &mut owrust::OwMessage, path: String) -> Self {
        match owserver.dirallslash(&path) {
            Ok(d) => Dir {
                contents: d.into_iter().map(File::new).collect(),
            },
            Err(e) => {
                eprintln!("Trouble reading directory {}: {} ", &path, e);
                Dir::null_dir()
            }
        }
    }
    fn null_dir() -> Self {
        Dir { contents: vec![] }
    }
    // print each file in directory
    fn print(&self, owserver: &mut owrust::OwMessage, prefix: &String) {
        let len = self.contents.len();
        for (i, f) in self.contents.iter().enumerate() {
            f.print(owserver, prefix, i == len - 1);
        }
    }
}

#[derive(Debug, Clone)]
// file structure for each entry
struct File {
    path: String, // full path
    name: String, // filename itself (for display)
    dir: bool,    // is this a directory?
}
impl File {
    // parse file
    fn new(path: String) -> Self {
        let parts: Vec<String> = path.split('/').map(String::from).collect();
        let len = parts.len();
        if len == 0 {
            File {
                path,
                name: "No name".to_string(),
                dir: false,
            }
        } else if len == 1 {
            File {
                path,
                name: parts[0].clone(),
                dir: false,
            }
        } else if parts[len - 1].is_empty() {
            // directory since null last element
            File {
                path,
                name: parts[len - 2].clone(),
                dir: true,
            }
        } else {
            // regular file
            File {
                path,
                name: parts[len - 1].clone(),
                dir: false,
            }
        }
    }
    fn root(path: String) -> Self {
        File {
            path: path.clone(),
            name: path.clone(),
            dir: true,
        }
    }
    fn root_print(&self, owserver: &mut owrust::OwMessage) {
        // File
        console_line(&self.name);
        let dir = Dir::new(owserver, self.path.clone());
        dir.print(owserver, &"".to_string());
    }
    // print each file with appropriate structure "prefix"
    fn print(&self, owserver: &mut owrust::OwMessage, prefix: &String, last: bool) {
        // File name printed
        if last {
            console_line(format!("{}{}{}", prefix, END, self.name));
        } else {
            console_line(format!("{}{}{}", prefix, NEXT, self.name));
        }
        // Dir followed
        if self.dir {
            let prefix: String = match last {
                true => format!("{}{}", prefix, TAB),
                false => format!("{}{}", prefix, RGT),
            };
            let dir = Dir::new(owserver, self.path.clone());
            dir.print(owserver, &prefix);
        }
    }
}

const END: &str = "└── ";
const RGT: &str = "│   ";
const NEXT: &str = "├── ";
const TAB: &str = "    ";
