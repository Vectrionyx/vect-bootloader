fn main() {
    let efi_path = std::env::var("CARGO_BIN_EXE_vect_uefi").expect("vect_uefi path not found");
    let out_dir = std::env::var("OUT_DIR").expect("out dir not found");

    std::fs::write(format!("{}/efi_path.txt", out_dir), efi_path)
        .expect("Unable to write to file");
}