fn main() {
    // read env variables that were set in build script
    let bios_path = env!("BIOS_PATH");
    println!("{}", &bios_path);
    
    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
    cmd.args(["-monitor", "stdio"]);
    cmd.args(["-gdb", "tcp::9000"]);
    cmd.arg("-S");

    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}