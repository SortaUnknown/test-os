use ape_fatfs::fs::IntoStorage;
use ape_fatfs::fs::FileSystem;
use ape_fatfs::fs::FsOptions;
use ape_fatfs::fs::LossyOemCpConverter;
use ape_fatfs::dir_entry::FileAttributes;
use ape_fatfs::time::ChronoTimeProvider;
use super::FilePermissions;
use super::FileError;
use super::ata_stream::AtaStream;
use super::device_stream::DeviceStream;
use alloc::vec::Vec;

pub fn init(device: super::device_stream::DeviceStream)
{
    let fs = FileSystem::new(rws, FsOptions::new());
}

pub struct FatFileSystem
{
    device: DeviceStream,
    fs: FileSystem<AtaStream, ChronoTimeProvider, LossyOemCpConverter>
}

impl super::FileSystem for FatFileSystem
{
    fn init(device: DeviceStream) -> Self
    {
        let fs = FileSystem::new(device, FsOptions::new()).unwrap();
        FatFileSystem{device, fs}
    }

    fn get_perms(&self, path: &str) -> Result<FilePermissions, FileError>
    {
        let dir: Vec<Result<ape_fatfs::dir_entry::DirEntry<'_, AtaStream, ChronoTimeProvider, LossyOemCpConverter>, ape_fatfs::error::Error<super::ata_stream::AtaError>>> = self.fs.root_dir().open_dir(path).unwrap().iter().collect();
        let mut a = FileAttributes::default();
        for i in dir
        {
            a = i.unwrap().attributes();
            break;
        }
        Ok(FilePermissions{read_privileged: a == FileAttributes::HIDDEN, write_delete_privileged: a == FileAttributes::READ_ONLY})
    }

    fn read(&self, path: &str) -> Result<Vec<u8>, FileError>
    {
        let file = self.fs.root_dir().open_file(path).unwrap();
        let res = Vec::new();
        for i in file.extents()
        {
            let e = i.unwrap();

        }
    }
}