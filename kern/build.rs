use std::fs;

const LD_PATH: &str = "tgt/linker.ldS";

fn main() {
    println!("cargo:rerun-if-changed={}", LD_PATH);
    let arch = std::env::var_os("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch = arch.to_str().unwrap();

    let base_address: usize = {
        match arch {
            "riscv32" => 0x8040_0000,
            "riscv64" => 0xFFFF_FFC0_8020_0000,
            other_arch => panic!("Unsupported arch: {}", other_arch),
        }
    };

    let ld = fs::read_to_string(LD_PATH).unwrap();
    let ld = ld.replace("%ARCH%", arch);
    let ld = ld.replace("%BASE_ADDRESS%", &format!("{:#x}", base_address));
    fs::write(format!("tgt/{arch}.ld"), ld).unwrap();
}
