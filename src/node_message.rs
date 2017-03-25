//! Contains the LCM message type used to represent a node.
//!
//! ID's should be unique among all nodes broadcast on the given channel,
//! which usually means the nodes need to be unique within the behavior
//! tree. The `uid()` function can be used to get a unique ID.

#![allow(missing_docs)]

// If there is more than one message that is generated and/or you
// want to preserve the mod heirarchy, there is a compiler plugin
// called mod_path! which would be very useful.
include!(concat!(env!("OUT_DIR"), "/node_message/mod.rs"));
