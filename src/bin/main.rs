use owrust ;
use owrust::parse_args ;

fn main() {
	let mut owserver = owrust::new() ;

	let paths = parse_args::command_line( &mut owserver ) ;

    println!("owrust skeleton");
}

#[cfg(test)]
mod tests {
    use super::*;
	use std::ffi::OsString;
	
	fn short( opt: &String ) -> String {
		let c = opt.chars().next().unwrap_or('X') ;
		format!("-{}",c)
	}

	fn long( opt: &String ) -> String {
		format!("--{}",opt)
	}

    #[test]
    fn test_short() {
		let r = short(&"Xxx".to_string()) ;
		assert_eq!(r,"-X");
	}
    #[test]
    fn test_long() {
		let r = long(&"Xxx".to_string()) ;
		assert_eq!(r,"--Xxx");
	}
	
    #[test]
    fn test_temperature() {
		for ts in ["Celsius","Kelvin","Farenheit","Rankine"] {
			let test = ts.to_string() ;        
			for t in [short(&test), long(&test)] {
				let args: Vec<OsString> = vec![ OsString::from(&t)];
				let mut owserver = owrust::new() ;
				let _ = parse_args::vector_line( &mut owserver, args ) ;
				let result = owserver.get_temperature() ;
				assert_eq!(result, test);
			}
		}
	}
}

