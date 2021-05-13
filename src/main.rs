use object::File;
use spear::memory::{Memory, RamDevice};
use spear::Architecture;
use spear::{cpu::Cpu, Address};

fn main() {
    let mut mem = Memory::new();

    let arch = Architecture::rv32i();
    //arch.base().write_register(2.into(), 0x8000_0000u32.into());

    let obj = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let obj = File::parse(obj.as_slice()).unwrap();
    mem.load_object(obj).unwrap();

    mem.add_device(Address::from(2000u32), RamDevice::new(0xFF));
    mem.write(2000u32.into(), 0xABu32);
    mem.write(2004u32.into(), 0xCDu32);

    let mut cpu = Cpu::new(arch, mem);
    for _ in 0..10 {
        cpu.step().unwrap();
    }
    println!("{}", cpu.arch().base());

    //println!("{:#x?}", mem.read::<u32>(0x8020_0000.into()));
    // 97 91 01 00
}
