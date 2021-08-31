extern crate bindgen;

static HEADER_MAP_STRUCTS: &str = "src/bpf/map_structs.h";

fn main() {
    println!("cargo:rerun-if-changed={}", HEADER_MAP_STRUCTS);

    let bindings = bindgen::Builder::default()
        .header(HEADER_MAP_STRUCTS)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
