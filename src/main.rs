use spear::cpu::Cpu;
use spear::memory::{Memory, RamDevice};
use spear::Architecture;

fn main() {
    let mut mem = Memory::new();
    // 0x03848493
    let ram = RamDevice::from_vec(vec![0x93, 0x84, 0x84, 0x03]);
    mem.add_device(0x8000_0000.into(), ram);

    let mut arch = Architecture::rv32i();
    arch.base().write_register(2.into(), 0x8000_0000.into());

    let mut cpu = Cpu::new(arch, mem);
    cpu.step().unwrap();
    println!("{:#x?}", cpu.arch().base());

    //let obj = std::fs::read("../novos/target/riscv64gc-unknown-none-elf/release/kernel").unwrap();
    //let obj = File::parse(&obj).unwrap();
    //mem.load_object(obj).unwrap();

    //println!("{:#x?}", mem.read::<u32>(0x8020_0000.into()));
    // 97 91 01 00
}
