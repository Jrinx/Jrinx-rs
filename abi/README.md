# Jrinx ABI

[Jrinx](https://github.com/Jrinx/Jrinx-rs) 的 ABI 定义。

## 使用

使用命令：

```console
$ cargo add jrinx-abi
```

将 jrinx-abi 添加到你的依赖中，若你正在开发 Jrinx 的用户态程序，并存在使用 Jrinx 系统调用的需求，可启用 `sysfn` 特性：

```console
$ cargo add jrinx-abi --features=sysfn
```

## 开源协议

Jrinx ABI 与 Jrinx 本体一样，以 MIT 协议开源，许可证文件请参见 [LICENSE](../LICENSE)。
