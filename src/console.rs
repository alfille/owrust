//! ### console module
//! Manage output to screen (stdout)
//! * Thread-safe with automic entries
//! * cli safe for broken_pipe so can manage "head" other command line manipulations
//! * Should only be used on CLI (command line interface) programs like owtree
//!   * exits on error (bad for library)
//!   * no recovery possible
//!
//! Credit to Gemini AI for much of the general code design

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use std::io::{self, ErrorKind, Stdout, Write};
use std::process;
use std::sync::{Mutex, OnceLock};

/// local shared state
static GLOBAL_STDOUT: OnceLock<Mutex<Stdout>> = OnceLock::new();

/// Internal initialization of mutex
fn get_handle() -> &'static Mutex<Stdout> {
    GLOBAL_STDOUT.get_or_init(|| Mutex::new(io::stdout()))
}

/// Internal helper to handle IO errors specifically for BrokenPipe
/// * clean program exit (0) on Broken Pipe
/// * annotated program exit (1) on other IO errors
fn handle_io_result(result: io::Result<()>) {
    if let Err(e) = result {
        if e.kind() == ErrorKind::BrokenPipe {
            // The pipe was closed by the receiver.
            // We exit silently (0) as is standard for CLI tools.
            process::exit(0);
        } else {
            // For other errors (disk full, etc.), we still want to know.
            eprintln!("Unexpected IO error: {}", e);
            process::exit(1);
        }
    }
}

/// ### write_line
/// Simple single line output
/// * atomic output to stdout
/// * &str input
/// #### Example
/// ```
/// use owrust::console_line;
/// console_line("Hello");
///```
pub fn console_line(message: &str) {
    // aquire mutex
    let mut guard = get_handle().lock().expect("Mutex poisoned");

    // Use writeln! and pass the result to our handler
    let result = writeln!(guard, "{}", message);
    handle_io_result(result);
}

/// ### console_lines
/// Write a series of lines to the console (stdout) atomically
/// * Generic function: Works with anything that can be treated as a string slice
/// * Handles Broken Pipe gracefully
/// #### Example
/// ```
/// use owrust::console_lines;
/// let text_lines = [
///   "Opening stanza",
///   "Meat of the problem",
///   "Reiteration",
/// ];
/// console_lines(&text_lines);
///```
pub fn console_lines<T, S>(lines: T)
where
    T: IntoIterator<Item = S>, // Can be iterated over
    S: AsRef<str>,             // Items can be seen as &str
{
    // aquire mutex
    let mut guard = get_handle().lock().expect("Mutex poisoned");

    for line in lines {
        let result = writeln!(guard, "{}", line.as_ref());
        handle_io_result(result);
    }
}
