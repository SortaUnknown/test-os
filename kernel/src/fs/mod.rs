use alloc::vec::Vec;
use alloc::string::String;

pub struct Directory
{
    entries: Vec<String>
}

pub struct File
{
    data: Vec<u8>,
    permissions: FilePermissions,
    dates: FileDates
}

pub struct FilePermissions
{
    read_privileged: bool,
    write_delete_privileged: bool
}

pub struct FileDates
{
    create_date : i64,
    modify_date: i64,
    access_date: i64
}

impl File
{
    pub const fn create() -> Self
    {
        let permissions = FilePermissions{read_privileged: false, write_delete_privileged: false};
        let dates = FileDates{create_date: 0, modify_date: 0, access_date: 0};
        File{data: Vec::new(), permissions, dates}
    }

    pub fn read(&mut self, privileged: bool) -> Result<Vec<u8>, &str>
    {
        if self.permissions.read_privileged && !privileged {return Err("Read operation failed: Not privileged");}
        self.dates.access_date = 0;
        Ok(self.data.clone())
    }

    pub fn write(&mut self, data: &[u8], privileged: bool) -> Result<(), &str>
    {
        if self.permissions.write_delete_privileged && !privileged {return Err("Write operation failed: Not privileged");}
        self.dates.modify_date = 0;
        self.data = data.into();
        Ok(())
    }

    pub fn mod_permissions(&mut self, read: Option<bool>, write: Option<bool>, privilege_execute: Option<bool>, privileged: bool) -> Result<(), &str>
    {
        if !privileged {return Err("Permission modification failed: Not privileged");}
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