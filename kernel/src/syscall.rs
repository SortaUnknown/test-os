use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;
use embedded_io::ErrorKind;
use crate::fs::{FILESYSTEM, FileSystem, FilePermissions};
use crate::proc_watch::{PROCESS_QUEUE, RUNNING_PROCESS, find};
use core::sync::atomic::Ordering;
use core::ffi::{CStr, c_char, c_uchar, c_schar, c_longlong, c_ulonglong};
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

#[repr(i8)]
pub enum ProcessError
{
    Unprivileged,
    NotFound
}

#[repr(C)]
pub struct CVecShort
{
    res: c_char,
    ptr: *const c_uchar,
    len: c_ulonglong
}

impl From<Result<Vec<u8>, ErrorKind>> for CVecShort
{
    fn from(input: Result<Vec<u8>, ErrorKind>) -> Self
    {
        let res = ffi_errorkind_res(input.as_ref().map(|_| ()).map_err(|e| e.to_owned().to_owned()));
        let mut vec = Vec::new();
        if input.is_ok()
        {
            vec.append(&mut input.unwrap());
        }
        let ptr = vec.as_ptr();
        let len: c_ulonglong = vec.len().try_into().unwrap();
        CVecShort{res, ptr, len}
    }
}

fn check_privilege() -> bool
{
    let pq = PROCESS_QUEUE.lock();
    let i = find(pq.iter(), RUNNING_PROCESS.load(Ordering::Relaxed)).unwrap();
    pq.get(i).unwrap().privileged
}

fn ffi_str_from_ptr(ptr: *const c_char) -> String
{
    let c = unsafe{CStr::from_ptr(ptr)};
    c.to_str().unwrap().to_owned()
}

fn ffi_errorkind_res(res: Result<(), ErrorKind>) -> c_char
{
    if Ok(()) == res {-1}
    else {res.unwrap_err() as i8}
}

fn ffi_byte_slice_from_parts(ptr: *const u8, len: c_ulonglong) -> Vec<u8>
{
    assert!(!ptr.is_null());
    unsafe{core::slice::from_raw_parts(ptr, len.try_into().unwrap())}.to_vec()
}

pub fn file_read_perms(path: &str) -> Result<FilePermissions, ErrorKind>
{
    FILESYSTEM.lock().get_perms(path)
}

pub fn file_write_perms(path: &str, perms: FilePermissions) -> Result<(), ErrorKind>
{
    FILESYSTEM.lock().write_perms_checked(path, perms, check_privilege())
}

pub fn file_read(path: &str) -> Result<Vec<u8>, ErrorKind>
{
    FILESYSTEM.lock().read_checked(path, check_privilege())
}

pub fn file_write(path: &str, data: &[u8]) -> Result<(), ErrorKind>
{
    FILESYSTEM.lock().write_checked(path, data, check_privilege())
}

pub fn file_delete(path: &str) -> Result<(), ErrorKind>
{
    FILESYSTEM.lock().delete_checked(path, check_privilege())
}

pub fn file_create(path: &str) -> Result<(), ErrorKind>
{
    FILESYSTEM.lock().create(path)
}

pub fn time_now() -> i64
{
    let time = crate::time::now();
    NaiveDateTime::new(NaiveDate::from_ymd_opt(time.year.into(), time.month.into(), time.day.into()).unwrap(), NaiveTime::from_hms_opt(time.hour.into(), time.minute.into(), time.second.into()).unwrap()).timestamp()
}

pub fn rand_buffer(buf: &mut [u8])
{
    let c = crate::rand::rand_hq(buf);
    if c.is_err() {crate::rand::rand_lq(buf);}
}

pub fn proc_spawn(buf: &[u8])
{
    crate::proc_watch::spawn(crate::proc::Process::spawn(buf, true));
}

pub fn proc_kill(pid: u64) -> Result<(), ProcessError>
{
    if !check_privilege() {Err(ProcessError::Unprivileged)}
    else
    {
        if crate::proc_watch::remove(pid) {Ok(())}
        else {Err(ProcessError::NotFound)}
    }
}

pub fn proc_kill_self() -> Result<(), ProcessError>
{
    let pid = RUNNING_PROCESS.load(Ordering::Relaxed);
    if crate::proc_watch::remove(pid) {Ok(())}
    else {Err(ProcessError::NotFound)}
}

pub extern "C" fn c_file_read_perms(path: *const c_char) -> CVecShort
{
    let path = &ffi_str_from_ptr(path);
    let res = file_read_perms(path).map(|p| p.to_byte_arr().to_vec());
    CVecShort::from(res)
}

pub extern "C" fn c_file_write_perms(path: *const c_char, perms_buf_ptr: *const c_uchar, perms_buf_len: c_ulonglong) -> c_schar
{
    let path = &ffi_str_from_ptr(path);
    ffi_errorkind_res(file_write_perms(path, FilePermissions::from_byte_slice(&ffi_byte_slice_from_parts(perms_buf_ptr, perms_buf_len))).map(|_| ()))
}

pub extern "C" fn c_file_read(path: *const c_char) -> CVecShort
{
    let path = &ffi_str_from_ptr(path);
    let res = file_read(path);
    CVecShort::from(res)
}

pub extern "C" fn c_file_write(path: *const c_char, data_buf_ptr: *const c_uchar, data_buf_len: c_ulonglong) -> c_schar
{
    let path = &ffi_str_from_ptr(path);
    ffi_errorkind_res(file_write(path, &ffi_byte_slice_from_parts(data_buf_ptr, data_buf_len)).map(|_| ()))
}

pub extern "C" fn c_file_delete(path: *const c_char) -> c_schar
{
    let path = &ffi_str_from_ptr(path);
    ffi_errorkind_res(file_delete(path).map(|_| ()))
}

pub extern "C" fn c_file_create(path: *const c_char) -> c_schar
{
    let path = &ffi_str_from_ptr(path);
    ffi_errorkind_res(file_create(path).map(|_| ()))
}

pub extern "C" fn c_time_now() -> c_longlong
{
    time_now()
}

pub extern "C" fn c_rand_buffer(buf_ptr: *mut c_uchar, buf_len: c_ulonglong)
{
    rand_buffer(&mut ffi_byte_slice_from_parts(buf_ptr, buf_len));
}