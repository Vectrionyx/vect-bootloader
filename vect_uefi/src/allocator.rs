use uefi::boot::MemoryType;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

pub struct BootFrameAllocator {
    memory_map: &'static dyn uefi::mem::memory_map::MemoryMap,
    next: usize,
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for desc in self.memory_map.entries().skip(self.next) {
            if desc.ty == MemoryType::CONVENTIONAL {
                let start = desc.phys_start;
                let frame = PhysFrame::containing_address(PhysAddr::new(start));
                self.next += 1;
                return Some(frame);
            }
        }
        
        None
    }
}

impl BootFrameAllocator {
    pub fn new(memory_map: &'static dyn uefi::mem::memory_map::MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }
}