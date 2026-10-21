use update_service::*;

fn main() {
    let mut args = std::env::args().skip(1);
    let platform = match args.next().as_deref() {
        Some("windows") => Platform::Windows,
        None | Some("pi") => Platform::RaspberryPi,
        Some(other) => panic!("unknown platform `{other}` (windows|pi)"),
    };
    let optimization = match args.next().as_deref() {
        Some("full") => Optimization::Full,
        None | Some("basic") => Optimization::Basic,
        Some(other) => panic!("unknown optimization `{other}` (basic|full)"),
    };
    build(platform, optimization).unwrap();
}
