use owrust ;
use owrust::parse_args ;

fn main() {
	let mut owserver = owrust::new() ;

	match parse_args::command_line( &mut owserver ) {
		Ok( paths ) => {
			for path in paths.iter() {
				match owserver.dir(&path) {
					Ok(files) => {
						println!("{}",files) ;
					}
					Err(e) => {
						eprintln!("Trouble with path {}",path);
					}
				}
			}
		}
		Err(_e) => {
			eprintln!("owdir trouble");
		},
	}
}
