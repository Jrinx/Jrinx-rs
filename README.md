# Jrinx-rs

![test](https://github.com/Coekjan/Jrinx-rs/actions/workflows/test.yml/badge.svg?branch=master)

Jrinx-rs 是 [Jrinx](https://github.com/Coekjan/Jrinx) 的 Rust 语言版本。

## 快速开始

> 开发、编译、运行以及回归测试均要求 \*nix 环境。

首先你需要安装 QEMU、Python 与 Rust 工具链：
- QEMU：https://www.qemu.org/download
  - 尽可能使用最新的 QEMU 版本
- Python：https://www.python.org/downloads
  - 尽可能使用 [.python-version](.python-version) 中的版本
- Rust：https://www.rust-lang.org/tools/install

此外，你还需通过 cargo 安装 [cargo-binutils](https://github.com/rust-embedded/cargo-binutils)：

```console
$ cargo install cargo-binutils
```

然后克隆本仓库，并进入仓库目录，运行：

```console
$ cargo uprog -a riscv64  # build user programs (artifacts can be found in `uprog/riscv64/release`)
$ cargo ar -s uprog/riscv64/release  # archive user programs (archive can be found at `uprog.jrz`)
$ cargo qemu -a riscv64
```

即可编译代码并（在 QEMU/virt 上）运行 riscv64 架构上的 Jrinx。

此外，使用：

```console
$ # NOTE: `pip install -r requirements.txt` is needed for the first time
$ ./scripts/test-run -pr
```

可运行项目的回归测试。

## 参与贡献

项目开发仍在早期，代码与文档尚未完善，各种 API 也可能随时发生变化。欢迎各位开发者的贡献，但在着手开发前请尽可能与我联系，避免不必要的冲突。
