
// Testing the Nozzle Rust APIs
extern crate rust_kronosnet as kronosnet;
use kronosnet::libnozzle as nozzle;
use std::io::{Result, Error, ErrorKind};
use std::env;
use std::{thread, time};

fn main() -> Result<()>
{
    let mut nozzle_name = String::from("rustnoz");
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

    match nozzle::add_ip(handle, &"192.160.100.1".to_string(), &"24".to_string()){
	Ok(_) => {},
	Err(e) => {
	    println!("Error from add_ip: {}", e);
	    return Err(e);
	}
    }
    match nozzle::add_ip(handle, &"192.160.100.2".to_string(), &"24".to_string()){
	Ok(_) => {},
	Err(e) => {
	    println!("Error from add_ip2: {}", e);
	    return Err(e);
	}
    }
    match nozzle::add_ip(handle, &"192.160.100.3".to_string(), &"24".to_string()){
	Ok(_) => {},
	Err(e) => {
	    println!("Error from add_ip3: {}", e);
	    return Err(e);
	}
    }

    match nozzle::set_mac(handle, &"AA:00:04:00:22:01".to_string()){
	Ok(_) => {},
	Err(e) => {
	    println!("Error from set_mac: {}", e);
	    return Err(e);
	}
    }

    match nozzle::set_mtu(handle, 157){
	Ok(_) => {},
	Err(e) => {
	    println!("Error from set_mtu: {}", e);
	    return Err(e);
	}
    }

    match nozzle::set_up(handle){
	Ok(_) => {},
	Err(e) => {
	    println!("Error from set_up: {}", e);
	    return Err(e);
	}
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
	    println!("");
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
    thread::sleep(time::Duration::from_millis(10000));

    match nozzle::set_down(handle){
	Ok(_) => {},
	Err(e) => {
	    println!("Error from set_down: {}", e);
	    return Err(e);
	}
    }

    match nozzle::close(handle) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from open: {}", e);
	    return Err(e);
	}
    }
    Ok(())
}
