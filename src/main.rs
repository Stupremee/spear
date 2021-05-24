use object::File;
use spear::emulator::Emulator;

fn main() {
    let obj = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let obj = File::parse(obj.as_slice()).unwrap();
    let mut emu = Emulator::from_object_with_htif(obj).unwrap();

    emu.run();
    println!("{}", emu.cpu().arch().base());

    //println!("{:#x?}", mem.read::<u32>(0x8020_0000.into()));
    // 97 91 01 00
}
