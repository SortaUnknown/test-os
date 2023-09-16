use x86_64::instructions::port::Port;

pub fn rand_hq(buf: &mut [u8]) -> Result<(), ()>
{
    let opt = x86_64::instructions::random::RdRand::new();
    if let Some(r) = opt
    {
        for i in 0..buf.len()
        {
            let mut opt = None;
            while opt == None
            {
                opt = r.get_u16();
                if let Some(c) = opt
                {
                    if c > u8::MAX.into() {opt = r.get_u16();}
                }
            }
        buf[i] = opt.unwrap() as u8;
        }
        Ok(())
    }
    else {Err(())}
}

pub fn rand_lq(buf: &mut [u8])
{
    for i in 0..buf.len()
    {
        let mut port = Port::new(0x40 + (i as u16 % 3));
        buf[i] = unsafe{port.read()}; //it is safe to read from ports 0x40 to 0x42
    }
}