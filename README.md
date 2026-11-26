# bvtop

A minimal GPU monitor for viewing over SSH on small screens.

<img src="raspberrypi.png" width="480" alt="raspberrypi">

## Motivation

[nvtop](https://github.com/Syllo/nvtop) is excellent but information-dense — hard to read from a distance on a small display. bvtop shows just four numbers in large block digits, readable from across the room.

## Build

Requires a C toolchain and Rust; NVML is loaded at runtime from the NVIDIA driver.

```bash
cargo build --release
```

The binary is at `target/release/bvtop`.

## Usage
```bash
./bvtop
# Press q to quit
```

## Constraints

- **Single GPU only.** Refuses to start if multiple GPUs are detected.
