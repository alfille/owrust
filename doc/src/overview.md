# Overview

This is the *Rust* language access to [OWFS](owfs_classic.md) for using [1-Wire](1wire/md) devices.

## Components

### Library

* API: [owrust](https://alfille.github.io/owrust/api/owrust/index.html)
* Connects to a running **owserver** over TCP networking for access
* Can run locally
* Simple calls to Read, Write, and List 1-wire devices and device properties
* No *unsafe* code


### Command Line (Shell) programs

* Similar to C-language **owshell** utilities
* Includes
  * [owread](https://alfille.github.io/owrust/api/owread/index.html) to read device properties
  * [owdir](https://alfille.github.io/owrust/api/owdir/index.html) to list devices or properties
  * [owget](https://alfille.github.io/owrust/api/owget/index.html) combining **owread** and **owdir**
  * [owwrite](https://alfille.github.io/owrust/api/owwrite/index.html) To set device properties or memory contents
* Add new functionality
  * [owtree](https://alfille.github.io/owrust/api/owtree/index.html) to show full directory structure
* Also includes for historical completeness
  * [owsize]()
  * [owpresent](https://alfille.github.io/owrust/api/owpresent/index.html)
