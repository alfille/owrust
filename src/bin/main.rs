use owrust ;
use owrust::parse_args ;

fn main() {
	let mut owserver = owrust::new() ;

	let _paths = parse_args::command_line( &mut owserver ) ;

    println!("owrust skeleton");
}
