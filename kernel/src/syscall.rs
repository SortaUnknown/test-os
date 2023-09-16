use alloc::vec::Vec;
use alloc::string::String;
use embedded_io::ErrorKind;
use crate::fs::{FILESYSTEM, FileSystem, FilePermissions};
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

#[repr(C)]
pub struct CVecShort
{
    ptr: *const i16,
    len: usize
}

impl From<Vec<i16>> for CVecShort
{
    fn from(vec: Vec<i16>) -> Self
    {
        let ptr = vec.as_ptr();
        let len = vec.len();
        CVecShort{ptr, len}
    }
}

fn ffi_errorkind_res(res: Result<(), &ErrorKind>) -> i8
{
    if Ok(()) == res {-1}
    else {res.unwrap_err().clone() as i8}
}

fn ffi_byte_slice_from_parts(ptr: *const u8, len: usize) -> Vec<u8>
{
    assert!(!ptr.is_null());
    unsafe{core::slice::from_raw_parts(ptr, len)}.to_vec()
}

fn ffi_str_from_parts(ptr: *const u8, len: usize) -> String
{
    String::from_utf8(ffi_byte_slice_from_parts(ptr, len)).unwrap()
}

pub fn file_read_perms(path: &str) -> Result<FilePermissions, ErrorKind>
{
    FILESYSTEM.lock().get_perms(path)
}

pub fn file_write_perms(privileged: bool, path: &str, perms: FilePermissions) -> Result<(), ErrorKind>
{
    FILESYSTEM.lock().write_perms_checked(path, perms, privileged)
}

pub fn file_read(privileged: bool, path: &str) -> Result<Vec<u8>, ErrorKind>
{
    FILESYSTEM.lock().read_checked(path, privileged)
}

pub fn file_write(privileged: bool, path: &str, data: &[u8]) -> Result<(), ErrorKind>
{
    FILESYSTEM.lock().write_checked(path, data, privileged)
}

pub fn file_delete(privileged: bool, path: &str) -> Result<(), ErrorKind>
{
    FILESYSTEM.lock().delete_checked(path, privileged)
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
    if let Err(_) = c {crate::rand::rand_lq(buf);}
}

pub fn proc_spawn(buf: &[u8])
{
    crate::proc::Process::spawn(buf, true).call()
}


pub extern "C" fn sys_file_read_perms(path_buf_ptr: *const u8, path_buf_len: usize) -> CVecShort
{
    let path = &ffi_str_from_parts(path_buf_ptr, path_buf_len);
    let mut vec: Vec<i16> = Vec::new();
    let res = file_read_perms(path);
    let m = res.as_ref().map(|_| ());
    vec.push(ffi_errorkind_res(m).into());
    if let Ok(perms) = res
    {
        for i in perms.to_byte_arr()
        {
            vec.push(i.into());
        }
    }
    CVecShort::from(vec)
}

pub extern "C" fn sys_file_write_perms(privileged: bool, path_buf_ptr: *const u8, path_buf_len: usize, perms_buf_ptr: *const u8, perms_buf_len: usize) -> i8
{
    ffi_errorkind_res(file_write_perms(privileged, &ffi_str_from_parts(path_buf_ptr, path_buf_len), FilePermissions::from_byte_slice(&ffi_byte_slice_from_parts(perms_buf_ptr, perms_buf_len))).as_ref().map(|_| ()))
}

pub extern "C" fn sys_file_read(privileged: bool, path_buf_ptr: *const u8, path_buf_len: usize) -> CVecShort
{
    let path = &ffi_str_from_parts(path_buf_ptr, path_buf_len);
    let mut vec: Vec<i16> = Vec::new();
    let res = file_read(privileged, path);
    let m = res.as_ref().map(|_| ());
    vec.push(ffi_errorkind_res(m).into());
    if let Ok(data) = res
    {
        for i in data
        {
            vec.push(i.into());
        }
    }
    CVecShort::from(vec)
}

pub extern "C" fn sys_file_write(privileged: bool, path_buf_ptr: *const u8, path_buf_len: usize, data_buf_ptr: *const u8, data_buf_len: usize) -> i8
{
    ffi_errorkind_res(file_write(privileged, &ffi_str_from_parts(path_buf_ptr, path_buf_len), &ffi_byte_slice_from_parts(data_buf_ptr, data_buf_len)).as_ref().map(|_| ()))
}

pub extern "C" fn sys_file_delete(privileged: bool, path_buf_ptr: *const u8, path_buf_len: usize) -> i8
{
    ffi_errorkind_res(file_delete(privileged, &ffi_str_from_parts(path_buf_ptr, path_buf_len)).as_ref().map(|_| ()))
}

pub extern "C" fn sys_file_create(path_buf_ptr: *const u8, path_buf_len: usize) -> i8
{
    ffi_errorkind_res(file_create(&ffi_str_from_parts(path_buf_ptr, path_buf_len)).as_ref().map(|_| ()))
}

pub extern "C" fn sys_time_now() -> i64
{
    time_now()
}

pub extern "C" fn sys_rand_buffer(buf_ptr: *mut u8, buf_len: usize)
{
    rand_buffer(&mut ffi_byte_slice_from_parts(buf_ptr, buf_len));
}