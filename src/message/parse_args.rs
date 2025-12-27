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
    let mut args = Arguments::from_env();
    parser(owserver, &mut args)
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
    parser(owserver, &mut Arguments::from_vec(os_args))
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

fn progname() -> Option<String> {
    match env::current_exe() {
        Ok(path) =>
        // Get the full path (e.g., /path/to/my_app)
        // Extract the filename component (e.g., my_app)
        {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        }
        _ => None,
    }
}

fn arg_dir(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    let _ = helper(
        args,
        &[
            "owdir [OPTIONS] [PATH]",
            "\tList a 1-wire directory using owserver",
            "\tMore than one PATH can be given",
            "",
            "OPTIONS",
        ],
    );
    parser_server(owserver, args)?;
    parser_directory(owserver, args)?;
    parser_format(owserver, args)?;
    parser_persist(owserver, args)?;
    Ok(())
}

fn arg_read(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    let _ = helper(
        args,
        &[
            "owread [OPTIONS] [PATH]",
            "\tRead a 1-wire file using owserver",
            "\ttypically a sensor or memory reading",
            "\tMore than one PATH can be given",
            "",
            "OPTIONS",
        ],
    );
    parser_server(owserver, args)?;
    parser_temperature(owserver, args)?;
    parser_pressure(owserver, args)?;
    parser_data(owserver, args)?;
    parser_persist(owserver, args)?;
    Ok(())
}

fn arg_write(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    let _ = helper(
        args,
        &[
            "owread [OPTIONS] [PATH] [Value]...",
            "\tWrite to a 1-wire file using owserver",
            "\tto set device memory or configuration",
            "\tMore than one PATH VALUE pair can be given",
            "",
            "OPTIONS",
        ],
    );
    parser_server(owserver, args)?;
    parser_temperature(owserver, args)?;
    parser_pressure(owserver, args)?;
    parser_data(owserver, args)?;
    parser_persist(owserver, args)?;
    Ok(())
}

fn arg_get(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    let _ = helper(
        args,
        &[
            "owget [OPTIONS] [PATH]",
            "\tList a 1-wire directory or read a value using owserver",
            "\tcombined function of owread and owdir depending on PATH",
            "\tMore than one PATH can be given",
            "",
            "OPTIONS",
        ],
    );
    parser_server(owserver, args)?;
    parser_directory(owserver, args)?;
    parser_format(owserver, args)?;
    parser_temperature(owserver, args)?;
    parser_pressure(owserver, args)?;
    parser_data(owserver, args)?;
    parser_persist(owserver, args)?;
    Ok(())
}

fn arg_present(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    let _ = helper(
        args,
        &[
            "owpresent [OPTIONS] [PATH]",
            "\tIs the 1-wire file valid using owserver",
            "\tMore than one PATH can be given",
            "\tReturns 0 (false) or 1 (true)",
            "",
            "OPTIONS",
        ],
    );
    parser_server(owserver, args)?;
    parser_persist(owserver, args)?;
    Ok(())
}

fn arg_size(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    let _ = helper(
        args,
        &[
            "owsize [OPTIONS] [PATH]",
            "\tHow much data would a read potentially return (in bytes)",
            "\tMore than one PATH can be given",
            "",
            "OPTIONS",
        ],
    );
    parser_server(owserver, args)?;
    parser_persist(owserver, args)?;
    Ok(())
}

fn arg_snoop(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    let _ = helper(
        args,
        &[
            "owsnoop {OPTIONS]",
            "\tRelay queries to owserver",
            "\tShows the message contents back and forth",
            "",
            "OPTIONS",
        ],
    );
    parser_server(owserver, args)?;
    parser_listener(owserver, args)?;
    Ok(())
}

fn arg_lib(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    let _ = helper(
        args,
        &[
            "owrust library {OPTIONS]",
            "Configure library",
            "",
            "OPTIONS",
        ],
    );
    parser_server(owserver, args)?;
    parser_listener(owserver, args)?;
    parser_format(owserver, args)?;
    parser_temperature(owserver, args)?;
    parser_pressure(owserver, args)?;
    parser_data(owserver, args)?;
    parser_directory(owserver, args)?;
    parser_persist(owserver, args)?;
    Ok(())
}

