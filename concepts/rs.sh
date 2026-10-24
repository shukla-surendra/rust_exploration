#!/bin/bash
# Shebang: the first line tells the OS which interpreter runs this file.
# Usage (from inside concepts/):  ./rs.sh box.rs   ->  builds target/box

# `set -e` exits immediately if any command fails, so we don't keep going
# after an error (e.g. a compile failure).
set -e

# $1 is the first command-line argument ("box.rs").
# Assignment rule: NO spaces around `=`. `file_name = $1` would make bash try
# to run a command named `file_name` with arguments `=` and `$1`.
# Quoting "$1" keeps filenames with spaces as a single value.
file_name="$1"

# Guard: -z tests for an empty string. If no argument was given, print usage
# to stderr (>&2) and exit with a non-zero status (failure).
if [ -z "$file_name" ]; then
    echo "Usage: $0 <file.rs>" >&2
    exit 1
fi

# Double quotes allow variable expansion inside a string; single quotes don't.
echo "Compiling $file_name"

# Parameter expansion: ${var%pattern} removes the SHORTEST match of `pattern`
# from the END of $var.  "box.rs" -> "box".
# (Related: ${var#pattern} strips from the start; %% / ## strip the longest match.)
name_without_extension="${file_name%.rs}"

# rustc won't create missing directories for its output, so make target/ first.
# -p: create parent dirs as needed and don't error if it already exists.
mkdir -p target

# Compile with the 2024 edition and write the binary to target/<name>.
rustc "$file_name" --edition=2024 -o "target/$name_without_extension"

echo "Built target/$name_without_extension"

./"target/$name_without_extension"