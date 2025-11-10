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
	match owserver.dir(&path) {
		Ok(files) => {
			println!("{}",owserver.printable(files)) ;
		}
		Err(_e) => {
			eprintln!("Trouble with path {}",path);
		}
	}
}	
