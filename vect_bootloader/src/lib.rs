use anyhow::Result;
use fatfs::{format_volume, FileSystem, FormatVolumeOptions, FsOptions};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{ Path, PathBuf };
use glob::glob;

pub struct UefiBoot {
    kernel_path: PathBuf,
    config: Option<BootConfig>
}

pub struct BootConfig {
    pub image_size_mb: u64, // will likely get removed sooner or later. I might use BootConfig to allow for feature flagging memory or other things
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            image_size_mb: 64
        }
    }
}

impl UefiBoot {
    pub fn new(kernel_path: impl Into<PathBuf>) -> Self {
        Self {
            kernel_path: kernel_path.into(),
            config: None
        }
    }

    pub fn with_config(mut self, config: BootConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn create_disk_image(self, output_path: impl AsRef<Path>) -> Result<()> {
        let config = self.config.unwrap_or_default();

        // 1. Create output file
        let img_path = output_path.as_ref();
        let file = File::create(img_path)?;
        file.set_len(config.image_size_mb * 1024 * 1024)?;
        let mut buf = File::options().read(true).write(true).open(img_path)?;
        buf.seek(SeekFrom::Start(0x1FE))?;
        buf.write_all(&[0x55, 0xAA])?;
        buf.seek(SeekFrom::Start(0x0))?;

        // 2. format as FAT
        let format_opts = FormatVolumeOptions::new()
            .bytes_per_sector(512);
        format_volume(&buf, format_opts)?;
        let fs = FileSystem::new(buf, FsOptions::new())?;
        let root_dir = fs.root_dir();

        // 3.Copy bootloader binary
        let vect_uefi_file = UefiBoot::locate_vect_efi();
        let bootloader_efi = Path::new(&vect_uefi_file);
        let efi_dir = root_dir.create_dir("EFI")?;
        let boot_dir = efi_dir.create_dir("BOOT")?;
        let mut boot_file = boot_dir.create_file("BOOTX64.EFI")?;
        std::io::copy(&mut File::open(bootloader_efi)?, &mut boot_file)?;

        // 4. Copy kernel (optional for now)
        let mut kernel_file = root_dir.create_file("KERNEL.ELF")?;
        std::io::copy(&mut File::open(self.kernel_path)?, &mut kernel_file)?;

        println!("Created UEFI boot image at {:?}", img_path);

        Ok(())
    }

    fn locate_vect_efi() -> PathBuf {
        let profile = std::env::var("PROFILE").expect("Expected a PROFILE env var");
        let pattern = format!("target/x86_64-unknown-uefi/{}/deps/artifact/vect_uefi-*/bin/vect_uefi-*.efi", profile);
        let efi_file: PathBuf;

        for entry in glob(&pattern) {
            match entry {
                Ok(path) => {
                    efi_file = PathBuf::from(path);
                    break;
                },
                Err(e) => {
                    println!("Failed to read glob pattern: {}", e);
                    panic!("Failed to read glob pattern");
                }
            }
        }

        efi_file
    }
}