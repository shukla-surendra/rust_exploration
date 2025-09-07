#!/bin/bash

# Usage: ./build.sh <rust_file.rs>
# Example: ./build.sh hello.rs
# Will produce: target/hello
# Then run: target/hello

# Check if a filename was provided
if [ -z "$1" ]; then
    echo "Usage: $0 <rust_file.rs>"
    exit 1
fi

# Setup paths
filename=$(basename -- "$1")
name="${filename%.*}"
target_dir="target"

# Ensure target directory exists
mkdir -p "$target_dir"

# Compile with rustc
rustc "$1" -o "$target_dir/$name"
if [ $? -ne 0 ]; then
    echo "❌ Compilation failed!"
    exit 1
fi

# Run
echo "🚀 Running $target_dir/$name ..."
"$target_dir/$name"
