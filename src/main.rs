use std::process;
use std::fs;
use std::io::Write;

use clap::Parser;

#[derive(Parser)]
struct Args {
  #[arg(long, default_value_t = false)]
  gdb: bool
}

fn main() {
    let args = Args::parse();

    create_import_file();

    // read env variables that were set in build script
    let bios_path = env!("BIOS_PATH");
    let mut cmd = process::Command::new("qemu-system-x86_64");
    cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
    cmd.args(["-monitor", "stdio"]);

    if args.gdb {
      cmd.args(["-gdb", "tcp::9000"]);
      cmd.arg("-S");
    }
    
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}

// GDBのPythonスクリプト用にインポートファイルを作成する
// インポートファイルの中には以下の情報を含める
//   - カーネルバイナリのパス
fn create_import_file() {
  let kernel_path = env!("KERNEL_PATH");

  let mut f = fs::File::create("./target/gdb_import.py").expect("cannot create import file for gdb");
  writeln!(f, r#"kernel_path = "{}""#, kernel_path).expect("cannot write to ./gdb_import.py");
}