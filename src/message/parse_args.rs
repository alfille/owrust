// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::error::{OwEResult, OwError};
use pico_args::Arguments;
use std::ffi::OsString;
use std::{env, process};

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
  --bare   Show only 1-wire devices, not virtual directories
             also suppress convenience properties like `id`, `crc`, etc.
  --prune  Suppresses bus, id, crc,address and location (only for DIR and DIRALL)
  --persist Keep the connection to owserver live rather than reestablishing
             for every query. Improves performance. 

Pressure
    --mbar --mmhg --inhg --atm --pa --psi
  
OTHER
  -h, --help  This help message
  -d, --debug Internal process information (more times gives more info)
";

/// ### command_line
/// * Argument OwMessage structure (mutable)
/// * Uses command line arguments as source
/// * Uses flags to set OwMessage configuration
/// * Computes owserver protocol flag field in OwMessage
/// * Returns all non-flags on command line (paths and values as required)
/// ### Example
/// ```
/// use std::ffi::OsString;
/// use pico_args::Arguments;
/// use owrust::parse_args::command_line ;
/// let mut owserver = owrust::new() ; // new OwMessage structure
/// let paths = command_line( &mut owserver ).expect("Bad configuration");
/// ```
pub fn command_line(owserver: &mut crate::OwMessage) -> OwEResult<Vec<String>> {
    // normal path -- from environment
    let args = Arguments::from_env();
    parser(owserver, args)
}

/// ### vector_line
/// * Argument OwMessage structure (mutable)
/// * Argument `Vec<String>` instead of command line
/// * Uses flags to set OwMessage configuration
/// * Computes owserver protocol flag field in OwMessage
/// * Returns all non-flags on command line (paths and values as required)
/// ### Example
/// ```
/// use std::ffi::OsString;
/// use pico_args::Arguments;
/// use owrust::parse_args::vector_line ;
/// let mut owserver = owrust::new() ; // new OwMessage structure
/// let args: Vec<&str> = vec!(
///     "-C",
///     "--bare",
///     "/bus.0"
///     );
/// let paths = vector_line( &mut owserver, args ).expect("Bad configuration");
/// ```
pub fn vector_line(owserver: &mut crate::OwMessage, args: Vec<&str>) -> OwEResult<Vec<String>> {
    // normal path -- from environment
    // convert Vec<String> to Vec<OsString>
    let os_args: Vec<OsString> = args.iter().map(OsString::from).collect();
    parser(owserver, Arguments::from_vec(os_args))
}

/// ### modified_messager
/// returns a clone of OwMessage with `args` added
///
/// Useful for temporarily amending a connection using different flags
pub fn modified_messager(
    owserver: &crate::OwMessage,
    args: Vec<&str>,
) -> OwEResult<crate::OwMessage> {
    let mut clone = owserver.clone();
    vector_line(&mut clone, args)?;
    Ok(clone)
}

fn progname() -> String {
    match env::current_exe() {
        Ok(path) => {
            // Get the full path (e.g., /path/to/my_app)
            // Extract the filename component (e.g., my_app)
            if let Some(name) = path.file_name() {
                name.to_string_lossy().into_owned()
            } else {
                "<no_name>".to_string()
            }
        }
        Err(_) => "BadEnv".to_string(),
    }
}

fn parser(owserver: &mut crate::OwMessage, mut args: Arguments) -> OwEResult<Vec<String>> {
    // Handle the help flag first
    if args.contains(["-h", "--help"]) {
        let p = progname();
        let pre_help = match &p[..] {
            "owdir" => format!(
                "\
{} [OPTIONS] <1-wire path>
Read a virtual 1-wire directory using owserver.
            ",
                p
            ),
            "owread" => format!(
                "\
{} [OPTIONS] <1-wire path>
Read a 1-wire device value using owserver.
            ",
                p
            ),
            "owwrite" => format!(
                "\
{} [OPTIONS] <1-wire path> <value>
Write a value to a 1-wire device field using owserver.
            ",
                p
            ),
            "owget" => format!(
                "\
{} [OPTIONS] <1-wire path>
Read a directory or value from 1-wire (depending on the path) using owserver.
            ",
                p
            ),
            "owsnoop" => format!(
                "\
{} [OPTIONS] -p | --port address:port (e.g. localhost:14304)
Inspect owserver messages as they pass through. Serves as an owserver intermediary.
            ",
                p
            ),
            &_ => format!(
                "\
{} [OPTIONS] <1-wire path>
Read a virtual 1-wire directory from owserver.
            ",
                p
            ),
        };
        println!("{}{}", pre_help, HELP);
        process::exit(0);
    }

    // debug
    while args.contains(["-d", "--debug"]) {
        owserver.debug += 1;
        eprintln!("Debuging level {}", owserver.debug);
    }

    // Temperature
    if args.contains(["-C", "--Celsius"]) {
        owserver.temperature = super::Temperature::CELSIUS;
    }
    if args.contains(["-F", "--Farenheit"]) {
        owserver.temperature = super::Temperature::FARENHEIT;
    }
    if args.contains(["-K", "--Kelvin"]) {
        owserver.temperature = super::Temperature::KELVIN;
    }
    if args.contains(["-R", "--Rankine"]) {
        owserver.temperature = super::Temperature::RANKINE;
    }

    // Pressure
    if args.contains("--mmhg") {
        owserver.pressure = super::Pressure::MMHG;
    }
    if args.contains("--inhg") {
        owserver.pressure = super::Pressure::INHG;
    }
    if args.contains("--mbar") {
        owserver.pressure = super::Pressure::MBAR;
    }
    if args.contains("--atm") {
        owserver.pressure = super::Pressure::ATM;
    }
    if args.contains("--pa") {
        owserver.pressure = super::Pressure::PA;
    }
    if args.contains("--psi") {
        owserver.pressure = super::Pressure::PSI;
    }

    // Format
    let d = args.opt_value_from_fn(["-f", "--format"], parse_device)?;
    owserver.format = d.unwrap_or(super::Format::DEFAULT);

    // Persist
    if args.contains("--persist") {
        owserver.stream.set_persistence(true);
    }

    // Display
    if args.contains("--hex") {
        owserver.hex = true;
    }
    if args.contains("--dir") {
        owserver.slash = true;
    }
    if args.contains("--bare") {
        owserver.bare = true;
    }
    if args.contains("--prune") {
        owserver.bare = true;
        owserver.prune = true;
    }
    let y = args.opt_value_from_str("--size")?;
    if let Some(x) = y {
        owserver.size = x;
    }

    let y = args.opt_value_from_str("--offset")?;
    if let Some(x) = y {
        owserver.offset = x;
    }

    // Server
    let serv: Option<String> = args.opt_value_from_str(["-s", "--server"])?;
    if let Some(s) = serv {
        owserver.stream.set_target(&s);
    }

    // Listener
    let listener: Option<String> = args.opt_value_from_str(["-p", "--port"])?;
    if listener.is_some() {
        owserver.listener = listener;
    }

    let mut result: Vec<String> = Vec::new();
    for os in args.finish() {
        match os.into_string() {
            Ok(s) => result.push(s),
            Err(_e) => {
                return Err(OwError::Input("Bad command line entry.".into()));
            }
        }
    }
    if owserver.debug > 1 {
        eprintln!("{} path entries", result.len());
    }

    owserver.make_flags();
    Ok(result)
}

