//mod fatfs;
//mod ntfs;
mod testfs;

use alloc::vec::Vec;
use embedded_io::ErrorKind;

#[derive(Default)]
pub struct FilePermissions
{
    read_privileged: bool,
    write_delete_privileged: bool
}

impl FilePermissions
{
    pub fn from_byte_slice(slice: &[u8]) -> Self
    {
        let read_privileged = byte_to_bool(slice[0]);
        let write_delete_privileged = byte_to_bool(slice[1]);
        FilePermissions {read_privileged, write_delete_privileged}
    }
    pub fn to_byte_arr(&self) -> [u8; 2]
    {
        let mut res = [0u8; 2];
        res[0] = self.read_privileged.into();
        res[1] = self.read_privileged.into();
        res
    }
}

#[derive(Default)]
pub struct FileDates
{
    create_date : i64,
    modify_date: i64,
    access_date: i64
}

impl FileDates
{
    pub fn from_byte_slice(slice: &[u8]) -> Self
    {
        let split1 = slice.split_at(8);
        let split2 = split1.1.split_at(8);
        let create_date = i64::from_le_bytes(slice_to_arr(split1.0));
        let modify_date = i64::from_le_bytes(slice_to_arr(split2.0));
        let access_date = i64::from_be_bytes(slice_to_arr(split2.1));
        FileDates {create_date, modify_date, access_date}
    }

    pub fn to_byte_arr(&self) -> [u8; 24]
    {
        let mut res = [0u8; 24];
        let mut t = self.create_date.to_le_bytes();
        for i in 0..8
        {
            res[i] = t[i];
        }
        t = self.modify_date.to_le_bytes();
        for i in 0..8
        {
            res[i + 8] = t[i];
        }
        t = self.access_date.to_le_bytes();
        for i in 0..8
        {
            res[i + 16] = t[i];
        }
        res
    }
}

/*pub struct Directory
{
    entries: Vec<String>
}*/

pub struct File
{
    data: Vec<u8>,
    permissions: FilePermissions,
    dates: FileDates
}

impl File
{
    pub const fn create() -> Self
    {
        let permissions = FilePermissions{read_privileged: false, write_delete_privileged: false};
        let dates = FileDates{create_date: 0, modify_date: 0, access_date: 0};
        File{data: Vec::new(), permissions, dates}
    }

    pub fn read(&mut self, privileged: bool) -> Result<Vec<u8>, ErrorKind>
    {
        if self.permissions.read_privileged && !privileged {return Err(ErrorKind::PermissionDenied);}
        self.dates.access_date = 0;
        Ok(self.data.clone())
    }

    pub fn write(&mut self, data: &[u8], privileged: bool) -> Result<(), ErrorKind>
    {
        if self.permissions.write_delete_privileged && !privileged {return Err(ErrorKind::PermissionDenied);}
        self.dates.modify_date = 0;
        self.data = data.into();
        Ok(())
    }

    pub fn mod_permissions(&mut self, read: Option<bool>, write: Option<bool>, privileged: bool) -> Result<(), ErrorKind>
    {
        if !privileged {return Err(ErrorKind::PermissionDenied);}
        let read_privileged =
        {
            if let Some(val) = read {val}
            else {self.permissions.read_privileged}
        };
        let write_delete_privileged =
        {
            if let Some(val) = write {val}
            else {self.permissions.write_delete_privileged}
        };
        self.permissions = FilePermissions{read_privileged, write_delete_privileged};
        Ok(())
    }
}

pub trait FileSystem
{
    fn init(device: &'static mut dyn crate::device::DeviceStream) -> Self;

    fn get_perms(&mut self, path: &str) -> Result<FilePermissions, ErrorKind>;
    fn write_perms(&mut self, path: &str, perms: FilePermissions) -> Result<(), ErrorKind>;
    
    fn read(&mut self, path: &str) -> Result<Vec<u8>, ErrorKind>;
    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), ErrorKind>;
    fn delete(&mut self, path: &str) -> Result<(), ErrorKind>;
    fn create(&mut self, path: &str) -> Result<(), ErrorKind>;

    fn write_perms_checked(&mut self, path: &str, perms: FilePermissions, privileged: bool) -> Result<(), ErrorKind>
    {
        if !privileged {return Err(ErrorKind::PermissionDenied);}
        self.write_perms(path, perms)
    }

    fn read_checked(&mut self, path: &str, privileged: bool) -> Result<Vec<u8>, ErrorKind>
    {
        let perms = self.get_perms(path)?;
        if perms.read_privileged && !privileged {return Err(ErrorKind::PermissionDenied);}
        self.read(path)
    }

    fn write_checked(&mut self, path: &str, data: &[u8], privileged: bool) -> Result<(), ErrorKind>
    {
        let perms = self.get_perms(path)?;
        if perms.write_delete_privileged && !privileged {return Err(ErrorKind::PermissionDenied);}
        self.write(path, data)
    }

    fn delete_checked(&mut self, path: &str, privileged: bool) -> Result<(), ErrorKind>
    {
        let perms = self.get_perms(path)?;
        if perms.write_delete_privileged && !privileged {return Err(ErrorKind::PermissionDenied);}
        self.delete(path)
    }
}

pub fn byte_to_bool(byte: u8) -> bool
{
    return byte != 0;
}

pub fn slice_to_arr(slice: &[u8]) -> [u8; 8]
{
    let mut res = [0u8; 8];
    for i in 0..8
    {
        res[i] = slice[i];
    }
    res
}