fn parser(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<Vec<String>> {
    // Choose the options and help message based on the program calling this function
    if let Some(prog) = progname() {
        match prog.as_str() {
            "owdir" => arg_dir(owserver, args)?,
            "owget" => arg_get(owserver, args)?,
            "owread" => arg_read(owserver, args)?,
            "owwrite" => arg_write(owserver, args)?,
            "owpresent" => arg_present(owserver, args)?,
            "owsize" => arg_size(owserver, args)?,
            "owsnoop" => arg_snoop(owserver, args)?,
            _ => arg_lib(owserver, args)?,
        };
    } else {
        arg_lib(owserver, args)?;
    }

    // debug
    while args.contains(["-d", "--debug"]) {
        owserver.debug += 1;
        eprintln!("Debuging level {}", owserver.debug);
    }

    // Handle the help flag for the trailing message
    if args.contains(["-h", "--help"]) {
        println!();
        println!("General");
        println!("\t-h\t--help\tThis help message");
        println!("\t-d\t--debug\tShow debugging information");
        println!();
        println!("See https://github.com/alfille/owrust for more information");
        process::exit(0);
    }

    // Gather PATH (and VALUES if owwrite) for return
    let mut result: Vec<String> = Vec::new();
    for os in args.clone().finish() {
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

    // owserver use configuration information to set up message parameters
    owserver.make_flags();
    Ok(result)
}

// Write a help message if resuired (from the supplied text)
fn helper(args: &Arguments, text: &[&str]) -> bool {
    // arg clone so help is still active for later help choices
    let mut args_clone = args.clone();
    if args_clone.contains(["-h", "--help"]) {
        for t in text {
            println!("{}", t);
        }
        println!();
        true
    } else {
        false
    }
}

fn parser_temperature(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    if !helper(
        args,
        &[
            "Temperature Scale (default Celsius)",
            "\t-C\t--celsius",
            "\t-F\t--fahrenheit",
            "\t-K\t--kelvin",
            "\t-R\t--rankine",
        ],
    ) {
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
    }
    Ok(())
}

fn parser_pressure(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    // Handle the help flag first
    if !helper(
        args,
        &[
            "Pressure Scale (default mBar)",
            "\t-mmhg  mm Mercury",
            "\t-inhg  inches Mercury",
            "\t-mbar  mili Bar",
            "\t-atm   atmospheres",
            "\t-ps    Pascals",
            "\t-psi   pounds / in^2",
        ],
    ) {
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
    }
    Ok(())
}

fn parser_format(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    if !helper(
        args,
        &[
            "Device format displayed",
            "\t-f\t--format",
            "\t\t\tfi | f.i",
            "\t\t\tfic | fi.c | f.ic | f.i.c",
        ],
    ) {
        // Format
        let d = args.opt_value_from_fn(["-f", "--format"], parse_device)?;
        owserver.format = d.unwrap_or(super::Format::DEFAULT);
    }
    Ok(())
}

fn parser_data(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    if !helper(
        args,
        &[
            "Data display (default text",
            "\t--hex\tShow hexidecimal bytes",
            "\t--size\tLimit data size returned (in bytes)",
            "\t--offset\tposition (in bytes) to start data returned",
        ],
    ) {
        // Display
        if args.contains("--hex") {
            owserver.hex = true;
        }
        let y = args.opt_value_from_str("--size")?;
        if let Some(x) = y {
            owserver.size = x;
        }
        let y = args.opt_value_from_str("--offset")?;
        if let Some(x) = y {
            owserver.offset = x;
        }
    }
    Ok(())
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

fn parser_server(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    if !helper(
        args,
        &[
            "OwServer address (default localhost:4304)",
            "\t-s\t--server\tIp address of owserver to contact",
        ],
    ) {
        // Server
        let serv: Option<String> = args.opt_value_from_str(["-s", "--server"])?;
        if let Some(s) = serv {
            owserver.stream.set_target(&s);
        }
    }
    Ok(())
}

fn parser_listener(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    if !helper(
        args,
        &[
            "Listening address (no default but required)",
            "\t-p\t--port\tIp address this program will answer on",
        ],
    ) {
        // Listener
        let listener: Option<String> = args.opt_value_from_str(["-p", "--port"])?;
        if listener.is_some() {
            owserver.listener = listener;
        }
    }
    Ok(())
}

fn parser_directory(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    if !helper(
        args,
        &[
            "Directory display options",
            "\t--dir\tMark directories with a trailing '/'",
            "\t--bare\tExclude non-device entries",
            "\t--prune\tExclude some convenience device entries (e.g. address)",
        ],
    ) {
        // Slash
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
    }
    Ok(())
}

fn parser_persist(owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
    if !helper(
        args,
        &[
            "Persistance keeps connection to owserver open",
            "\t--persist\tFor better performance on repeated queries",
        ],
    ) {
        // Persist
        if args.contains("--persist") {
            owserver.stream.set_persistence(true);
        }
    }
    Ok(())
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
        let owserver2 = modified_messager(&owserver, args).unwrap();
        assert_eq!(owserver2.listener, None);
    }
    #[test]
    fn port_test() {
        let args: Vec<&str> = vec!["-p", "localhost:14304"];
        let owserver = crate::new();
        let owserver2 = modified_messager(&owserver, args).unwrap();
        assert_eq!(owserver2.listener, Some("localhost:14304".to_string()));
    }
    #[test]
    fn port2_test() {
        let args: Vec<&str> = vec!["--port", "localhost:14304"];
        let owserver = crate::new();
        let owserver2 = modified_messager(&owserver, args).unwrap();
        assert_eq!(owserver2.listener, Some("localhost:14304".to_string()));
    }
}
