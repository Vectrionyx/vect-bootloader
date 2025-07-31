fn main() {
    println!("cargo:rerun-if-changed=../vect_uefi/Cargo.toml");
    println!("cargo:rerun-if-changed=../vect_uefi/src");

    let status = std::process::Command::new("cargo")
        .args([
            "build",
            "-p", "vect_uefi",
            "--target", "x86_64-unknown-uefi",
            "--release",
        ])
        .status()
        .expect("Failed to run cargo build for vect_uefi");
    assert!(status.success(), "Failed to build uefi binary");
}