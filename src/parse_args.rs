// owrust project
// https://github.com/alfille/owrust
//
// This is a rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille


use std::ffi::OsString;
use pico_args::Arguments;
use std::{env,process} ;
use crate::OwError ;

const HELP: &str = "\
.
OPTIONS:
-s --server IPaddress:port (default localhost:4304)

Temperature scale
  -C, --Celsius
  -F, --Farenheit
  -K, --Kelvin
  -R, --Rankine
  
Format serial number
  f(amily) i(d) c(hecksum
  -f, --format fi | f.i | fic | f.ic | fi.c | f.i.c

Display
  --dir    Add a directory separator (/) after directories
  --hex    Display values read in hexidecimal
  --size   Max size (in bytes) of returned field (truncate if needed)
  --offset Position in field to start returned value (for long memory contents)
  
OTHER
  -h, --help  This help message
  -d, --debug Internal process information (more times gives more info)
";

pub fn command_line( owserver: &mut crate::OwClient ) -> Result<Vec<String>,OwError> {
	// normal path -- from environment
	let args = Arguments::from_env();
	match parser( owserver, args ) {
		Ok(v) => return Ok(v),
		Err(e) => {
			eprintln!("Parsing error {:?}",e);
			return Err(OwError::ConfigError);
		},
	};
}

pub fn vector_line( owserver: &mut crate::OwClient, raw_args: Vec<OsString> ) -> Result<Vec<String>,OwError> {
	// normal path -- from envoronment
	let args = Arguments::from_vec(raw_args);
	match parser( owserver, args ) {
		Ok(v) => return Ok(v),
		Err(e) => {
			eprintln!("Parsing error {:?}",e);
			return Err(OwError::ConfigError);
		},
	};
}

fn progname() -> String {
	match env::current_exe() {
        Ok(path) => {
            // Get the full path (e.g., /path/to/my_app)
            // Extract the filename component (e.g., my_app)
            if let Some(name) = path.file_name() {
				return name.to_string_lossy().into_owned() ;
            } else {
				return "<no_name>".to_string() ;
			}
        },
        Err(_e) => {
			return "<error>".to_string() ;
		}
	}
}

fn parser( owserver: &mut crate::OwClient, mut args: Arguments ) -> Result<Vec<String>,pico_args::Error> {

	// Handle the help flag first
	if args.contains(["-h", "--help"]) {
		let p = progname() ;
		let pre_help = match &p[..] {
			"owdir" => format!("\
{} [OPTIONS] <1-wire path>
Read a virtual 1-wire directory using owserver.
			", p),
			"owread" => format!("\
{} [OPTIONS] <1-wire path>
Read a 1-wire device value using owserver.
			", p),
			"owwrite" => format!("\
{} [OPTIONS] <1-wire path> <value>
Write a value to a 1-wire device field using owserver.
			", p),
			"owget" => format!("\
{} [OPTIONS] <1-wire path>
Read a directory or value from 1-wire (depending on the path) using owserver.
			", p),
			&_ => format!("\
{} [OPTIONS] <1-wire path>
Read a virtual 1-wire directory from owserver.
			", p),
		} ;
		println!("{}{}", pre_help,HELP);
		process::exit(0) ;
	}

	// debug
	while args.contains(["-d","--debug"]) {
		owserver.debug += 1 ;
		eprintln!("Debuging level {}",owserver.debug);
	}

	// Temperature
	if args.contains(["-C","--Celsius"]) {
		owserver.set_temperature( crate::Temperature::CELSIUS ) ;
	}
	if args.contains(["-F","--Farenheit"]) {
		owserver.set_temperature( crate::Temperature::FARENHEIT ) ;
	}
	if args.contains(["-K","--Kelvin"]) {
		owserver.set_temperature( crate::Temperature::KELVIN ) ;
	}
	if args.contains(["-R","--Rankine"]) {
		owserver.set_temperature( crate::Temperature::RANKINE ) ;
	}

	// Pressure
	if args.contains("--mmhg") {
		owserver.set_pressure( crate::Pressure::MMHG ) ;
	}
	if args.contains("--inhg") {
		owserver.set_pressure( crate::Pressure::INHG ) ;
	}
	if args.contains("--mbar") {
		owserver.set_pressure( crate::Pressure::MBAR ) ;
	}
	if args.contains("--atm") {
		owserver.set_pressure( crate::Pressure::ATM ) ;
	}
	if args.contains("--pa") {
		owserver.set_pressure( crate::Pressure::PA ) ;
	}
	if args.contains("--psi") {
		owserver.set_pressure( crate::Pressure::PSI ) ;
	}

	// Format
	let d = args.opt_value_from_fn(["-f","--format"],parse_device) ? ;
	owserver.set_format( d.unwrap_or(crate::Format::DEFAULT) ) ;
	
	// Display
	owserver.hex = args.contains("--hex") ;
	owserver.slash = args.contains("--dir") ;
	
	let y = args.opt_value_from_str("--size") ? ;
	if let Some(x) = y {
		owserver.size = x ;
	}
	
	let y = args.opt_value_from_str("--offset") ? ;
	if let Some(x) = y {
		owserver.offset = x ;
	}
	
	// Server
	let s: Option<String> = args.opt_value_from_str(["-s","--server"]) ? ;
	owserver.set_server(s.unwrap_or(String::from("localhost:4304"))) ;

	let mut result: Vec<String> = Vec::new() ;
	for os in args.finish() {
		match os.into_string() {
			Ok(s) => result.push(s),
			Err(_) => eprintln!("Bad command line entry."),
		}
	}
	if owserver.debug > 1 {
		eprintln!("{} path entries",result.len());
	}
	Ok(result)
}

fn parse_device(s: &str) -> Result<crate::Format, &'static str> {
	match s {
		"fi" => Ok(crate::Format::FI),
		"f.i" => Ok(crate::Format::FdI),
		"fic" => Ok(crate::Format::FIC),
		"f.ic" => Ok(crate::Format::FdIC),
		"fi.c" => Ok(crate::Format::FIdC),
		"f.i.c" => Ok(crate::Format::FdIdC),
		_  => Err("Not a number"),
	}
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
				let mut owserver = crate::new() ;
				let _ = vector_line( &mut owserver, args ) ;
				let result = owserver.get_temperature() ;
				assert_eq!(result, test);
			}
		}
	}
}
