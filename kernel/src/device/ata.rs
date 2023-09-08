use embedded_io::{SeekFrom, ErrorKind};
use super::DeviceError;
use alloc::vec::Vec;
use alloc::vec;

pub struct AtaStream
{
    cursor: u64
}

impl super::DeviceStream for AtaStream
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DeviceError>
    {
        if self.cursor + buf.len() as u64 > u64::MAX {return Err(DeviceError::new(ErrorKind::InvalidInput));}
        let mut res = Vec::new();
        let cs = self.cursor % ata_x86::ATA_BLOCK_SIZE as u64 != 0;
        let ce = (self.cursor + buf.len() as u64) % ata_x86::ATA_BLOCK_SIZE as u64 != 0;
        let start_block: u32 = (self.cursor / ata_x86::ATA_BLOCK_SIZE as u64).try_into().unwrap();
        let end_block: u32 = ((buf.len() as u64 + self.cursor) / ata_x86::ATA_BLOCK_SIZE as u64).try_into().unwrap();
        let mut tbuf = [0u8; ata_x86::ATA_BLOCK_SIZE];
        let mut sn = 0;
        if cs
        {
            sn = 1;
            let s: usize = (self.cursor - ata_x86::ATA_BLOCK_SIZE as u64 * start_block as u64).try_into().unwrap();
            let mut vec = vec![0u8; s.try_into().unwrap()];
            ata_x86::read(0, 0, start_block.try_into().unwrap(), &mut tbuf);
            for i in 0..s
            {
                vec[i] = tbuf[ata_x86::ATA_BLOCK_SIZE - s + i];
            }
            res.append(&mut vec);
        }
        for i in (start_block + sn)..end_block
        {
            ata_x86::read(0, 0, i, &mut tbuf);
            res.append(&mut tbuf.to_vec());
        }
        if ce
        {
            let r: usize = ((self.cursor + buf.len() as u64) - (ata_x86::ATA_BLOCK_SIZE as u64 * end_block as u64)).try_into().unwrap();
            ata_x86::read(0, 0, end_block, &mut tbuf);
            let mut vec = vec![0u8; r.try_into().unwrap()];
            for i in 0..r
            {
                vec[i] = tbuf[i];
            }
            res.append(&mut vec);
        }
        self.cursor += buf.len() as u64;
        Ok(buf.len())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, DeviceError>
    {
        if self.cursor + buf.len() as u64 > u64::MAX {return Err(DeviceError::new(ErrorKind::InvalidInput));}
        let cs = self.cursor % ata_x86::ATA_BLOCK_SIZE as u64 != 0;
        let ce = (self.cursor + buf.len() as u64) % ata_x86::ATA_BLOCK_SIZE as u64 != 0;
        let start_block: u32 = (self.cursor / ata_x86::ATA_BLOCK_SIZE as u64).try_into().unwrap();
        let end_block: u32 = ((buf.len() as u64 + self.cursor) / ata_x86::ATA_BLOCK_SIZE as u64).try_into().unwrap();
        let mut tbuf = [0u8; ata_x86::ATA_BLOCK_SIZE];
        let mut sn = 0;
        let mut s: usize = 0;
        if cs
        {
            sn = 1;
            s = (self.cursor - ata_x86::ATA_BLOCK_SIZE as u64 * start_block as u64).try_into().unwrap();
            ata_x86::read(0, 0, start_block, &mut tbuf);
            for i in 0..s
            {
                tbuf[ata_x86::ATA_BLOCK_SIZE - s + i] = buf[i];
            }
            ata_x86::write(0, 0, start_block, &tbuf);
        }
        for i in (start_block + sn)..end_block
        {
            for a in 0..ata_x86::ATA_BLOCK_SIZE
            {
                tbuf[a] = buf[s + ata_x86::ATA_BLOCK_SIZE * i as usize + a];
            }
            ata_x86::write(0, 0, i, &tbuf);
        }
        if ce
        {
            let r: usize = ((self.cursor + buf.len() as u64) - (ata_x86::ATA_BLOCK_SIZE as u64 * end_block as u64)).try_into().unwrap();
            ata_x86::read(0, 0, end_block, &mut tbuf);
            for i in 0..r
            {
                tbuf[i] = buf[buf.len() - r + i];
            }
            ata_x86::write(0, 0, end_block, &tbuf);
        }
        self.cursor += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), DeviceError>
    {
        Ok(())
    }

    fn seek(&mut self, pos: SeekFrom) -> Result<u64, DeviceError>
    {
        match pos
        {
            SeekFrom::Start(p) => self.cursor = p,
            SeekFrom::End(p) =>
            {
                if p > 0 {return Err(DeviceError::new(ErrorKind::AddrNotAvailable));}
                self.cursor = u64::MAX + p as u64;
            },
            SeekFrom::Current(p) =>
            {
                if self.cursor + p as u64 > u64::MAX {return Err(DeviceError::new(ErrorKind::AddrNotAvailable));}
                self.cursor += p as u64;
            }
        }
        Ok(self.cursor)
    }
}