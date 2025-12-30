// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::console::console_lines;
use crate::error::{OwEResult, OwError};
use pico_args::Arguments;
use std::ffi::OsString;
use std::process;

/// ### OwDir
/// Structure encapsulating the command line argument processing and help for **owdir**
///
/// Uses default implementation except function **help_and_options**
pub struct OwDir;
impl Parser for OwDir {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
            args,
            &[
                "owdir [OPTIONS] [PATH]",
                "\tList a 1-wire directory using owserver",
                "\tMore than one PATH can be given",
                "",
                "OPTIONS",
            ],
        );
        self.server_options(owserver, args)?;
        self.directory_options(owserver, args)?;
        self.format_options(owserver, args)?;
        self.persist_options(owserver, args)?;
        Ok(())
    }
}

/// ### OwTree
/// Structure encapsulating the command line argument processing and help for **owtree**
///
/// Uses default implementation except function **help_and_options**
pub struct OwTree;
impl Parser for OwTree {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
            args,
            &[
                "owtree [OPTIONS] [PATH]",
                "\tShow a deep 1-wire directory structure using owserver",
                "\tMore than one PATH can be given",
                "",
                "OPTIONS",
            ],
        );
        self.server_options(owserver, args)?;
        self.directory_options(owserver, args)?;
        self.format_options(owserver, args)?;
        self.persist_options(owserver, args)?;
        // special consideration for owtree -- alway persistent
        owserver.stream.set_persistence(true);
        Ok(())
    }
}

/// ### OwGet
/// Structure encapsulating the command line argument processing and help for **owget**
///
/// Uses default implementation except function **help_and_options**
pub struct OwGet;
impl Parser for OwGet {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
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
        self.server_options(owserver, args)?;
        self.directory_options(owserver, args)?;
        self.format_options(owserver, args)?;
        self.temperature_options(owserver, args)?;
        self.pressure_options(owserver, args)?;
        self.data_options(owserver, args)?;
        self.persist_options(owserver, args)?;
        Ok(())
    }
}

/// ### OwRead
/// Structure encapsulating the command line argument processing and help for **owread**
///
/// Uses default implementation except function **help_and_options**
pub struct OwRead;
impl Parser for OwRead {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
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
        self.server_options(owserver, args)?;
        self.temperature_options(owserver, args)?;
        self.pressure_options(owserver, args)?;
        self.data_options(owserver, args)?;
        self.persist_options(owserver, args)?;
        Ok(())
    }
}

/// ### OwWrite
/// Structure encapsulating the command line argument processing and help for **owwrite**
///
/// Uses default implementation except function **help_and_options**
pub struct OwWrite;
impl Parser for OwWrite {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
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
        self.server_options(owserver, args)?;
        self.temperature_options(owserver, args)?;
        self.pressure_options(owserver, args)?;
        self.data_options(owserver, args)?;
        self.persist_options(owserver, args)?;
        Ok(())
    }
}

/// ### OwSize
/// Structure encapsulating the command line argument processing and help for **owsize**
///
/// Uses default implementation except function **help_and_options**
pub struct OwSize;
impl Parser for OwSize {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
            args,
            &[
                "owsize [OPTIONS] [PATH]",
                "\tHow much data would a read potentially return (in bytes)",
                "\tMore than one PATH can be given",
                "",
                "OPTIONS",
            ],
        );
        self.server_options(owserver, args)?;
        self.persist_options(owserver, args)?;
        Ok(())
    }
}

/// ### OwPresent
/// Structure encapsulating the command line argument processing and help for **owpresent**
///
/// Uses default implementation except function **help_and_options**
pub struct OwPresent;
impl Parser for OwPresent {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
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
        self.server_options(owserver, args)?;
        self.persist_options(owserver, args)?;
        Ok(())
    }
}

/// ### OwSnoop
/// Structure encapsulating the command line argument processing and help for **owsnoop**
///
/// Uses default implementation except function **help_and_options**
pub struct OwSnoop;
impl Parser for OwSnoop {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
            args,
            &[
                "owsnoop {OPTIONS]",
                "\tRelay queries to owserver",
                "\tShows the message contents back and forth",
                "",
                "OPTIONS",
            ],
        );
        self.server_options(owserver, args)?;
        self.listener_options(owserver, args)?;
        Ok(())
    }
}

/// ### OwLib
/// Structure encapsulating the command line argument processing and help for generic implementation
///
/// Uses default implementation except function **help_and_options**
pub struct OwLib;
impl Parser for OwLib {
    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        let _ = self.helper(
            args,
            &[
                "owrust library {OPTIONS]",
                "Configure library",
                "",
                "OPTIONS",
            ],
        );
        self.server_options(owserver, args)?;
        self.listener_options(owserver, args)?;
        self.format_options(owserver, args)?;
        self.temperature_options(owserver, args)?;
        self.pressure_options(owserver, args)?;
        self.data_options(owserver, args)?;
        self.directory_options(owserver, args)?;
        self.persist_options(owserver, args)?;
        Ok(())
    }
}

