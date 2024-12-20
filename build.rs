#![deny(warnings, clippy::all)]

use std::{env, fs};

fn main() {
    // Setup pkg-config if cross-compiling
    let host_triple = env::var("HOST").unwrap();
    let target_triple = env::var("TARGET").unwrap();
    if host_triple != target_triple {
        // NOTE: only setup for aarch64-unknown-linux-gnu atm
        if target_triple.as_str() == "aarch64-unknown-linux-gnu" {
            env::set_var(
                "PKG_CONFIG_PATH_aarch64-unknown-linux-gnu",
                "/usr/lib/aarch64-linux-gnu/pkgconfig",
            )
        }
    }

    let mut config = autotools::Config::new("vendor/babeltrace");
    //config.arg("--verbose");
    config.reconf("-vif");
    config.enable("built-in-plugins", None);
    config.enable("silent-rules", None);
    config.disable("debug-info", None);
    config.disable("man-pages", None);
    config.disable("glibtest", None);
    config.disable("doxygen-doc", None);
    config.disable("doxygen-html", None);
    config.disable("maintainer-mode", None);
    config.disable("dependency-tracking", None);
    config.disable_shared();
    config.enable_static();
//    println!("building");
    config.fast_build(true);
  //  println!("built");
    if cfg!(debug_assertions) {
        config.enable("asan", None);
        config.env("BABELTRACE_DEV_MODE", "1");
        config.env("BABELTRACE_DEBUG_MODE", "1");
        config.env("BABELTRACE_MINIMAL_LOG_LEVEL", "INFO");
    } else {
        config.disable("asan", None);
    }

    let babeltrace_path = config.build();

    let glib2 = pkg_config::Config::new()
        .atleast_version("2.0.0")
        .statik(true)
        .probe("glib-2.0")
        .expect("Failed to find glib-2.0 pkg-config");

    let gmod2 = pkg_config::Config::new()
        .atleast_version("2.0.0")
        .statik(true)
        .probe("gmodule-2.0")
        .expect("Failed to find gmodule-2.0 pkg-config");

    let pcre = pkg_config::Config::new()
        .statik(true)
        .probe("libpcre")
        .expect("Failed to find libpcre pkg-config");

    //println!("cargo:rustc-link-lib=static={}", "/usr/lib/x86_64-linux-gnu/libm-2.36.a");
    //println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");

    println!(
        "cargo:rustc-link-search=native={}/lib",
        babeltrace_path.display()
    );
    println!("cargo:rustc-link-lib=static=babeltrace2");
    println!("cargo:rustc-link-lib=static=babeltrace2-ctf-writer");

    let plugin_path = babeltrace_path.join("lib/babeltrace2/plugins");
    fs::copy(
        plugin_path.join("babeltrace-plugin-utils.a"),
        plugin_path.join("libbabeltrace-plugin-utils.a"),
    )
    .unwrap();
    fs::copy(
        plugin_path.join("babeltrace-plugin-ctf.a"),
        plugin_path.join("libbabeltrace-plugin-ctf.a"),
    )
    .unwrap();
    println!(
        "cargo:rustc-link-search=native={}/lib/babeltrace2/plugins/",
        babeltrace_path.display()
    );
    println!("cargo:rustc-link-lib=static=babeltrace-plugin-utils");
    println!("cargo:rustc-link-lib=static=babeltrace-plugin-ctf");

    println!(
        "cargo:rustc-link-search=native={}",
        gmod2.link_paths[0].display()
    );
    println!("cargo:rustc-link-lib=dylib={}", gmod2.libs[0]);
    println!(
        "cargo:rustc-link-search=native={}",
        glib2.link_paths[0].display()
    );
    println!("cargo:rustc-link-lib=gmodule-2.0");
    println!("cargo:rustc-link-lib=glib-2.0");
    println!(
        "cargo:rustc-link-search=native={}",
        pcre.link_paths[0].display()
    );
    println!("cargo:rustc-link-lib=dylib={}", pcre.libs[0]);

    println!("cargo:rustc-link-lib=dylib=c");
    println!("cargo:rustc-link-lib=dylib=m");
}
