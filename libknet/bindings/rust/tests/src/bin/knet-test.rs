// Testing the Knet Rust APIs
//
// Copyright (c) 2021 Red Hat, Inc.
//
// All rights reserved.
//
// Author: Christine Caulfield (ccaulfi@redhat.com)
//

use kronosnet::libknet as knet;
use std::net::{SocketAddr, IpAddr,Ipv4Addr};
use std::thread::spawn;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::io::{Result, ErrorKind, Error};
use std::{thread, time};

const CHANNEL: i8 = 1;

// Callbacks
fn sock_notify_fn(private_data: u64,
		  datafd: i32,
		  channel: i8,
		  txrx: knet::TxRx,
		  _res: Result<()>)
{
    println!("sock notify called for host {}, datafd: {}, channel: {}, {}",
	     private_data, datafd, channel, txrx);
}


fn link_notify_fn(private_data: u64,
		  host_id: knet::HostId,
		  link_id: u8,
		  connected: bool,
		  _remote: bool,
		  _external: bool)
{
    println!("link status notify called ({}) for host {}, linkid: {}, connected: {}",
	     private_data, host_id.to_u16(), link_id, connected);
}

fn host_notify_fn(private_data: u64,
		  host_id: knet::HostId,
		  connected: bool,
		  _remote: bool,
		  _external: bool)
{
    println!("host status notify called ({}) for host {}, connected: {}",
	     private_data, host_id.to_u16(), connected);
}

fn filter_fn(private_data: u64,
	     _outdata: &[u8],
	     txrx: knet::TxRx,
	     this_host_id: knet::HostId,
	     src_host_id: knet::HostId,
	     channel: &mut i8,
	     dst_host_ids: &mut Vec<knet::HostId>) -> knet::FilterDecision
{
    println!("Filter ({}) called {} to {} from {}, channel: {}",
	     private_data, txrx, this_host_id, src_host_id, channel);

    match txrx {
	knet::TxRx::Tx => {
	    knet::FilterDecision::Multicast
	}
	knet::TxRx::Rx => {
	    dst_host_ids.push(this_host_id);
	    knet::FilterDecision::Unicast
	}
    }
}

fn logging_thread(recvr: Receiver<knet::LogMsg>)
{
    loop {
	for i in &recvr {
	    eprintln!("KNET: {}", i.msg);
	}
    }
}

fn setup_node(our_hostid: &knet::HostId, other_hostid: &knet::HostId) -> Result<knet::Handle>
{
    let (log_sender, log_receiver) = channel::<knet::LogMsg>();
    spawn(move || logging_thread(log_receiver));

    let knet_handle = match knet::handle_new(&our_hostid, Some(log_sender),
					     knet::LogLevel::Debug, knet::HandleFlags::NONE) {
	Ok(h) => h,
	Err(e) => {
	    println!("Error from handle_new: {}", e);
	    return Err(e);
	}
    };

    match knet::host_add(knet_handle, &other_hostid) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from host_add: {}", e);
	    return Err(e);
	}
    }
    match knet::link_set_config(knet_handle, &other_hostid, 0,
				knet::TransportId::Udp,
				&SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000+(our_hostid.to_u16())),
				&SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000+(other_hostid.to_u16())),
				knet::LinkFlags::NONE) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from link_set_config: {}", e);
	    return Err(e);
	}
    }
    match knet::handle_enable_sock_notify(knet_handle, our_hostid.to_u16() as u64, Some(sock_notify_fn)) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from handle_enable_sock_notify: {}", e);
	    return Err(e);
	}
    }

    match knet::link_enable_status_change_notify(knet_handle, our_hostid.to_u16() as u64, Some(link_notify_fn)) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from handle_enable_link_notify: {}", e);
	    return Err(e);
	}
    }
    match knet::host_enable_status_change_notify(knet_handle, our_hostid.to_u16() as u64, Some(host_notify_fn)) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from handle_enable_host_notify: {}", e);
	    return Err(e);
	}
    }
    match knet::handle_enable_filter(knet_handle, our_hostid.to_u16() as u64, Some(filter_fn)) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from handle_enable_filter: {}", e);
	    return Err(e);
	}
    }
    match knet::handle_add_datafd(knet_handle, 0, CHANNEL) {
	Ok((fd,chan)) => {
	    println!("Added datafd, fd={}, channel={}", fd, chan);
	},
	Err(e) => {
	    println!("Error from add_datafd: {}", e);
	    return Err(e);
	}
    }

    match knet::handle_crypto_rx_clear_traffic(knet_handle, knet::RxClearTraffic::Allow) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from handle_crypto_rx_clear_traffic: {}", e);
	    return Err(e);
	}
    }

    match knet::link_set_enable(knet_handle, &other_hostid, 0, true) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from set_link_enable(true): {}", e);
	    return Err(e);
	}
    }

    match knet::link_set_ping_timers(knet_handle, &other_hostid, 0,
				     500, 1000, 1024) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from set_link_ping_timers: {}", e);
	    return Err(e);
	}
    }

    match knet::link_get_ping_timers(knet_handle, &other_hostid, 0) {
	Ok((a,b,c)) => {
	    if a != 500 || b != 1000 || c != 1024 {
		println!("get_link_ping_timers return wronte values {}, {},{} (s/b 500,1000,1024)",
			 a,b,c);
		return Err(Error::new(ErrorKind::Other, "Error in ping timers"));
	    }
	},
	Err(e) => {
	    println!("Error from set_link_ping_timers: {}", e);
	    return Err(e);
	}
    }


    match knet::handle_setfwd(knet_handle, true) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from setfwd(true): {}", e);
	    return Err(e);
	}
    }

    Ok(knet_handle)
}

