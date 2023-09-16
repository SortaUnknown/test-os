use cmos_rtc::ReadRTC;
use spin::Mutex;

static RTC: Mutex<ReadRTC> = Mutex::new(ReadRTC::new(0, 0xA5));

pub fn now() -> cmos_rtc::Time
{
    RTC.lock().read()
}