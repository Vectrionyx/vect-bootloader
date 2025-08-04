use x86_64::VirtAddr;

#[repr(C)]
#[derive(Debug)]
struct Elf64Header {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug)]
struct Elf64ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

// Segment types
const PT_LOAD: u32 = 1;

pub fn parse_elf(kernel_image: &[u8]) -> VirtAddr {
    let header = unsafe {
        &*(kernel_image.as_ptr() as *const Elf64Header)
    };

    for i in 0..header.e_phnum {
        let ph_addr = kernel_image.as_ptr()
            .wrapping_add(header.e_phoff as usize + (i as usize * header.e_phentsize as usize));

        let ph = unsafe {&*(ph_addr as *const Elf64ProgramHeader)};
        if ph.p_type == PT_LOAD {
            let dest = ph.p_vaddr as *mut u8;
            let src = kernel_image.as_ptr().wrapping_add(ph.p_offset as usize);

            unsafe {
                core::ptr::copy_nonoverlapping(src, dest, ph.p_filesz as usize);

                if ph.p_memsz > ph.p_filesz {
                    core::ptr::write_bytes(dest.add(ph.p_filesz as usize), 0,
                                           (ph.p_memsz - ph.p_filesz) as usize);
                }
            }
        }
    };

    VirtAddr::new(header.e_entry)
}