/// ### Parser trait
/// Handles commandline parsing and help
///
/// * Requires implementation of **help_and_options**
/// * **command_line** reads from the command line
/// * **vector_line** reads from an array of String arguments (useful for testing or internal configuration)
/// * **xxx_options** are bundles of options with common usage, including related help
/// * **helper** prints out help text
pub trait Parser {
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
    /// use owrust::parse_args::{Parser,OwLib} ;
    /// let mut owserver = owrust::new() ; // new OwMessage structure
    /// let prog = OwLib ;
    /// let paths = prog.command_line( &mut owserver ).expect("Bad configuration");
    /// ```
    fn command_line(&self, owserver: &mut crate::OwMessage) -> OwEResult<Vec<String>> {
        // normal path -- from environment
        let mut args = Arguments::from_env();
        self.parser(owserver, &mut args)
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
    /// use owrust::parse_args::{Parser,OwLib} ;
    /// let mut owserver = owrust::new() ; // new OwMessage structure
    /// let prog = OwLib ;
    /// let args: Vec<&str> = vec!(
    ///     "-C",
    ///     "--bare",
    ///     "/bus.0"
    ///     );
    /// let paths = prog.vector_line( &mut owserver, args ).expect("Bad configuration");
    /// ```
    fn vector_line(
        &self,
        owserver: &mut crate::OwMessage,
        args: Vec<&str>,
    ) -> OwEResult<Vec<String>> {
        // normal path -- from environment
        // convert Vec<String> to Vec<OsString>
        let os_args: Vec<OsString> = args.iter().map(OsString::from).collect();
        self.parser(owserver, &mut Arguments::from_vec(os_args))
    }

    fn help_and_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()>;

    fn parser(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<Vec<String>> {
        // Choose the options and help message based on the program calling this function
        self.help_and_options(owserver, args)?;

        // debug
        while args.contains(["-d", "--debug"]) {
            owserver.debug += 1;
            eprintln!("Debuging level {}", owserver.debug);
        }

        // Handle the help flag for the trailing message
        if args.contains(["-h", "--help"]) {
            console_lines([
                "",
                "General",
                "\t-h\t--help\tThis help message",
                "\t-d\t--debug\tShow debugging information",
                "",
                "See https://github.com/alfille/owrust for more information",
            ]);
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
    fn helper(&self, args: &Arguments, text: &[&str]) -> bool {
        // arg clone so help is still active for later help choices
        let mut args_clone = args.clone();
        if args_clone.contains(["-h", "--help"]) {
            console_lines(text);
            true
        } else {
            false
        }
    }

    fn temperature_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        if !self.helper(
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

    fn pressure_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        // Handle the help flag first
        if !self.helper(
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

    fn format_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        if !self.helper(
            args,
            &[
                "Device format displayed",
                "\t-f\t--format",
                "\t\t\tfi | f.i",
                "\t\t\tfic | fi.c | f.ic | f.i.c",
            ],
        ) {
            // Format
            let d = args.opt_value_from_fn(["-f", "--format"], format_match)?;
            owserver.format = d.unwrap_or(super::Format::DEFAULT);
        }
        Ok(())
    }

    fn data_options(&self, owserver: &mut crate::OwMessage, args: &mut Arguments) -> OwEResult<()> {
        if !self.helper(
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

    fn server_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        if !self.helper(
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

    fn listener_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        if !self.helper(
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

    fn directory_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        if !self.helper(
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

    fn persist_options(
        &self,
        owserver: &mut crate::OwMessage,
        args: &mut Arguments,
    ) -> OwEResult<()> {
        if !self.helper(
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
}

fn format_match(s: &str) -> OwEResult<super::Format> {
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
                let prog = OwLib;
                let _ = prog.vector_line(&mut owserver, args);
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
                let prog = OwLib;
                let _ = prog.vector_line(&mut owserver, args);
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
                let mut owserver = crate::new();
                let prog = OwLib;
                let _ = prog.vector_line(&mut owserver, args);
                owserver.make_flags();
                let result = owserver.flags & ts.1;
                assert_eq!(result, ts.1);
            }
        }
    }
    #[test]
    fn noport_test() {
        let args: Vec<&str> = vec![];
        let mut owserver = crate::new();
        let prog = OwLib;
        let _ = prog.vector_line(&mut owserver, args);
        owserver.make_flags();
        assert_eq!(owserver.listener, None);
    }
    #[test]
    fn port_test() {
        let args: Vec<&str> = vec!["-p", "localhost:14304"];
        let mut owserver = crate::new();
        let prog = OwLib;
        let _ = prog.vector_line(&mut owserver, args);
        owserver.make_flags();
        assert_eq!(owserver.listener, Some("localhost:14304".to_string()));
    }
    #[test]
    fn port2_test() {
        let args: Vec<&str> = vec!["--port", "localhost:14304"];
        let mut owserver = crate::new();
        let prog = OwLib;
        let _ = prog.vector_line(&mut owserver, args);
        owserver.make_flags();
        assert_eq!(owserver.listener, Some("localhost:14304".to_string()));
    }
    
    fn has_help<P: Parser>(prog: P) {
        let mut owserver = crate::new();
		let result = prog.vector_line(&mut owserver, vec!("-h"));
		assert!(result.is_ok(),"help available {:?}", result) ;
	}
	
	#[test]
	fn all_help() {
		has_help( OwDir ) ;
		has_help( OwGet ) ;
		has_help( OwRead ) ;
		has_help( OwWrite ) ;
		has_help( OwLib ) ;
		has_help( OwPresent ) ;
		has_help( OwSize ) ;
		has_help( OwSnoop ) ;
		has_help( OwTree ) ;
	}

    fn has_server<P: Parser>(prog: P) {
        let mut owserver = crate::new();
		let result = prog.vector_line(&mut owserver, vec!("-s","localhost:4304"));
		assert!(result.is_ok(),"server available {:?}", result) ;
	}
	
	#[test]
	fn all_server() {
		has_server( OwDir ) ;
		has_server( OwGet ) ;
		has_server( OwRead ) ;
		has_server( OwWrite ) ;
		has_server( OwLib ) ;
		has_server( OwPresent ) ;
		has_server( OwSize ) ;
		has_server( OwSnoop ) ;
		has_server( OwTree ) ;
	}
}
