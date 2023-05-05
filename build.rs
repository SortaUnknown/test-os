use bootloader::DiskImageBuilder;
use std::env::var;
use std::path::PathBuf;

fn main()
{
    //set by cargo for the kernel artifact dependency
    let kernel_path = var("CARGO_BIN_FILE_KERNEL").unwrap();
    let disk_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));

    //specify output paths
    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("test-os-uefi.img");
    let bios_path = out_dir.join("test-os-bios.img");

    //create the disk images
    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    //pass the disk image paths via environment variables
    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}