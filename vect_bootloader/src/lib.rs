use anyhow::Result;
use fatfs::{FileSystem, FormatVolumeOptions, FsOptions};
use std::fs::File;
use std::io::Write;
use std::path::{ Path, PathBuf };

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
            kernel_path,
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
        let buf = File::options().read(true).write(true).open(img_path)?;

        // 2. format as FAT
        let format_opts = FormatVolumeOptions::new();
        fatfs::FileSystem::new(&buf, FsOptions::new())?;
        let fs = FileSystem::new(buf, FsOptions::new())?;
        let root_dir = fs.root_dir();

        // 3.Copy bootloader binary

        Ok(())
    }
}