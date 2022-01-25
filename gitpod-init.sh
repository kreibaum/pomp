# Prebuild file

# Prepare backend build
cargo build

# Prepare frontend
swc static/main.ts -o target/main.js
