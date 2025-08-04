#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::boot::{get_handle_for_protocol, open_protocol_exclusive, MemoryType, ScopedProtocol};
use uefi::mem::memory_map::MemoryMap;
use uefi::proto::media::file::{File};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB};
use vect_bootapi::{BootInfo, FramebufferInfo, MemoryRegion, MemoryRegionType, PixelFormat as KernelPixelFormat};
use crate::allocator::BootFrameAllocator;
use crate::elf_reader::parse_elf;

type KernelEntry = extern "C" fn(&BootInfo) -> !;

extern crate uefi;
extern crate uefi_services;
extern crate vect_bootapi;

mod allocator;
mod elf_reader;

const PHYS_MEM_OFFSET: u64 = 0xFFFF_8000_0000_0000;

const MAX_MEMORY_REGIONS: usize = 256;
static mut MEMORY_REGIONS: [MemoryRegion; MAX_MEMORY_REGIONS] =
    [MemoryRegion {
        start: PhysAddr::new(0),
        end: PhysAddr::new(0),
        kind: MemoryRegionType::Unknown,
    }; MAX_MEMORY_REGIONS];


#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = open_protocol_exclusive::<GraphicsOutput>(handle).expect("Opening graphics output");
    let fb_info = get_framebuffer(&mut gop);

    // let mmap = boot::memory_map(MemoryType::CONVENTIONAL).unwrap();
    // let mut frame_allocator = unsafe {BootFrameAllocator::new(&mmap)};
    // let mut mapper = unsafe { init_page_tables(&mut frame_allocator)};
    //
    // map_memory_from_uefi(&mut mapper, &mut frame_allocator, &mmap);
    // let mem_regions = build_memory_regions(&mmap);
    // let kernel_buf = load_kernel_image();
    // let kernel_start = parse_elf(kernel_buf);
    //
    // static mut BOOT_INFO: BootInfo = BootInfo {
    //     memory_map: &[],
    //     framebuffer: None,
    //     memory_offset: PHYS_MEM_OFFSET,
    //     kernel_entry: VirtAddr::zero()
    // };
    //
    // unsafe {
    //     BOOT_INFO.memory_map = mem_regions;
    //     BOOT_INFO.framebuffer = Some(fb_info);
    //     BOOT_INFO.kernel_entry = kernel_start;
    //
    //     let _ = boot::exit_boot_services(None);
    //
    //     let entry_fn: KernelEntry = core::mem::transmute(kernel_start.as_u64());
    //     let boot_info_ref: &BootInfo = &*(&raw const BOOT_INFO);
    //     entry_fn(boot_info_ref);
    // };

    Status::SUCCESS
}

fn get_framebuffer(gop: &mut ScopedProtocol<GraphicsOutput>) -> FramebufferInfo {
    let mode = gop.current_mode_info();
    let (width, height) = mode.resolution();
    let format = match mode.pixel_format() {
        PixelFormat::Rgb => KernelPixelFormat::RGB,
        PixelFormat::Bgr => KernelPixelFormat::BGR,
        _ => KernelPixelFormat::Undef
    };
    let stride = mode.stride();

    let mut fb = gop.frame_buffer();
    let fb_ptr = fb.as_mut_ptr();
    let fb_size = fb.size();
    FramebufferInfo {
        address: PhysAddr::new(fb_ptr as u64),
        width,
        height,
        stride,
        format,
        bytes_per_pixel: fb_size / (stride * height),
        byte_len: fb_size
    }
}

fn map_memory_from_uefi<M: Mapper<Size4KiB>, A: FrameAllocator<Size4KiB>>(
    mapper: &mut M,
    frame_allocator: &mut A,
    memory_map: &dyn MemoryMap
) {
    for descriptor in memory_map.entries() {
        let phys_start = descriptor.phys_start;
        let page_count = descriptor.page_count;
        let virt_start = PHYS_MEM_OFFSET + phys_start;
        let flags = flags_for(descriptor.ty);
        for i in 0..page_count {
            let phys_addr = PhysAddr::new(phys_start + i * 4096);
            let virt_addr = VirtAddr::new(virt_start + i * 4096);

            let page = Page::containing_address(virt_addr);
            let frame = PhysFrame::containing_address(phys_addr);

            unsafe {
                mapper.map_to(page, frame, flags, frame_allocator)
                    .expect("Failed to map memory")
                    .flush();
            }
        }
    }
}

