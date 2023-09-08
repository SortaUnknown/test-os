use super::{FilePermissions, FileDates, slice_to_arr};
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::borrow::ToOwned;
use crate::device::DeviceStream;
use embedded_io::{ErrorKind, SeekFrom};

const BLOCK_SIZE: usize = 512;

struct MetadataBlock
{
    name: String,
    perms: FilePermissions,
    dates: FileDates,
    data_start_block: u64,
    data_block_len: u64,
    last_block_len: u64
}

impl MetadataBlock
{
    pub fn new(name: &str, data_start_block: u64) -> Self
    {
        MetadataBlock{name: name.to_string(), perms: FilePermissions::default(), dates: FileDates::default(), data_start_block, data_block_len: 0, last_block_len: 0}
    }

    pub fn parse(slice: &[u8]) -> Self
    {
        let split1 = slice.split_at(ata_x86::ATA_BLOCK_SIZE - 50);
        let split2 = split1.1.split_at(2);
        let split3 = split2.1.split_at(24);
        let split4 = split3.1.split_at(8);
        let split5 = split4.1.split_at(8);
        let split6 = split5.1.split_at(8);
        let name =
        {
            let mut vec: Vec<u8> = Vec::new();
            for i in split1.0
            {
                let o = i.to_owned();
                if o != 0 {vec.push(o);}
            }
            String::from_utf8(vec).unwrap()
        };
        let perms = FilePermissions::from_byte_slice(split2.0);
        let dates = FileDates::from_byte_slice(split3.0);
        let data_start_block = u64::from_le_bytes(slice_to_arr(split4.0));
        let data_block_len = u64::from_le_bytes(slice_to_arr(split5.0));
        let last_block_len = u64::from_le_bytes(slice_to_arr(split6.0));
        MetadataBlock{name, perms, dates, data_start_block, data_block_len, last_block_len}
    }

    pub fn to_bytes(&self) -> [u8; ata_x86::ATA_BLOCK_SIZE]
    {
        let mut res = [0u8; ata_x86::ATA_BLOCK_SIZE];
        let name_vec = self.name.as_bytes().to_vec();
        for i in 0..ata_x86::ATA_BLOCK_SIZE - 50
        {
            if i < name_vec.len() {res[i] = name_vec[i];}
        }
        let perms_arr = self.perms.to_byte_arr();
        let dates_arr = self.dates.to_byte_arr();
        let start_arr = self.data_start_block.to_le_bytes();
        let block_len_arr = self.data_block_len.to_le_bytes();
        let last_len_arr = self.last_block_len.to_le_bytes();
        for i in 0..2
        {
            res[(ata_x86::ATA_BLOCK_SIZE - 50) + i] = perms_arr[i];
        }
        for i in 0..24
        {
            res[(ata_x86::ATA_BLOCK_SIZE - 48) + i] = dates_arr[i];
        }
        for i in 0..8
        {
            res[(ata_x86::ATA_BLOCK_SIZE - 24) + i] = start_arr[i];
        }
        for i in 0..8
        {
            res[(ata_x86::ATA_BLOCK_SIZE - 16) + i] = block_len_arr[i];
        }
        for i in 0..8
        {
            res[(ata_x86::ATA_BLOCK_SIZE - 8) + i] = last_len_arr[i];
        }
        res
    }
}

pub struct TestFS
{
    free_data_block: u64,
    free_file_block: u64,
    device: &'static mut dyn DeviceStream
}

impl TestFS
{
    fn find_file(&mut self, path: &str) -> Result<(MetadataBlock, u32), ErrorKind>
    {
        let mut buf = [0u8; BLOCK_SIZE];
        for i in 1..21
        {
            self.device.seek(SeekFrom::Start(i * BLOCK_SIZE as u64))?;
            self.device.read(&mut buf)?;
            let meta = MetadataBlock::parse(&buf);
            if path.to_owned() == meta.name {return Ok((meta, i.try_into().unwrap()));}
        }
        Err(ErrorKind::NotFound)
}
}