fn recv_stuff(handle: knet::Handle, host: knet::HostId) -> Result<()>
{
    let mut buf = [0u8; 1024];

    loop {
	match knet::recv(handle, &mut buf, CHANNEL) {
	    Ok(len) => {
		let recv_len = len as usize;
		if recv_len == 0 {
		    break; // EOF??
		} else {
		    let s = String::from_utf8(buf[0..recv_len].to_vec()).unwrap();
		    println!("recvd on {}: {} {:?}  {} ", host, recv_len, &buf[0..recv_len], s);
		    if s == "QUIT".to_string() {
			println!("got QUIT on {}, exitting", host);
			break;
		    }
		}
	    }
	    Err(e) => {
		if e.kind() == ErrorKind::WouldBlock {
		    thread::sleep(time::Duration::from_millis(100));
		} else {
		    println!("recv failed: {}", e);
		    return Err(e);
		}
	    }
	}
    }
    Ok(())
}


fn close_handle(handle: knet::Handle, remnode: u16) -> Result<()>
{
    let other_hostid = knet::HostId::new(remnode);

    match knet::handle_setfwd(handle, false) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from setfwd 1 (false): {}", e);
	    return Err(e);
	}
    }

    match knet::link_set_enable(handle, &other_hostid, 0, false) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from set_link_enable(false): {}", e);
	    return Err(e);
	}
    }

    match knet::link_clear_config(handle, &other_hostid, 0) {
	Ok(_) => {},
	Err(e) => {
	    println!("clear config failed: {}", e);
	    return Err(e);
	}
    }

    match knet::host_remove(handle, &other_hostid) {
	Ok(_) => {},
	Err(e) => {
	    println!("host remove failed: {}", e);
	    return Err(e);
	}
    }

    match knet::handle_free(handle) {
	Ok(_) => {},
	Err(e) => {
	    println!("handle_free failed: {}", e);
	    return Err(e);
	}
    }
    Ok(())
}


fn set_compress(handle: knet::Handle) -> Result<()>
{
    let compress_config = knet::CompressConfig {
	compress_model: "zlib".to_string(),
	compress_threshold : 10,
	compress_level: 1,
    };
    match knet::handle_compress(handle, &compress_config) {
	Ok(_) => Ok(()),
	Err(e) => {
	    println!("Error from handle_compress: {}", e);
	    Err(e)
	}
    }
}

fn set_crypto(handle: knet::Handle) -> Result<()>
{
    let private_key = [55u8; 2048];

    // Add some crypto
    let crypto_config = knet::CryptoConfig {
	crypto_model: "openssl".to_string(),
	crypto_cipher_type: "aes256".to_string(),
	crypto_hash_type: "sha256".to_string(),
	private_key: &private_key,
    };

    match knet::handle_crypto_set_config(handle, &crypto_config, 1) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from handle_crypto_set_config: {}", e);
	    return Err(e);
	}
    }

    match knet::handle_crypto_use_config(handle, 1) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from handle_crypto_use_config: {}", e);
	    return Err(e);
	}
    }

    match knet::handle_crypto_rx_clear_traffic(handle, knet::RxClearTraffic::Disallow) {
	Ok(_) => {},
	Err(e) => {
	    println!("Error from handle_crypto_rx_clear_traffic: {}", e);
	    return Err(e);
	}
    }
    Ok(())
}


