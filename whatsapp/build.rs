//! Build script for the whatsapp crate.
//!
//! Compiles the Go whatsmeow wrapper into a shared library and links it.

#![allow(clippy::unwrap_used, reason = "build script panics are acceptable")]
#![allow(clippy::expect_used, reason = "build script panics are acceptable")]
#![allow(clippy::panic, reason = "build script panics are acceptable")]
#![allow(clippy::manual_assert, reason = "if-then-panic is clearer here")]
#![allow(clippy::unused_result_ok, reason = "copy failure is non-fatal")]
#![allow(clippy::absolute_paths, reason = "build script is simple")]
#![allow(clippy::let_underscore_must_use, reason = "copy failure is non-fatal")]
#![allow(clippy::let_underscore_untyped, reason = "copy failure is non-fatal")]
#![allow(let_underscore_drop, reason = "copy failure is non-fatal")]
#![allow(missing_docs, reason = "build script does not need docs")]

use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let go_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
    )
    .join("go");

    let lib_path = out_dir.join("libwhatsapp.so");

    let status = Command::new("go")
        .arg("build")
        .arg("-buildmode=c-shared")
        .arg("-o")
        .arg(&lib_path)
        .arg(".")
        .current_dir(&go_dir)
        .status()
        .expect("Failed to execute go build. Is Go installed?");

    assert!(status.success(), "Go build failed");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=dylib=whatsapp");
    println!("cargo:rerun-if-changed=go/main.go");
    println!("cargo:rerun-if-changed=go/go.mod");
    println!("cargo:rerun-if-changed=go/go.sum");

    let target_dir =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"))
            .ancestors()
            .nth(3)
            .expect("OUT_DIR path too short")
            .to_path_buf();
    let so_target = target_dir.join("libwhatsapp.so");
    drop(fs::copy(&lib_path, &so_target));

    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", target_dir.display());
    println!("cargo:rustc-link-arg=-Wl,-rpath,/tmp");
}