impl super::FileSystem for TestFS
{
    fn init(device: &'static mut dyn DeviceStream) -> Self
    {
        let mut buf = [0u8; 8];
        device.seek(SeekFrom::Start(0)).unwrap();
        device.read(&mut buf).unwrap();
        let free_data_block = u64::from_le_bytes(buf);
        device.read(&mut buf).unwrap();
        let free_file_block = u64::from_le_bytes(buf);
        TestFS{free_data_block, free_file_block, device}
    }

    fn get_perms(&mut self, path: &str) -> Result<FilePermissions, ErrorKind>
    {
        Ok(self.find_file(path)?.0.perms)
    }

    fn write_perms(&mut self, path: &str, perms: FilePermissions) -> Result<(), ErrorKind>
    {
        let mut meta = self.find_file(path)?;
        meta.0.perms = perms;
        let buf = meta.0.to_bytes();
        self.device.seek(SeekFrom::Start(meta.1 as u64 * BLOCK_SIZE as u64))?;
        self.device.write(&buf)?;
        Ok(())
    }

    fn read(&mut self, path: &str) -> Result<Vec<u8>, ErrorKind>
    {
        let meta = self.find_file(path)?.0;
        let mut data = Vec::new();
        let end_block = meta.data_start_block + meta.data_block_len;
        let mut buf = [0u8; BLOCK_SIZE];
        for i in meta.data_start_block..end_block
        {
            self.device.seek(SeekFrom::Start(i * BLOCK_SIZE as u64))?;
            self.device.read(&mut buf)?;
            if i == end_block - 1
            {
                let mut res: Vec<u8> = Vec::new();
                for a in 0..meta.last_block_len
                {
                    res.push(buf[a as usize]);
                }
                data.append(&mut res);
            }
            else {data.append(&mut buf.to_vec());}
        }
        Ok(data)
    }

    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), ErrorKind>
    {
        let mut data_vec = data.to_vec();
        let mut meta = self.find_file(path)?;
        let mut g = data.len() / BLOCK_SIZE;
        let mut r = BLOCK_SIZE;
        if data.len() % BLOCK_SIZE != 0
        {
            g += 1;
            let e = BLOCK_SIZE * g - data.len();
            r = BLOCK_SIZE - e;
            let mut extra = vec![0u8; e];
            data_vec.append(&mut extra);
        }
        meta.0.data_start_block = self.free_data_block;
        meta.0.data_block_len = g as u64;
        meta.0.last_block_len = r as u64;
        let mut split: (&[u8], &[u8]) = (data_vec.as_ref(), data_vec.as_ref());
        for i in 0..g
        {
            split = split.1.split_at(BLOCK_SIZE);
            let split = data_vec.split_at(BLOCK_SIZE);
            self.device.seek(SeekFrom::Start((i as u64 + meta.0.data_start_block) * BLOCK_SIZE as u64))?;
            self.device.write(split.0)?;
        }
        let meta_arr = meta.0.to_bytes();
        self.device.seek(SeekFrom::Start(meta.1 as u64 * BLOCK_SIZE as u64))?;
        self.device.write(&meta_arr)?;
        Ok(())
    }

    fn delete(&mut self, path: &str) -> Result<(), ErrorKind>
    {
        let meta = self.find_file(path)?;
        let buf = [0u8; BLOCK_SIZE];
        self.device.seek(SeekFrom::Start(meta.1 as u64 * BLOCK_SIZE as u64))?;
        self.device.write(&buf)?;
        Ok(())
    }

    fn create(&mut self, path: &str) -> Result<(), ErrorKind>
    {
        if self.free_file_block > 20 {return Err(ErrorKind::AddrNotAvailable);}
        let buf = MetadataBlock::new(path, self.free_file_block).to_bytes();
        self.device.seek(SeekFrom::Start(self.free_file_block * BLOCK_SIZE as u64))?;
        self.device.write(&buf)?;
        self.free_file_block += 1;
        Ok(())
    }
}