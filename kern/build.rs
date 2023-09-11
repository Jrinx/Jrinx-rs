use std::{
    fs,
    io::{Read, Write},
};

fn main() {
    let arch = std::env::var_os("CARGO_CFG_TARGET_ARCH")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    if let Some(lds_path) = find_lds(&arch) {
        create_ld_file(&arch, &lds_path);
    }
}

fn create_ld_file(arch: &str, lds_path: &str) {
    let out_path = format!("tgt/{}.ld", &arch);
    let mut out_file = fs::File::create(&out_path).unwrap();
    let mut lds_file = fs::File::open(&lds_path).unwrap();

    let mut ori_content = String::new();
    lds_file.read_to_string(&mut ori_content).unwrap();

    let out_content = ori_content.replace(
        "BASE_ADDRESS",
        format!("{:#x}", base_addr_of(arch)).as_str(),
    );

    out_file.write_all(out_content.as_bytes()).unwrap();
}

fn find_lds(arch: &str) -> Option<String> {
    for i in 0..arch.len() {
        let path = format!("tgt/{}.ldS", &arch[..arch.len() - i]);
        if let Ok(_) = fs::metadata(&path) {
            return Some(path);
        }
    }
    return None;
}

fn base_addr_of(arch: &str) -> usize {
    match arch {
        "riscv32" => 0x80400000,
        "riscv64" => 0x80200000,
        _ => panic!("Unsupported arch: {}", arch),
    }
}
