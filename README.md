# rive_demo

Example of integration Rive animation to rust program.

1. Clone the repo:
```bash
git clone --recurse-submodules https://github.com/obox-systems/rive_demo.git
```

2. Update all required submodules for rive-rs building:
```bash
cd submodules/rive-rs/ && git submodule add --branch release-v2.0 https://github.com/facebook/yoga submodules/yoga && git submodule update --init && cd ../..
```

3. Run:
```bash
cargo run --release
```
