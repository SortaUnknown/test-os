use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use alloc::alloc::{GlobalAlloc, Layout};
use alloc::string::String;
use alloc::format;
use crate::allocator::ALLOCATOR;
use crate::syscall::*;
use xmas_elf::sections::SectionData;
use iced_x86::{Decoder, DecoderOptions, NasmFormatter, Formatter, Instruction};
use log::info;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[repr(C)]
pub struct SysCallTable
{
    sys_file_read_perms: extern "C" fn(*const u8, usize) -> CVecShort,
    sys_file_write_perms: extern "C" fn(bool, *const u8, usize, *const u8, usize) -> i8,
    sys_file_read: extern "C" fn(bool, *const u8, usize) -> CVecShort,
    sys_file_write: extern "C" fn(bool, *const u8, usize, *const u8, usize) -> i8,
    sys_file_delete: extern "C" fn(bool, *const u8, usize) -> i8,
    sys_file_create: extern "C" fn(*const u8, usize) -> i8,
    sys_time_now: extern "C" fn() -> i64,
    sys_rand_buffer: extern "C" fn(*mut u8, usize)
}

impl SysCallTable
{
    pub const fn gen() -> Self
    {
        SysCallTable{sys_file_read_perms, sys_file_write_perms, sys_file_read, sys_file_write, sys_file_delete, sys_file_create, sys_time_now, sys_rand_buffer}
    }
}

type StartFunc = extern "C" fn (SysCallTable) -> !;

pub struct Process
{
    code: Vec<u8>,
    start: StartFunc,
    pid: u64,
    log: bool,
    privileged: bool
}

impl Process
{
    pub fn spawn(data: &[u8], log: bool) -> Self
    {
        let elf = xmas_elf::ElfFile::new(data).expect("invalid ELF code");
        let mut min = u64::MAX;
        let mut max = 0u64;
        for header in elf.program_iter()
        {
            let size = header.mem_size();
            if size > 0
            {
                let addr = header.virtual_addr();
                min = min.min(addr);
                max = max.max(addr + size);
            }
        }
        let cap = max as usize;
        let ptr = unsafe{ALLOCATOR.alloc(Layout::from_size_align_unchecked(cap, 4096))};
        let mut data_vec = unsafe{Vec::from_raw_parts(ptr, 0, cap)};
        for _ in min..max
        {
            data_vec.push(0);
        }
        for header in elf.program_iter()
        {
            if let Ok(xmas_elf::program::Type::Load) = header.get_type()
            {
                let mut b = 0;
                let t = (header.virtual_addr() - min) as usize;
                let os = header.offset() as usize;
                for e in os..os + header.file_size() as usize
                {
                    data_vec[b + t] = data[e];
                    b += 1;
                }
                if let Some(rela) = elf.find_section_by_name(".rela.dyn")
                {
                    if let Ok(SectionData::Rela64(arr)) = rela.get_data(&elf)
                    {
                        for r in &arr[0..]
                        {
                            let o = r.get_offset();
                            let a = r.get_addend();
                            //let t = r.get_type();
                            if o >= header.virtual_addr() && o < header.virtual_addr() + header.mem_size()
                            {
                                let go = data_vec.as_ptr() as u64;
                                let p64 = unsafe{data_vec.as_ptr().offset(o as isize)} as *mut u64;
                                unsafe{p64.write(a + go)};
                            }
                        }
                    }
                }
            }
        }
        let funcd = unsafe{data_vec.as_mut_ptr().offset((elf.header.pt2.entry_point() - min) as isize)};
        let func: StartFunc = unsafe{core::intrinsics::transmute(funcd)};

        if log
        {
            let entry = elf.header.pt2.entry_point() - min;
            let mut code = Vec::new();
            for i in entry.try_into().unwrap()..data_vec.len()
            {
                code.push(data_vec[i]);
            }
            let hcbl = 8;
            let mut decoder = Decoder::with_ip(64, &code, entry, DecoderOptions::NONE);
            let mut formatter = NasmFormatter::new();
            formatter.options_mut().set_digit_separator("_");
            formatter.options_mut().set_first_operand_char_index(10);
            let mut output = String::new();
            let mut inst = Instruction::new();
            while decoder.can_decode()
            {
                decoder.decode_out(&mut inst);
                info!("{:?}", inst);
                output.clear();
                formatter.format(&inst, &mut output);
                let mut strbuild = String::new();
                strbuild = format!("{}{:016X} ", strbuild, inst.ip());
                let start = inst.ip() - entry;
                let mut instr_bytes: Vec<u8> = Vec::new();
                for i in start..start + inst.len() as u64
                {
                    instr_bytes.push(code[i as usize]);
                }
                if instr_bytes.len() < hcbl
                {
                    for _ in 0..hcbl - instr_bytes.len()
                    {
                        strbuild = format!("{} {}", strbuild, output);
                    }
                }
            }
        }


        let id = NEXT_ID.load(Ordering::Relaxed);
        NEXT_ID.fetch_add(id + 1, Ordering::Relaxed);
        Process{code: data_vec, start: func, pid: id, log: true, privileged: true}
    }

    pub fn call(&self) -> !
    {
        let table = SysCallTable::gen();
        (self.start)(table)
    }
}