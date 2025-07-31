#![no_std]
use x86_64::{PhysAddr, VirtAddr};

#[repr(C)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub end: PhysAddr,
    pub kind: MemoryRegionType
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryRegionType {
    Usable = 1,
    Reserved = 2,
    ACPI = 3,
    Bad = 4,
    Unknown = 0
}

#[repr(C)]
pub struct FramebufferInfo {
    pub address: PhysAddr,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: PixelFormat,
    pub bytes_per_pixel: usize,
    pub byte_len: usize,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    RGB,
    BGR,
    Undef,
}

#[repr(C)]
pub struct BootInfo {
    pub memory_map: &'static [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
    pub kernel_entry: VirtAddr,
}

#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn _start(boot_info: &'static mut BootInfo) -> ! {
            $path(boot_info)
        }
    }
}