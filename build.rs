// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=exports/xgameruntime.def");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_os == "windows" && target_env == "msvc" {
        let manifest_dir = PathBuf::from(
            env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by Cargo"),
        );
        let def_file = manifest_dir.join("exports").join("xgameruntime.def");
        println!("cargo:rustc-link-arg-cdylib=/DEF:{}", def_file.display());
    }
}
