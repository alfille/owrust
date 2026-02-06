//! ### BusManage struct
//! Low level management of bus (e.g. close)

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

///pub trait BusManage
pub trait BusManage {
    // attempt to close any device handles before deleting this bus master
    fn close(&mut self) {
    }
}
