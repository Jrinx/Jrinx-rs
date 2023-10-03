use std::{fs, io};

fn main() {
    let arch = std::env::var_os("CARGO_CFG_TARGET_ARCH")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    if let Some(lds_path) = find_lds(&arch) {
        create_ld_file(&arch, &lds_path).unwrap();
    }
}

fn create_ld_file(arch: &str, lds_path: &str) -> io::Result<()> {
    fs::write(
        format!("tgt/{}.ld", &arch),
        fs::read_to_string(lds_path)?.replace(
            "BASE_ADDRESS",
            format!("{:#x}", base_addr_of(arch)).as_str(),
        ),
    )
}

fn find_lds(arch: &str) -> Option<String> {
    for i in 0..arch.len() {
        let path = format!("tgt/{}.ldS", &arch[..arch.len() - i]);
        if fs::metadata(&path).is_ok() {
            return Some(path);
        }
    }
    None
}

fn base_addr_of(arch: &str) -> usize {
    match arch {
        "riscv32" => 0x80400000,
        "riscv64" => 0x80200000,
        _ => panic!("Unsupported arch: {}", arch),
    }
}
