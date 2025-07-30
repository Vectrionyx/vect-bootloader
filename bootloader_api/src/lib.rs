#![no_std]

use x86_64::{PhysAddr, VirtAddr};

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

pub struct FramebufferInfo {
    pub address: PhysAddr,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: PixelFormat,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    RGB,
    BGR,
    Undef,
}

pub struct BootInfo {
    pub memory_map: &'static [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
    pub kernel_entry: VirtAddr,
}