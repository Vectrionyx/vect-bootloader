use uefi::boot::{MemoryDescriptor, MemoryType};
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB};
use vect_bootapi::{MemoryRegion, MemoryRegionType};

pub struct BootFrameAllocator {
    memory_map: &'static [MemoryRegion],
    next: usize,
    min_address: Option<PhysAddr>,
    max_address: Option<PhysAddr>,
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let mut remaining_frames = self.next as u64;
        for region in self.memory_map.iter() {
            if !self.in_range(region) {
                continue;
            }
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
    pub unsafe fn new(memory_map: &'static [MemoryDescriptor]) -> Self {
        static mut MEMORY_REGIONS: [MemoryRegion; 256] = [MemoryRegion {
            start: PhysAddr::new(0),
            end: PhysAddr::new(0),
            kind: MemoryRegionType::Unknown
        }; 256];
        let mut next = 0;

        for desc in memory_map {
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
            min_address: None,
            max_address: None,
        }
    }

    pub fn restrict_range(&mut self, start: PhysAddr, end: PhysAddr) {
        self.min_address = Some(start);
        self.max_address = Some(end);
    }

    pub fn unrestrict(&mut self) {
        self.min_address = None;
        self.max_address = None;
    }

    fn in_range(&self, region: &MemoryRegion) -> bool {
        if let Some(min) = self.min_address {
            if region.end <= min {
                return false;
            }
        }
        if let Some(max) = self.max_address {
            if region.start >= max {
                return false;
            }
        }
        true
    }
}