use object::File;
use spear::emulator::Emulator;

fn main() {
    init_logger();

    let obj = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let obj = File::parse(obj.as_slice()).unwrap();
    let mut emu = Emulator::from_object_with_htif(obj).unwrap();

    emu.run();
    log::info!("{}", emu.cpu().arch().base());
}

fn init_logger() {
    use env_logger::Env;

    let env = Env::default().filter_or("SPEAR_LOG", "info");

    env_logger::Builder::from_env(env)
        .format(|f, record| {
            use env_logger::fmt;
            use std::io::Write;

            fn colored_level<'a>(
                style: &'a mut fmt::Style,
                level: log::Level,
            ) -> fmt::StyledValue<'a, &'static str> {
                match level {
                    log::Level::Trace => style.set_color(fmt::Color::Magenta).value("TRACE"),
                    log::Level::Debug => style.set_color(fmt::Color::Blue).value("DEBUG"),
                    log::Level::Info => style.set_color(fmt::Color::Green).value("INFO "),
                    log::Level::Warn => style.set_color(fmt::Color::Yellow).value("WARN "),
                    log::Level::Error => style.set_color(fmt::Color::Red).value("ERROR"),
                }
            }

            let mut style = f.style();
            style.set_bold(true);
            let level = colored_level(&mut style, record.level());
            writeln!(f, "{} > {}", level, record.args())
        })
        .init();
}
