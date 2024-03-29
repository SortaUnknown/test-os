use std::process::Command;

fn main()
{
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));
    //println!("{:?} {:?}", qemu.get_program(), qemu.get_args());
    let exit_status = qemu.status().unwrap();
    std::process::exit(exit_status.code().unwrap_or(-1));
}