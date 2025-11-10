// owrust project
// https://github.com/alfille/owrust
//
// This is a rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

// main.rs is a skeleton structure to talk to the library

use owrust ;
use owrust::parse_args ;

fn main() {
	let mut owserver = owrust::new() ;

	let _paths = parse_args::command_line( &mut owserver ) ;

    println!("owrust skeleton");
}
