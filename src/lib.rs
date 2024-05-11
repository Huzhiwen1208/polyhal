#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(stdsimd)]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts_mut)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![cfg_attr(target_arch = "aarch64", feature(const_option))]

extern crate alloc;

#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

#[macro_use]
mod utils;

cfg_if! {
    if #[cfg(feature = "libos")] {
        #[path = "libos/mod.rs"]
        mod imp;
    } else {
        #[path = "bare/mod.rs"]
        mod imp;
        use imp::api::*;
        use imp::pagetable::*;
        use imp::consts::*;
        use imp::multicore::MultiCore;
        use imp::*;
        use imp::current_arch::*;
        use imp::debug::*;
        use imp::time::*;

        pub use imp::{
            addr::*, 
            page::PageAlloc, 
            pagetable::{PageTable, PageTableWrapper, MappingFlags, MappingSize}
        };
        pub use imp::{
            get_mem_areas, init,
            TrapFrameArgs, TrapType, PAGE_SIZE,
            time::*,
            debug::DebugConsole,
            current_arch::{
                run_user_task, shutdown, kernel_page_table,
                TrapFrame, VIRT_ADDR_START,
            },
        };
        pub use polyhal_macro::{arch_entry, arch_interrupt};
        
        #[cfg(feature = "kcontext")]
        pub use imp::{KContextArgs, current_arch::{KContext, read_current_tp, context_switch_pt, context_switch}};
    }
}
