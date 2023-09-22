use core::sync::atomic::{AtomicU64, Ordering};
use core::ffi::{c_char, c_uchar, c_schar, c_longlong, c_ulonglong};
use core::marker::FnPtr;
use alloc::vec::Vec;
use alloc::alloc::{GlobalAlloc, Layout};
use alloc::string::String;
use alloc::format;
use crate::allocator::ALLOCATOR;
use crate::fs::FilePermissions;
use crate::syscall::*;
use embedded_io::ErrorKind;
use xmas_elf::sections::SectionData;
use iced_x86::{Decoder, DecoderOptions, NasmFormatter, Formatter, Instruction};
use x86_64::VirtAddr;
use log::info;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(PartialEq, Clone)]
pub enum ProcessStatus
{
    Busy,
    Done(bool)
}


#[repr(C)]
pub struct NativeSysCallTable
{
    file_read_perms: fn(&str) -> Result<FilePermissions, ErrorKind>,
    file_write_perms: fn(&str, FilePermissions) -> Result<(), ErrorKind>,
    file_read: fn(&str) -> Result<Vec<u8>, ErrorKind>,
    file_write: fn(&str, data: &[u8]) -> Result<(), ErrorKind>,
    file_delete: fn(&str) -> Result<(), ErrorKind>,
    file_create: fn(&str) -> Result<(), ErrorKind>,
    time_now: fn() -> i64,
    rand_buffer: fn(buf: &mut [u8])
}

impl NativeSysCallTable
{
    pub const fn gen() -> Self
    {
        NativeSysCallTable{file_read_perms, file_write_perms, file_read, file_write, file_delete, file_create, time_now, rand_buffer}
    }

    pub fn to_byte_slice(&self) -> [u8; 64]
    {
        let mut res = [0u8; 64];
        let mut addr = self.file_read_perms.addr().addr();
        let mut arr = addr.to_le_bytes();
        for i in 0..8
        {
            res[i] = arr[i];
        }
        addr = self.file_write_perms.addr().expose_addr();
        arr = addr.to_le_bytes();
        for i in 0..8
        {
            res[i + 8] = arr[i];
        }
        addr = self.file_read.addr().expose_addr();
        arr = addr.to_le_bytes();
        for i in 0..8
        {
            res[i + 16] = arr[i];
        }
        addr = self.file_write.addr().expose_addr();
        arr = addr.to_le_bytes();
        for i in 0..8
        {
            res[i + 24] = arr[i];
        }
        addr = self.file_delete.addr().expose_addr();
        arr = addr.to_le_bytes();
        for i in 0..8
        {
            res[i + 32] = arr[i];
        }
        addr = self.file_create.addr().expose_addr();
        arr = addr.to_le_bytes();
        for i in 0..8
        {
            res[i + 40] = arr[i];
        }
        addr = self.time_now.addr().expose_addr();
        arr = addr.to_le_bytes();
        for i in 0..8
        {
            res[i + 48] = arr[i];
        }
        addr = self.rand_buffer.addr().expose_addr();
        arr = addr.to_le_bytes();
        for i in 0..8
        {
            res[i + 56] = arr[i];
        }
        res
    }
}

#[repr(C)]
pub struct FFISysCallTable
{
    c_file_read_perms: extern "C" fn(*const c_char) -> CVecShort,
    c_file_write_perms: extern "C" fn(*const c_char, *const c_uchar, c_ulonglong) -> c_schar,
    c_file_read: extern "C" fn(*const c_char) -> CVecShort,
    c_file_write: extern "C" fn(*const c_char, *const c_uchar, c_ulonglong) -> c_schar,
    c_file_delete: extern "C" fn(*const c_char) -> c_schar,
    c_file_create: extern "C" fn(*const c_char) -> c_schar,
    c_time_now: extern "C" fn() -> c_longlong,
    c_rand_buffer: extern "C" fn(*mut c_uchar, c_ulonglong)
}

impl FFISysCallTable
{
    pub const fn gen() -> Self
    {
        FFISysCallTable{c_file_read_perms, c_file_write_perms, c_file_read, c_file_write, c_file_delete, c_file_create, c_time_now, c_rand_buffer}
    }
}

#[allow(improper_ctypes_definitions)] //C apps are not supposed to use NativeSysCallTable
type StartFunc = extern "C" fn ([u8; 64], FFISysCallTable) -> bool;

#[derive(Clone)]
pub struct Process
{
    //code: Vec<u8>,
    mem_ptr: VirtAddr,
    layout: Layout,
    pub start: StartFunc,
    pub pid: u64,
    pub privileged: bool,
    pub status: ProcessStatus,
    pub retries: u64
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
        let layout = unsafe{Layout::from_size_align_unchecked(cap, 4096)};
        let ptr = unsafe{ALLOCATOR.alloc(layout)};
        let mut data_vec = unsafe{Vec::from_raw_parts(ptr, 0, cap)};
        let ptr = VirtAddr::from_ptr(ptr);
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
        NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Process{/*code: data_vec,*/ mem_ptr: ptr, layout, start: func, pid: id, privileged: true, status: ProcessStatus::Done(true), retries: 0}
    }

    pub fn call(&mut self)
    {
        let ntable = NativeSysCallTable::gen().to_byte_slice();
        let ctable = FFISysCallTable::gen();
        self.status = ProcessStatus::Busy;
        let r = (self.start)(ntable, ctable);
        self.status = ProcessStatus::Done(r);
    }
}

impl Drop for Process
{
    fn drop(&mut self)
    {
        let ptr: *mut u8 = self.mem_ptr.as_mut_ptr();
        unsafe{ALLOCATOR.dealloc(ptr, self.layout)};
    }
}