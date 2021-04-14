extern crate pkg_config;

fn main() {
    if let Err(e) = pkg_config::probe_library("libknet") {
	match e {
	    pkg_config::Error::Failure { .. } => panic! (
		"Pkg-config failed - usually this is because knet development headers are not installed.\n\n\
                For Fedora users:\n# dnf install libknet1-devel\n\n\
                For Debian/Ubuntu users:\n# apt-get install libknet1-dev\n\n\
                pkg_config details:\n{}",
		e
	    ),
	    _ => panic!("{}", e)
	}
    }

    if let Err(e) = pkg_config::probe_library("libnozzle") {
	match e {
	    pkg_config::Error::Failure { .. } => panic! (
		"Pkg-config failed - usually this is because knet development headers are not installed.\n\n\
                For Fedora users:\n# dnf install libnozzle1-devel\n\n\
                For Debian/Ubuntu users:\n# apt-get install libnozzle1-dev\n\n\
                pkg_config details:\n{}",
		e
	    ),
	    _ => panic!("{}", e)
	}
    }

}