fn flags_for(mem_type: MemoryType) -> PageTableFlags {
    match mem_type {
        MemoryType::CONVENTIONAL => PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        MemoryType::LOADER_CODE |
        MemoryType::LOADER_DATA |
        MemoryType::BOOT_SERVICES_CODE |
        MemoryType::BOOT_SERVICES_DATA |
        MemoryType::RUNTIME_SERVICES_CODE |
        MemoryType::RUNTIME_SERVICES_DATA => PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        MemoryType::ACPI_RECLAIM |
        MemoryType::ACPI_NON_VOLATILE |
        MemoryType::MMIO |
        MemoryType::MMIO_PORT_SPACE |
        MemoryType::RESERVED => PageTableFlags::PRESENT,
        _ => PageTableFlags::PRESENT,
    }
}

fn to_region_type(memory_type: MemoryType) -> MemoryRegionType {
    match memory_type {
        MemoryType::CONVENTIONAL => MemoryRegionType::Usable,
        MemoryType::ACPI_RECLAIM |
        MemoryType::ACPI_NON_VOLATILE |
        MemoryType::UNUSABLE |
        MemoryType::UNACCEPTED => MemoryRegionType::Bad,
        _ => MemoryRegionType::Reserved,
    }
}

fn build_memory_regions<'a>(
    memory_map: &dyn MemoryMap,
) -> &'a [MemoryRegion] {
    let mut count = 0;

    for desc in memory_map.entries() {
        let kind = to_region_type(desc.ty);
        unsafe {
            MEMORY_REGIONS[count] = MemoryRegion {
                start: PhysAddr::new(desc.phys_start),
                end: PhysAddr::new(desc.phys_start + desc.page_count * 4096),
                kind
            };
        }
        count += 1;
    }

    unsafe { &MEMORY_REGIONS[..count] }
}

unsafe fn init_page_tables(
    _frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> OffsetPageTable<'static> {
    let (level_4_frame, _) = Cr3::read();
    let phys_mem_offset = VirtAddr::new(PHYS_MEM_OFFSET);

    let phys_addr = level_4_frame.start_address();
    let virt_addr = phys_mem_offset + phys_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    unsafe {
        let l4_table = &mut *page_table_ptr;
        OffsetPageTable::new(l4_table, phys_mem_offset)
    }
}

fn load_kernel_image() -> &'static [u8] {
    use uefi::proto::media::fs::SimpleFileSystem;
    use uefi::proto::media::file::{FileMode, FileAttribute, FileType, FileInfo};
    use uefi::CStr16;

    let sfs_handle = get_handle_for_protocol::<SimpleFileSystem>()
        .expect("Opening SimpleFileSystem failed");
    let mut sfs = open_protocol_exclusive::<SimpleFileSystem>(sfs_handle).expect("Opening SimpleFileSystem failed");
    let mut root = sfs.open_volume().expect("Failed to open volume");

    let mut str_buff = [0u16; 64];
    let kernel_path = CStr16::from_str_with_buf("\\EFI\\KERNEL.ELF", &mut str_buff)
        .expect("Bad kernel path");

    let file_handle = root.open(kernel_path, FileMode::Read, FileAttribute::empty())
        .expect("Failed to open kernel")
        .into_type()
        .expect("Failed to convert file type");

    let mut file = match file_handle {
        FileType::Regular(f) => f,
        _ => panic!("Bad file type"),
    };

    let mut info_buf = [0u8; 512];
    let info = file.get_info::<FileInfo>(&mut info_buf)
        .expect("Failed to get file info");
    let file_size = info.file_size();
    let buf_ptr = boot::allocate_pool(MemoryType::LOADER_DATA, file_size as usize)
        .expect("Failed to allocate buffer");
    let alloc_size = ((file_size + 7) & !7) as usize;
    let kernel_buffer = unsafe {
        core::slice::from_raw_parts_mut(buf_ptr.as_ptr(), alloc_size)
    };

    let read_len = file.read(kernel_buffer).expect("Failed to read kernel");
    assert_eq!(read_len, file_size as usize);
    kernel_buffer
}