fn parse_device(s: &str) -> OwEResult<super::Format> {
    match s {
        "fi" => Ok(super::Format::FI),
        "f.i" => Ok(super::Format::FdI),
        "fic" => Ok(super::Format::FIC),
        "f.ic" => Ok(super::Format::FdIC),
        "fi.c" => Ok(super::Format::FIdC),
        "f.i.c" => Ok(super::Format::FdIdC),
        _ => Err(OwError::Input(format!("Invalid format {}", s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn short(opt: &String) -> String {
        let c = opt.chars().next().unwrap_or('X');
        format!("-{}", c)
    }

    fn long(opt: &String) -> String {
        format!("--{}", opt)
    }

    #[test]
    fn test_short() {
        let r = short(&"Xxx".to_string());
        assert_eq!(r, "-X");
    }
    #[test]
    fn test_long() {
        let r = long(&"Xxx".to_string());
        assert_eq!(r, "--Xxx");
    }

    #[test]
    fn test_short_long() {
        for ts in [
            ("Celsius", crate::OwMessage::TEMPERATURE_C),
            ("Kelvin", crate::OwMessage::TEMPERATURE_K),
            ("Farenheit", crate::OwMessage::TEMPERATURE_F),
            ("Rankine", crate::OwMessage::TEMPERATURE_R),
        ] {
            let test = ts.0.to_string();
            for t in [short(&test), long(&test)] {
                let args: Vec<&str> = vec![&t];
                let mut owserver = crate::new();
                let _ = vector_line(&mut owserver, args);
                owserver.make_flags();
                let result = owserver.flags & ts.1;
                assert_eq!(result, ts.1);
            }
        }
    }
    #[test]
    fn long_opt() {
        for ts in [
            ("mbar", crate::OwMessage::PRESSURE_MBAR),
            ("mmhg", crate::OwMessage::PRESSURE_MMHG),
            ("inhg", crate::OwMessage::PRESSURE_INHG),
            ("atm", crate::OwMessage::PRESSURE_ATM),
            ("pa", crate::OwMessage::PRESSURE_PA),
            ("psi", crate::OwMessage::PRESSURE_PSI),
            ("persist", crate::OwMessage::PERSISTENCE),
        ] {
            let test = ts.0.to_string();
            for t in [long(&test)] {
                let args: Vec<&str> = vec![&t];
                let mut owserver = crate::new();
                let _ = vector_line(&mut owserver, args);
                owserver.make_flags();
                let result = owserver.flags & ts.1;
                assert_eq!(result, ts.1);
            }
        }
    }
    #[test]
    fn clone_temperature() {
        for ts in [
            ("Celsius", crate::OwMessage::TEMPERATURE_C),
            ("Kelvin", crate::OwMessage::TEMPERATURE_K),
            ("Farenheit", crate::OwMessage::TEMPERATURE_F),
            ("Rankine", crate::OwMessage::TEMPERATURE_R),
        ] {
            let test = ts.0.to_string();
            for t in [short(&test), long(&test)] {
                let args: Vec<&str> = vec![&t];
                let owserver = crate::new();
                let mut owserver2 = modified_messager(&owserver, args).unwrap();
                owserver2.make_flags();
                let result = owserver2.flags & ts.1;
                assert_eq!(result, ts.1);
            }
        }
    }
    #[test]
    fn noport_test() {
        let args: Vec<&str> = vec![];
        let owserver = crate::new();
        let mut owserver2 = modified_messager(&owserver, args).unwrap();
        assert_eq!(owserver2.listener, None);
    }
    #[test]
    fn port_test() {
        let args: Vec<&str> = vec!["-p", "localhost:14304"];
        let owserver = crate::new();
        let mut owserver2 = modified_messager(&owserver, args).unwrap();
        assert_eq!(owserver2.listener, Some("localhost:14304".to_string()));
    }
    #[test]
    fn port2_test() {
        let args: Vec<&str> = vec!["--port", "localhost:14304"];
        let owserver = crate::new();
        let mut owserver2 = modified_messager(&owserver, args).unwrap();
        assert_eq!(owserver2.listener, Some("localhost:14304".to_string()));
    }
}
