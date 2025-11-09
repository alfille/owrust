use owrust ;

use pico_args::Arguments;
use std::path::PathBuf;

const HELP: &str = "\
owdir [OPTIONS] <PATH>
Read a 1-wire directory from owserver.

OPTIONS:
-s --server IPaddress:port (default localhost:3504)

Temperature scale
  -C, --Celsius
  -F, --Farenheit
  -K, --Kelvin
  -R, --Rankine
  
Device serial number format
f(amily) i(d) c(hecksum
  -d, --device fi | f.i | fic | f.ic | fi.c | f.i.c
";

fn command_line( owserver: &mut owrust::OwClient ) -> Result<Vec<String>,pico_args::Error> {
	// parse the command line
	let mut args = Arguments::from_env();

    // Handle the help flag first
    if args.contains(["-h", "--help"]) {
        println!("{}", HELP);
        return Ok(Vec::new()) ;
    }
	// Temperature
	if args.contains(["-C","--Celsius"]) {
		owserver.temperature( owrust::Temperature::CELSIUS ) ;
	}
	if args.contains(["-F","--Farenheit"]) {
		owserver.temperature( owrust::Temperature::FARENHEIT ) ;
	}
	if args.contains(["-K","--Kelvin"]) {
		owserver.temperature( owrust::Temperature::KELVIN ) ;
	}
	if args.contains(["-R","--Rankine"]) {
		owserver.temperature( owrust::Temperature::RANKINE ) ;
	}

	// Pressure
	if args.contains("--mmhg") {
		owserver.pressure( owrust::Pressure::MMHG ) ;
	}
	if args.contains("--inhg") {
		owserver.pressure( owrust::Pressure::INHG ) ;
	}
	if args.contains("--mbar") {
		owserver.pressure( owrust::Pressure::MBAR ) ;
	}
	if args.contains("--atm") {
		owserver.pressure( owrust::Pressure::ATM ) ;
	}
	if args.contains("--pa") {
		owserver.pressure( owrust::Pressure::PA ) ;
	}
	if args.contains("--psi") {
		owserver.pressure( owrust::Pressure::PSI ) ;
	}

	// Device
	let d = args.opt_value_from_fn(["-d","--device"],parse_device) ? ;
	owserver.device( d.unwrap_or(owrust::Device::DEFAULT) ) ;
	
	// Server
	let s: Option<String> = args.opt_value_from_str(["-s","--server"]) ? ;
	owserver.server(s.unwrap()) ;

	let mut result: Vec<String> = Vec::new() ;
	for os in args.finish() {
		match os.into_string() {
			Ok(s) => result.push(s),
			Err(_) => eprintln!("Bad command line entry."),
		}
	}
	Ok(result)
}

fn parse_device(s: &str) -> Result<owrust::Device, &'static str> {
	match s {
		"fi" => Ok(owrust::Device::FI),
		"f.i" => Ok(owrust::Device::F_I),
		"fic" => Ok(owrust::Device::FIC),
		"f.ic" => Ok(owrust::Device::F_IC),
		"fi.c" => Ok(owrust::Device::FI_C),
		"f.i.c" => Ok(owrust::Device::F_I_C),
		_  => Err("Not a number"),
	}
}
fn main() {
	let mut owserver = owrust::new() ;

	let _ = command_line( &mut owserver ) ;

    println!("owrust skeleton");
}