fn send_messages(handle: knet::Handle, send_quit: bool) -> Result<()>
{
    let mut buf : [u8; 20] = [b'0'; 20];
    for i in 0..10 {
	buf[i as usize + 1] = i + b'0';
	match knet::send(handle, &buf, CHANNEL) {
	    Ok(len) => {
		if len as usize != buf.len() {
		    println!("sent {} bytes instead of {}", len, buf.len());
		}
	    },
	    Err(e) => {
		println!("send failed: {}", e);
		return Err(e);
	    }
	}
    }
    if send_quit {
	let b = String::from("QUIT").into_bytes();
	match knet::send(handle, &b, CHANNEL) {
	    Ok(len) => {
		if len as usize != b.len() {
		    println!("sent {} bytes instead of {}", len, b.len());
		}
	    },
	    Err(e) => {
		println!("send failed: {}", e);
		return Err(e);
	    }
	}
    }
    Ok(())
}

fn test_link_host_list(handle: knet::Handle) -> Result<()>
{
    match knet::host_get_host_list(handle) {
	Ok(hosts) => {
	    for i in &hosts {
		print!("host {}: links: ", i);
		match knet::link_get_link_list(handle, i) {
		    Ok(ll) => {
			for j in ll {
			    print!(" {}",j);
			}
		    },
		    Err(e) => {
			println!("link_get_link_list failed: {}", e);
			return Err(e);
		    }
		}
		println!("");
	    }
	}
	Err(e) => {
	    println!("link_get_host_list failed: {}", e);
	    return Err(e);
	}
    }
    Ok(())
}


fn main() -> Result<()>
{
    // Start with some non-handle information
    match knet::get_crypto_list() {
	Ok(l) => {
	    print!("Crypto models:");
	    for i in &l {
		print!(" {}", i.name);
	    }
	    println!("");
	}
	Err(e) => {
	    println!("link_get_crypto_list failed: {:?}", e);
	    return Err(e);
	}
    }

    match knet::get_compress_list() {
	Ok(l) => {
	    print!("Compress models:");
	    for i in &l {
		print!(" {}", i.name);
	    }
	    println!("");
	}
	Err(e) => {
	    println!("link_get_compress_list failed: {:?}", e);
	    return Err(e);
	}
    }

    match knet::get_transport_list() {
	Ok(l) => {
	    print!("Transports:");
	    for i in &l {
		print!(" {}", i.name);
	    }
	    println!("");
	}
	Err(e) => {
	    println!("link_get_transport_list failed: {:?}", e);
	    return Err(e);
	}
    }
    let host1 = knet::HostId::new(1);
    let host2 = knet::HostId::new(2);

    // Now test traffic
    let handle1 = setup_node(&host1, &host2).unwrap();
    let handle2 = setup_node(&host2, &host1).unwrap();

    // Clone stuff for the threads
    let handle1_clone = handle1.clone();
    let handle2_clone = handle2.clone();
    let host1_clone = host1.clone();
    let host2_clone = host2.clone();

    // Wait for links to start
    thread::sleep(time::Duration::from_millis(10000));
    test_link_host_list(handle1)?;
    test_link_host_list(handle2)?;

    let mut thread_handles = vec![];
    thread_handles.push(spawn(move || recv_stuff(handle1_clone, host1_clone)));
    thread_handles.push(spawn(move || recv_stuff(handle2_clone, host2_clone)));

    send_messages(handle1, false)?;
    send_messages(handle2, false)?;
    thread::sleep(time::Duration::from_millis(3000));

    set_crypto(handle1)?;
    set_crypto(handle2)?;

    set_compress(handle1)?;
    set_compress(handle2)?;

    thread::sleep(time::Duration::from_millis(3000));

    send_messages(handle1, true)?;
    send_messages(handle2, true)?;

    // Wait for recv threads to finish
    for handle in thread_handles {
        match handle.join() {
            Err(error) => println!("thread join error: {:?}", error),
            Ok(_) => { }
        }
    }

    // Try somee statuses
    match knet::handle_get_stats(handle1) {
	Ok(s) => println!("handle stats: {}", s),
	Err(e) => {
	    println!("handle_get_stats failed: {:?}", e);
	    return Err(e);
	}
    }
    match knet::link_get_status(handle1, &host2, 0) {
	Ok(s) => println!("link status: {}", s),
	Err(e) => {
	    println!("link_get_status failed: {:?}", e);
	    return Err(e);
	}
    }

    close_handle(handle1, 2)?;
    close_handle(handle2, 1)?;

    // Sleep to see if log thread dies
    thread::sleep(time::Duration::from_millis(3000));
    Ok(())
}
