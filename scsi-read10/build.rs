fn main() {
    if cfg!(target_os = "linux") {
        linux_bindings();
    }
}

#[cfg(target_os = "linux")]
fn linux_bindings() {
    println!("cargo:rerun-if-changed=/usr/include/scsi/sg.h");

    let bindings = bindgen::Builder::default()
        .header("/usr/include/scsi/sg.h")
        .derive_default(true)
        .allowlist_type("sg_io_hdr")
        .allowlist_var("SG_IO")
        .allowlist_var("SG_DXFER_.*")
        .generate()
        .expect("bindgen failed to generate SCSI bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    bindings
        .write_to_file(out_path.join("sg_bindings.rs"))
        .expect("failed to write SCSI bindings");
}

#[cfg(not(target_os = "linux"))]
fn linux_bindings() {
    return;
}
