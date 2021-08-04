use std::{env, error::Error, path::PathBuf};

const HEADER: &str = "../arrow/cpp/src/arrow/c/abi.h";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={}", HEADER);

    bindgen::Builder::default()
        .header(HEADER)
        .allowlist_type("Arrow.*")
        .allowlist_var("ARROW_FLAG.*")
        .no_copy("ArrowSchema")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap()
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))?;

    Ok(())
}
