# Jrinx-rs

Jrinx-rs 是 [Jrinx](https://github.com/Coekjan/Jrinx) 的 Rust 语言版本。

## 快速开始

> Windows 与 \*nix 开发者皆可进行开发、编译、运行，但是目前仅支持在 \*nix 系统上进行回归测试。

首先你需要安装 QEMU、Python 与 Rust 工具链：
- QEMU：https://www.qemu.org/download
  - 尽可能使用最新的 QEMU 版本
- Python：https://www.python.org/downloads
  - 尽可能使用 [.python-version](.python-version) 中的版本
- Rust：https://www.rust-lang.org/tools/install

然后克隆本仓库，并进入仓库目录，运行：

```console
$ cargo qemu -a riscv64
```

即可编译代码并（在 QEMU/virt 上）运行 riscv64 架构上的 Jrinx。

此外，在 \*nix 系统上使用：

```console
$ # NOTE: `pip install -r requirements.txt` is needed for the first time
$ ./scripts/test-run -pr
```

可运行项目的回归测试。

## 参与贡献

项目开发仍在早期，代码与文档尚未完善，各种 API 也可能随时发生变化。欢迎各位开发者的贡献，但在着手开发前请尽可能与我联系，避免不必要的冲突。
