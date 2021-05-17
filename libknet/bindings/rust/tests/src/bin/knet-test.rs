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

fn setup_node(our_hostid: &knet::HostId, other_hostid: &knet::HostId,
	      name: &str) -> Result<knet::Handle>
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

    if let Err(e) = knet::host_add(knet_handle, &other_hostid) {
	println!("Error from host_add: {}", e);
	return Err(e);
    }
    if let Err(e) = knet::link_set_config(knet_handle, &other_hostid, 0,
				knet::TransportId::Udp,
				&SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000+(our_hostid.to_u16())),
				&SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000+(other_hostid.to_u16())),
				knet::LinkFlags::NONE) {
	println!("Error from link_set_config: {}", e);
	return Err(e);
    }
    if let Err(e) = knet::host_set_name(knet_handle, &other_hostid, name) {
	println!("Error from host_set_name: {}", e);
	return Err(e);
    }

    if let Err(e) = knet::handle_enable_sock_notify(knet_handle, our_hostid.to_u16() as u64, Some(sock_notify_fn)) {
	println!("Error from handle_enable_sock_notify: {}", e);
	return Err(e);
    }

    if let Err(e) = knet::link_enable_status_change_notify(knet_handle, our_hostid.to_u16() as u64, Some(link_notify_fn)) {
	println!("Error from handle_enable_link_notify: {}", e);
	return Err(e);
    }
    if let Err(e) = knet::host_enable_status_change_notify(knet_handle, our_hostid.to_u16() as u64, Some(host_notify_fn)) {
	println!("Error from handle_enable_host_notify: {}", e);
	return Err(e);
    }
    if let Err(e) = knet::handle_enable_filter(knet_handle, our_hostid.to_u16() as u64, Some(filter_fn)) {
	println!("Error from handle_enable_filter: {}", e);
	return Err(e);
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

    if let Err(e) = knet::handle_crypto_rx_clear_traffic(knet_handle, knet::RxClearTraffic::Allow) {
	println!("Error from handle_crypto_rx_clear_traffic: {}", e);
	return Err(e);
    }

    if let Err(e) = knet::link_set_enable(knet_handle, &other_hostid, 0, true) {
	println!("Error from set_link_enable(true): {}", e);
	return Err(e);
    }

    if let Err(e) = knet::link_set_ping_timers(knet_handle, &other_hostid, 0,
					       500, 1000, 1024) {
	println!("Error from set_link_ping_timers: {}", e);
	return Err(e);
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

    if let Err(e) = knet::handle_setfwd(knet_handle, true) {
	println!("Error from setfwd(true): {}", e);
	return Err(e);
    }

    // Check status
    let data_fd =
    match knet::handle_get_datafd(knet_handle, CHANNEL) {
	Ok(f) => {
	    println!("got datafd {} for channel", f);
	    f
	}
	Err(e) => {
	    println!("Error from handle_get_datafd: {}", e);
	    return Err(e);
	}
    };

    match knet::handle_get_channel(knet_handle, data_fd) {
	Ok(c) =>
	    if c != CHANNEL {
		println!("handle_get_channel returned wrong channel ID: {}, {}",c, CHANNEL);
		return Err(Error::new(ErrorKind::Other, "Error in handle_get_channel"));
	    }
	Err(e) => {
	    println!("Error from handle_get_channel: {}", e);
	    return Err(e);
	}
    }



    match knet::link_get_enable(knet_handle, other_hostid, 0) {
	Ok(b) => if !b {
	    println!("link not enabled (according to link_get_enable()");
	},
	Err(e) => {
	    println!("Error from link_get_enable: {}", e);
	    return Err(e);
	}
    }



    Ok(knet_handle)
}

fn recv_stuff(handle: knet::Handle, host: knet::HostId) -> Result<()>
{
    let buf = [0u8; 1024];

    loop {
	match knet::recv(handle, &buf, CHANNEL) {
	    Ok(len) => {
		let recv_len = len as usize;
		if recv_len == 0 {
		    break; // EOF??
		} else {
		    let s = String::from_utf8(buf[0..recv_len].to_vec()).unwrap();
		    println!("recvd on {}: {} {:?}  {} ", host, recv_len, &buf[0..recv_len], s);
		    if s == *"QUIT" {
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

    if let Err(e) = knet::handle_setfwd(handle, false) {
	println!("Error from setfwd 1 (false): {}", e);
	return Err(e);
    }

    if let Err(e) = knet::link_set_enable(handle, &other_hostid, 0, false) {
	println!("Error from set_link_enable(false): {}", e);
	return Err(e);
    }

    if let Err(e) =knet::link_clear_config(handle, &other_hostid, 0) {
	println!("clear config failed: {}", e);
	return Err(e);
    }

    if let Err(e) = knet::host_remove(handle, &other_hostid) {
	println!("host remove failed: {}", e);
	return Err(e);
    }

    if let Err(e) = knet::handle_free(handle) {
	println!("handle_free failed: {}", e);
	return Err(e);
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
    if let Err(e) = knet::handle_compress(handle, &compress_config) {
	println!("Error from handle_compress: {}", e);
	Err(e)
    } else {
	Ok(())
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

    if let Err(e) = knet::handle_crypto_set_config(handle, &crypto_config, 1) {
	println!("Error from handle_crypto_set_config: {}", e);
	return Err(e);
    }

    if let Err(e) = knet::handle_crypto_use_config(handle, 1) {
	println!("Error from handle_crypto_use_config: {}", e);
	return Err(e);
    }

    if let Err(e) = knet::handle_crypto_rx_clear_traffic(handle, knet::RxClearTraffic::Disallow) {
	println!("Error from handle_crypto_rx_clear_traffic: {}", e);
	return Err(e);
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

    // if let Err(e) = knet::handle_enable_filter(handle, 0, None) {
    // 	println!("Error from handle_enable_filter (disable): {}", e);
    // 	return Err(e);
    // }

    // let s = String::from("SYNC TEST").into_bytes();
    // if let Err(e) = knet::send_sync(handle, &s, CHANNEL) {
    //  	println!("send_sync failed: {}", e);
    //  	return Err(e);
    // }

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
		println!();
	    }
	}
	Err(e) => {
	    println!("link_get_host_list failed: {}", e);
	    return Err(e);
	}
    }
    Ok(())
}

// Try some metadata calls
fn test_metadata_calls(handle: knet::Handle, host: &knet::HostId) ->Result<()>
{
    if let Err(e) = knet::handle_set_threads_timer_res(handle, 190000) {
	println!("knet_handle_set_threads_timer_res failed: {:?}", e);
	return Err(e);
    }
    match knet::handle_get_threads_timer_res(handle) {
	Ok(v) => {
	    if v != 190000 {
		println!("knet_handle_get_threads_timer_res returned wrong value {}", v);
	    }
	},
	Err(e) => {
	    println!("knet_handle_set_threads_timer_res failed: {:?}", e);
	    return Err(e);
	}
    }

    if let Err(e) = knet::handle_pmtud_set(handle, 1000) {
	println!("knet_handle_pmtud_set failed: {:?}", e);
	return Err(e);
    }
    match knet::handle_pmtud_get(handle) {
	Ok(v) => {
	    if v != 1000 {
		println!("knet_handle_pmtud_get returned wrong value {}", v);
	    }
	},
	Err(e) => {
	    println!("knet_handle_pmtud_get failed: {:?}", e);
	    return Err(e);
	}
    }

    if let Err(e) = knet::handle_pmtud_setfreq(handle, 1000) {
	println!("knet_handle_pmtud_setfreq failed: {:?}", e);
	return Err(e);
    }
    match knet::handle_pmtud_getfreq(handle) {
	Ok(v) => {
	    if v != 1000 {
		println!("knet_handle_pmtud_getfreq returned wrong value {}", v);
	    }
	},
	Err(e) => {
	    println!("knet_handle_pmtud_getfreq failed: {:?}", e);
	    return Err(e);
	}
    }

    if let Err(e) = knet::handle_set_transport_reconnect_interval(handle, 100) {
	println!("knet_handle_set_transport_reconnect_interval failed: {:?}", e);
	return Err(e);
    }
    match knet::handle_get_transport_reconnect_interval(handle) {
	Ok(v) => {
	    if v != 100 {
		println!("knet_handle_get_transport_reconnect_interval {}", v);
	    }
	},
	Err(e) => {
	    println!("knet_handle_get_transport_reconnect_interval failed: {:?}", e);
	    return Err(e);
	}
    }


    if let Err(e) = knet::link_set_pong_count(handle, host, 0, 4) {
	println!("knet_link_set_pong_count failed: {:?}", e);
	return Err(e);
    }
    match knet::link_get_pong_count(handle, host, 0) {
	Ok(v) => {
	    if v != 4 {
		println!("knet_link_get_pong_count returned wrong value {}", v);
	    }
	},
	Err(e) => {
	    println!("knet_link_get_pong_count failed: {:?}", e);
	    return Err(e);
	}
    }

    if let Err(e) = knet::host_set_policy(handle, host, knet::LinkPolicy::Active) {
	println!("knet_host_set_policy failed: {:?}", e);
	return Err(e);
    }
    match knet::host_get_policy(handle, host) {
	Ok(v) => {
	    if v != knet::LinkPolicy::Active {
		println!("knet_host_get_policy returned wrong value {}", v);
	    }
	},
	Err(e) => {
	    println!("knet_host_get_policy failed: {:?}", e);
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
	    println!();
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
	    println!();
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
	    println!();
	}
	Err(e) => {
	    println!("link_get_transport_list failed: {:?}", e);
	    return Err(e);
	}
    }
    let host1 = knet::HostId::new(1);
    let host2 = knet::HostId::new(2);

    // Now test traffic
    let handle1 = setup_node(&host1, &host2, "host2")?;
    let handle2 = setup_node(&host2, &host1, "host1")?;

    // Copy stuff for the threads
    let handle1_clone = handle1;
    let handle2_clone = handle2;
    let host1_clone = host1;
    let host2_clone = host2;

    // Wait for links to start
    thread::sleep(time::Duration::from_millis(10000));
    test_link_host_list(handle1)?;
    test_link_host_list(handle2)?;

    // Start recv threads for each handle
    let thread_handles = vec![
	spawn(move || recv_stuff(handle1_clone, host1_clone)),
	spawn(move || recv_stuff(handle2_clone, host2_clone))];

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
        if let Err(error) = handle.join() {
            println!("thread join error: {:?}", error);
	}
    }

    // Try some statses
    match knet::handle_get_stats(handle1) {
	Ok(s) => println!("handle stats: {}", s),
	Err(e) => {
	    println!("handle_get_stats failed: {:?}", e);
	    return Err(e);
	}
    }
    match knet::host_get_status(handle1, &host2) {
	Ok(s) => println!("host status: {}", s),
	Err(e) => {
	    println!("host_get_status failed: {:?}", e);
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

    test_metadata_calls(handle1, &knet::HostId::new(2))?;

    close_handle(handle1, 2)?;
    close_handle(handle2, 1)?;

    // Sleep to see if log thread dies
    thread::sleep(time::Duration::from_millis(3000));
    Ok(())
}
