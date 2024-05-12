mod apic;
mod barrier;
mod consts;
mod context;
mod gdt;
mod idt;
mod interrupt;
#[cfg(feature = "kcontext")]
mod kcontext;
mod multiboot;
mod page_table;
mod sigtrx;
mod time;
mod uart;

use ::multiboot::information::MemoryType;
use alloc::vec::Vec;
pub use consts::*;
pub use context::TrapFrame;
pub use interrupt::*;
#[cfg(feature = "kcontext")]
pub use kcontext::{context_switch, context_switch_pt, read_current_tp, KContext};
pub use multiboot::kernel_page_table;
use raw_cpuid::CpuId;
pub use uart::*;

use x86_64::{
    instructions::port::PortWriteOnly,
    registers::{
        control::{Cr4, Cr4Flags},
        xcontrol::{XCr0, XCr0Flags},
    },
};

use crate::imp::{
    current_arch::multiboot::use_multiboot, CPU_NUM,
    DTB_BIN, MEM_AREA,
};
use crate::MultiCore;
use super::once::LazyInit;

#[percpu::def_percpu]
static CPU_ID: usize = 1;

pub fn shutdown() -> ! {
    unsafe { PortWriteOnly::new(0x604).write(0x2000u16) };
    loop {}
}

static MBOOT_PTR: LazyInit<usize> = LazyInit::new();

fn rust_tmp_main(magic: usize, mboot_ptr: usize) {
    crate::clear_bss();
    idt::init();
    apic::init();
    sigtrx::init();
    // Init allocator
    percpu::init(1);
    percpu::set_local_thread_pointer(0);
    gdt::init();
    interrupt::init_syscall();
    time::init_early();

    // enable avx extend instruction set and sse if support avx
    // TIPS: QEMU not support avx, so we can't enable avx here
    // IF you want to use avx in the qemu, you can use -cpu IvyBridge-v2 to
    // select a cpu with avx support
    CpuId::new().get_feature_info().map(|features| {
        info!("is there a avx feature: {}", features.has_avx());
        info!("is there a xsave feature: {}", features.has_xsave());
        info!("cr4 has OSXSAVE feature: {:?}", Cr4::read());
        if features.has_avx() && features.has_xsave() && Cr4::read().contains(Cr4Flags::OSXSAVE) {
            unsafe {
                XCr0::write(XCr0::read() | XCr0Flags::AVX | XCr0Flags::SSE | XCr0Flags::X87);
            }
        }
    });

    // TODO: This is will be fixed with ACPI support
    CPU_NUM.init_by(1);

    info!(
        "TEST CPU ID: {}  ptr: {:#x}",
        CPU_ID.read_current(),
        unsafe { CPU_ID.current_ptr() } as usize
    );
    CPU_ID.write_current(345);
    info!(
        "TEST CPU ID: {}  ptr: {:#x}",
        CPU_ID.read_current(),
        unsafe { CPU_ID.current_ptr() } as usize
    );

    info!("magic: {:#x}, mboot_ptr: {:#x}", magic, mboot_ptr);

    MBOOT_PTR.init_by(mboot_ptr);

    unsafe { crate::_main_for_arch(0) };

    shutdown()
}

pub fn arch_init() {
    DTB_BIN.init_by(Vec::new());
    if let Some(mboot) = use_multiboot(*MBOOT_PTR as _) {
        mboot
            .boot_loader_name()
            .inspect(|x| info!("bootloader: {}", x));
        mboot
            .command_line()
            .inspect(|x| info!("command_line: {}", x));
        let mut mem_area = Vec::new();
        if mboot.has_memory_map() {
            mboot
                .memory_regions()
                .unwrap()
                .filter(|x| x.memory_type() == MemoryType::Available)
                .for_each(|x| {
                    let start = x.base_address() as usize | VIRT_ADDR_START;
                    let size = x.length() as usize;
                    // ArchInterface::add_memory_region(start, end);
                    mem_area.push((start, size));
                });
        }
        MEM_AREA.init_by(mem_area);
    }
}

pub fn hart_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

#[cfg(feature = "multicore")]
impl MultiCore {
    pub fn boot_all() {}
}
