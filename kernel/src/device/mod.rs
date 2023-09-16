pub mod ata;

use embedded_io::{Read, Write, Seek, SeekFrom, Error, ErrorType, ErrorKind};
use spin::Mutex;

pub static ATA_DEVICE: Mutex<ata::AtaStream> = Mutex::new(ata::AtaStream::new());

pub fn test_init() -> ata::AtaStream
{
    ata::init_test()
}

#[derive(Debug)]
pub struct DeviceError
{
    error: ErrorKind
}

impl DeviceError
{
    pub fn new(error: ErrorKind) -> Self
    {
        DeviceError{error}
    }
}

impl Error for DeviceError
{
    fn kind(&self) -> ErrorKind
    {
        self.error
    }
}

impl From<DeviceError> for ErrorKind
{
    fn from(error: DeviceError) -> Self
    {
        error.kind()
    }
}

pub trait DeviceStream
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DeviceError>;

    fn write(&mut self, buf: &[u8]) -> Result<usize, DeviceError>;

    fn flush(&mut self) -> Result<(), DeviceError>;

    fn seek(&mut self, pos: SeekFrom) -> Result<u64, DeviceError>;
}

impl ErrorType for dyn DeviceStream
{
    type Error = DeviceError;
}

impl Read for dyn DeviceStream
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DeviceError>
    {
        self.read(buf)
    }
}

impl Write for dyn DeviceStream
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, DeviceError>
    {
        self.write(buf)
    }

    fn flush(&mut self) -> Result<(), DeviceError>
    {
        self.flush()
    }
}

impl Seek for dyn DeviceStream
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, DeviceError>
    {
        self.seek(pos)
    }
}