# owrust
Rust language library to link to owserver, the 1-wire server to use Dallas 1-wire devices

[Documentation](https://alfille.github.io/owrust/index.html)


The provided files constitute a **Rust project (`owrust`)** designed to interface with the **OWFS (1-Wire File System)** protocol, specifically communicating with an **`owserver`** instance over a network connection (TCP/IP).

The project is structured as a robust **Rust Library** (`src/lib.rs` and modules) that provides the core communication logic, accompanied by a suite of separate **binary executables** (`src/bin/*`) that act as command-line tools mirroring the functionality of the standard C-based OWFS utilities.

Here is an analysis focused on the `/src` and `/src/lib` directories:

***

## 1. Project Core: `src/lib.rs` and `src/error.rs`

The primary goal of the library is to provide the Rust-native abstraction for the proprietary **`owserver` network protocol**.

### `src/lib.rs`: The Public API

The `lib.rs` file defines the core public API of the `owrust` library, centered around the **`OwClient` struct**:

* **`OwClient` Struct:** This is the main configuration and state management structure. It holds:
    * The `owserver` network address (`ip:port`).
    * Client configuration flags, such as **temperature scale** (Celsius, Fahrenheit, etc.), **pressure scale**, and **serial number format**.
    * Methods for all network operations.
* **Core Communication Methods:** The `OwClient` exposes high-level functions that execute the protocol commands:
    * **`dir()` / `dir_all()`:** Lists contents of a 1-Wire directory path.
    * **`read()` / `write()`:** Reads and writes data to a specific 1-Wire device property/file.
    * **`present()`:** Checks if a device or path exists on the 1-Wire bus.
    * **`get()`:** A utility function combining read/dir behavior for convenience.
* **Output Formatting:** The **`show_result(values: Vec<u8>)`** method is essential for converting the raw byte data received from `owserver` into a human-readable format, respecting the client's configured `hex` or standard output flags.

---

### `src/error.rs`: Centralized Error Handling

This module implements comprehensive error handling for the library, a standard practice for robust Rust projects.

* **`OwError` Enum:** Defines a custom error type that wraps various possible failure points:
    * `Io`: Wraps `std::io::Error` (for underlying network/system communication issues).
    * `Args`: Wraps `pico_args::Error` (for command-line argument issues).
    * `Network`: For protocol-specific errors received from `owserver`.
    * `Text`, `Numeric`: For data parsing errors (e.g., failed UTF-8 conversion or non-numeric input).
* **Error Propagation:** It implements the `From` trait for several standard error types, which allows the use of the idiomatic **`?` operator** (`try!` macro in older Rust) for automatic and clean error propagation throughout the library functions.

---

### `src/parse_args.rs`: Configuration & Command Parsing

This module is responsible for initializing the `OwClient` based on runtime arguments.

* **Argument Parsing:** Uses the **`pico-args`** crate to parse CLI options.
* **Configuration Logic:** Parses all standard OWFS configuration options, including:
    * Server address (`-s`/`--server`).
    * Scale settings (`-C`/`--Celsius`, `-F`/`--Farenheit`, etc.).
    * Output control flags (`--dir`, `--hex`, `--bare`, `--prune`).
* **Output:** The primary function, `command_line()`, populates the configuration within the mutable `OwClient` struct and returns a list (`Vec<String>`) of the 1-Wire paths provided by the user.

***

## 2. Executable Binaries (`/src/bin`) Analysis

The project includes seven executable binaries that act as command-line wrappers for the core library functions. They all follow the pattern of configuring an `OwClient` via `parse_args::command_line()` and then invoking the relevant `OwClient` method.

| Binary File | Purpose (OWFS Equivalent) | Primary Library Call | Structure |
| :--- | :--- | :--- | :--- |
| **`owdir.rs`** | Lists contents of a directory (`owdir`). | `owserver.dir()` | Iterates over paths and prints results, usually comma-separated. |
| **`owread.rs`** | Reads the value of a 1-Wire file (`owread`). | `owserver.read()` | Reads the file contents and prints the formatted result to stdout. |
| **`owwrite.rs`**| Writes a value to a 1-Wire file (`owwrite`). | `owserver.write()`| Expects paired `PATH VALUE` arguments and calls `write()` for each pair. |
| **`owpresent.rs`**| Checks if a path exists (`owpresent`). | `owserver.present()`| Prints `1` if the path exists, `0` otherwise. |
| **`owget.rs`** | Reads a file or directory (`owget`). | `owserver.get()` | Uses the versatile `get()` method to handle both file reads and directory listings. |
| **`owtree.rs`** | Displays the bus structure in a Unix-like tree format (`owtree`). | `owserver.dir()` (Recursive) | Contains internal structs (`File`, `Dir`) and logic to handle recursive traversal and format the output with connection characters (`├`, `└──`, `│`). |
