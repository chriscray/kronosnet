fn main() {

    // Tell the compiler to use the build-tree libs & headers for compiling
    println!("cargo:rustc-link-search=native=../../../.libs/");
    println!("cargo:rustc-link-lib=knet");

    cc::Build::new()
	.file("src/bin/set_plugin_path.c")
	.file("../../../tests/test-common.c") // for find_plugins_path()
	.flag("-Wno-unused-parameter")  // Needed for test-common.c to compile cleanly
	.include("../../..")            // for internals.h
	.include("../../../..")         // for config.h
	.include("../../../tests")      // for test-common.h
	.compile("set_plugin_path");
}
