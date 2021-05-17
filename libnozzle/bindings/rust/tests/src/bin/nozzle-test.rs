// Testing the Nozzle Rust APIs
//
// Copyright (c) 2021 Red Hat, Inc.
//
// All rights reserved.
//
// Author: Christine Caulfield (ccaulfi@redhat.com)
//

use libnozzle::libnozzle as nozzle;
use std::io::{Result, Error, ErrorKind};
use std::env;
use std::{thread, time};

const SKIP: i32 = 77;

fn main() -> Result<()>
{
    // We must be root
    if unsafe { libc::geteuid() != 0 } {
	std::process::exit(SKIP);
    }


    // Name must be tapXX for it to work on FreeBSD
    let mut nozzle_name = String::from("tap33");
    let handle = match nozzle::open(&mut nozzle_name,  &String::from(env::current_dir().unwrap().to_str().unwrap())) {
	Ok(h) => {
	    println!("Opened device {}", nozzle_name);
	    h
	},
	Err(e) => {
	    println!("Error from open: {}", e);
	    return Err(e);
	}
    };

    // Get default state for checking reset_* calls later
    let saved_mtu = match nozzle::get_mtu(handle) {
	Ok(m) => m,
	Err(e) => {
	    println!("Error from get_mtu: {}", e);
	    return Err(e);
	}
    };
    let saved_mac = match nozzle::get_mac(handle) {
	Ok(m) => m,
	Err(e) => {
	    println!("Error from get_mac: {}", e);
	    return Err(e);
	}
    };

    // Play with APIs
    if let Err(e) = nozzle::add_ip(handle, &"192.160.100.1", &"24") {
	println!("Error from add_ip: {}", e);
	return Err(e);
    }
    if let Err(e) = nozzle::add_ip(handle, &"192.160.100.2", &"24") {
	println!("Error from add_ip2: {}", e);
	return Err(e);
    }
    if let Err(e) = nozzle::add_ip(handle, &"192.160.100.3", &"24") {
	println!("Error from add_ip3: {}", e);
	return Err(e);
    }

    if let Err(e) = nozzle::set_mac(handle, &"AA:00:04:00:22:01") {
	println!("Error from set_mac: {}", e);
	return Err(e);
    }

    if let Err(e) = nozzle::set_mtu(handle, 157) {
	println!("Error from set_mtu: {}", e);
	return Err(e);
    }

    if let Err(e) = nozzle::set_up(handle){
	println!("Error from set_up: {}", e);
	return Err(e);
    }

    match nozzle::run_updown(handle, nozzle::Action::Up){
	Ok(s) => println!("Returned from Up script: {}", s),
	Err(e) => {
	    println!("Error from run_updown: {}", e);
	    return Err(e);
	}
    }


    match nozzle::get_ips(handle) {
	Ok(ips) => {
	    print!("Got IPs:");
	    for i in ips {
		print!(" {}", i);
	    }
	    println!();
	},
	Err(e) => {
	    println!("Error from get_ips: {}", e);
	    return Err(e);
	}
    }

    match nozzle::get_mtu(handle) {
	Ok(m) => println!("Got mtu: {}", m),
	Err(e) => {
	    println!("Error from get_ips: {}", e);
	    return Err(e);
	}
    }
    match nozzle::get_mac(handle) {
	Ok(m) => println!("Got mac: {}", m),
	Err(e) => {
	    println!("Error from get_ips: {}", e);
	    return Err(e);
	}
    }

    match nozzle::get_fd(handle) {
	Ok(f) => println!("Got FD: {}", f),
	Err(e) => {
	    println!("Error from get_fd: {}", e);
	    return Err(e);
	}
    }

    match nozzle::get_handle_by_name(&nozzle_name) {
	Ok(h) => if h != handle {
	    return Err(Error::new(ErrorKind::Other, "get_handle_by_name returned wrong value"));
	}
	Err(e) => {
	    println!("Error from get_ips: {}", e);
	    return Err(e);
	}
    }

    match nozzle::get_name_by_handle(handle) {
	Ok(n) => if n != nozzle_name {
	    println!("n: {}, nozzle_name: {}", n, nozzle_name);
	    return Err(Error::new(ErrorKind::Other, "get_name_by_handle returned wrong name"));
	}
	Err(e) => {
	    println!("Error from get_ips: {}", e);
	    return Err(e);
	}
    }

    // Wait a little while in case user wants to check with 'ip' command
    thread::sleep(time::Duration::from_millis(1000));

    if let Err(e) = nozzle::del_ip(handle, &"192.160.100.3", &"24") {
	println!("Error from del_ip: {}", e);
	return Err(e);
    }

    if let Err(e) = nozzle::reset_mtu(handle) {
	println!("Error from reset_mtu: {}", e);
	return Err(e);
    }
    match nozzle::get_mtu(handle) {
	Ok(m) => {
	    if m != saved_mtu {
		println!("Got default MTU of {}, not  {}", m, saved_mtu);
	    }
	}
	Err(e) => {
	    println!("Error from get_ips: {}", e);
	    return Err(e);
	}
    }

    if let Err(e) = nozzle::reset_mac(handle) {
	println!("Error from reset_mac: {}", e);
	return Err(e);
    }
    match nozzle::get_mac(handle) {
	Ok(m) => {
	    if m != saved_mac {
		println!("Got default MAC of {}, not  {}", m, saved_mac);
	    }
	}
	Err(e) => {
	    println!("Error from get_ips: {}", e);
	    return Err(e);
	}
    }


    if let Err(e) = nozzle::set_down(handle){
	println!("Error from set_down: {}", e);
	return Err(e);
    }

    if let Err(e) = nozzle::close(handle) {
	println!("Error from open: {}", e);
	return Err(e);
    }
    Ok(())
}
