//! This crate provides access to the kronosnet libknet
//! from Rust. They are a fairly thin layer around the actual API calls but with Rust data types
//! and iterators.
//!
//! No more information about knet itself will be provided here, it is expected that if
//! you feel you need access to the knet API calls, you know what they do :)
//!
//! # Example
//! extern crate rust_kronosnet as kronosnet;
//! use kronosnet::libknet as knet;
//! use std::net::{SocketAddr, IpAddr,Ipv4Addr};
//! use std::thread::spawn;
//! use std::sync::mpsc::Receiver;
//! use std::sync::mpsc::channel;
//! use std::io::{Result, ErrorKind, Error};
//! use std::{thread, time};
//!
//! const CHANNEL: i8 = 1;

//! pub fn main() -> Result<()>
//! {
//!     let host_id = knet::HostId::new(1);
//!     let other_host_id = knet::HostId::new(2);
//!
//!     let (log_sender, log_receiver) = channel::<knet::LogMsg>();
//!     spawn(move || logging_thread(log_receiver));
//!
//!     let knet_handle = match knet::handle_new(&our_hostid, Some(log_sender),
//! 					         knet::LogLevel::Debug, knet::HandleFlags::NONE) {
//! 	    Ok(h) => h,
//! 	    Err(e) => {
//! 	        return Err(e);
//! 	    }
//!     };
//!
//!     match knet::host_add(knet_handle, &other_hostid) {
//! 	    Ok(_) => {},
//! 	        Err(e) => {
//! 	        return Err(e);
//! 	    }
//!     }
//!     match knet::link_set_config(knet_handle, &other_hostid, 0,
//! 				    knet::TransportId::Udp,
//! 				    &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000+(our_hostid.to_u16())),
//!				    &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000+(other_hostid.to_u16())),
//! 				    knet::LinkFlags::NONE) {
//! 	    Ok(_) => {},
//!         Err(e) => {
//! 	        return Err(e);
//! 	    }
//!     }
//!     match knet::handle_add_datafd(knet_handle, 0, CHANNEL) {
//! 	        Ok(_) => {
//! 	        },
//! 	        Err(e) => {
//! 	            return Err(e);
//! 	    }
//!     }
//!
//!     match knet::handle_crypto_rx_clear_traffic(knet_handle, knet::RxClearTraffic::Allow) {
//! 	    Ok(_) => {},
//! 	    Err(e) => {
//! 	        return Err(e);
//! 	    }
//!     }
//!
//!     match knet::link_set_enable(knet_handle, &other_hostid, 0, true) {
//! 	    Ok(_) => {},
//! 	    Err(e) => {
//! 	        return Err(e);
//! 	    }
//!     }
//!
//!     match knet::handle_set_fwd(knet_handle, true) {
//! 	    Ok(_) => {},
//! 	    Err(e) => {
//! 	        return Err(e);
//! 	    }
//!     }
//!
//!     Ok()
//! }
//!


mod sys;
pub mod libknet;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

use std::os::raw::c_char;
use std::ptr::copy_nonoverlapping;
use std::ffi::CString;
use std::io::{Error, Result, ErrorKind};


// Quick & dirty u8 to boolean
fn u8_to_bool(val: u8) -> bool
{
    if val == 0 {false} else {true}
}

fn u32_to_bool(val: u32) -> bool
{
    if val == 0 {false} else {true}
}

// General internal routine to copy bytes from a C array into a Rust String
fn string_from_bytes(bytes: *const ::std::os::raw::c_char, max_length: usize) -> Result<String>
{
    let mut newbytes = Vec::<u8>::new();
    newbytes.resize(max_length, 0u8);

    unsafe {
	// We need to fully copy it, not shallow copy it.
	// Messy casting on both parts of the copy here to get it to work on both signed
	// and unsigned char machines
	copy_nonoverlapping(bytes as *mut i8, newbytes.as_mut_ptr() as *mut i8, max_length);
    }

    // Get length of the string in old-fashioned style
    let mut length: usize = 0;
    let mut count : usize = 0;
    for i in &newbytes {
	if *i == 0 && length == 0 {
	    length = count;
	    break;
	}
	count += 1;
    }

    // Cope with an empty string
    if length == 0 {
	return Ok(String::new());
    }

    let cs = CString::new(&newbytes[0..length as usize])?;

    // This is just to convert the error type
    match cs.into_string() {
	Ok(s) => Ok(s),
	Err(_) => Err(Error::new(ErrorKind::Other, "Cannot convert to String")),
    }
}

// As below but always returns a string even if there was an error doing the conversion
fn string_from_bytes_safe(bytes: *const ::std::os::raw::c_char, max_length: usize) -> String
{
    match string_from_bytes(bytes, max_length) {
	Ok(s) => s,
	Err(_)=> "".to_string()
    }
}

fn string_to_bytes(s: &String, bytes: &mut [c_char]) ->Result<()>
{
    let c_name = match CString::new(s.as_str()) {
	Ok(n) => n,
	Err(_) => return Err(Error::new(ErrorKind::Other, "Rust conversion error")),
    };

    if c_name.as_bytes().len() > bytes.len() {
	return Err(Error::new(ErrorKind::Other, "String too long"));
    }
    unsafe {
	// NOTE param order is 'wrong-way round' from C
	copy_nonoverlapping(c_name.as_ptr(), bytes.as_mut_ptr(), bytes.len());
    }
    Ok(())
}
