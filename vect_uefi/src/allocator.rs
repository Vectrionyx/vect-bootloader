use uefi::boot::MemoryType;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB};
use vect_bootapi::{MemoryRegion, MemoryRegionType};

pub struct BootFrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let mut remaining_frames = self.next as u64;
        for region in self.memory_map.iter() {
            let frame_count = (region.end - region.start) / Size4KiB::SIZE;
            if frame_count > remaining_frames {
                let addr = PhysAddr::new((remaining_frames * Size4KiB::SIZE) + region.start.as_u64());
                self.next += 1;
                return Some(PhysFrame::from_start_address(addr).unwrap());
            } else {
                remaining_frames -= frame_count;
            }
        }
        
        None
    }
}

impl BootFrameAllocator {
    pub unsafe fn new(memory_map: &dyn uefi::mem::memory_map::MemoryMap) -> Self {
        static mut MEMORY_REGIONS: [MemoryRegion; 256] = [MemoryRegion {
            start: PhysAddr::new(0),
            end: PhysAddr::new(0),
            kind: MemoryRegionType::Unknown
        }; 256];
        let mut next = 0;

        for desc in memory_map.entries() {
            if desc.ty == MemoryType::CONVENTIONAL {
                unsafe {
                    MEMORY_REGIONS[next] = MemoryRegion {
                        start: PhysAddr::new(desc.phys_start),
                        end: PhysAddr::new(desc.phys_start + desc.page_count * 4096),
                        kind: MemoryRegionType::Usable
                    }
                }
                next += 1;
            }
        }

        Self {
            memory_map: &MEMORY_REGIONS[..next],
            next: 0,
        }
    }
}