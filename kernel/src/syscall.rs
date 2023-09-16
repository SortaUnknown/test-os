use alloc::vec::Vec;
use embedded_io::ErrorKind;
use crate::fs::{FILESYSTEM, FileSystem, FilePermissions};
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

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