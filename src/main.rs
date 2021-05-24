use object::File;
use spear::memory::RamDevice;
use spear::{emulator::Emulator, Address};

fn main() {
    let obj = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let obj = File::parse(obj.as_slice()).unwrap();
    let mut emu = Emulator::from_object_with_htif(obj).unwrap();

    let mem = emu.cpu().mem();
    mem.add_device(Address::from(2000u32), RamDevice::new(0xFF));
    mem.write(2000u32.into(), 0xABu32).unwrap();
    mem.write(2004u32.into(), 0xCDu32).unwrap();

    emu.run();
    println!("{}", emu.cpu().arch().base());

    //println!("{:#x?}", mem.read::<u32>(0x8020_0000.into()));
    // 97 91 01 00
}
