# Phytium SD MMC Driver

Rust 实现的SD MMC驱动

## 测试


```bash
cargo install ostool

# 运行测试，飞腾派dtb文件选择 firmware/phytium.dtb，默认DMA模式
cargo test --test test --  --show-output
```
如果需要测试PIO模式，需要执行如下指令，或直接修改`Cargo.toml`
```bash
cargo test --test test --no-default-features --features pio -- --show-output 
```
