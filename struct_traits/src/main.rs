// ✅ With structs in Rust you can:
// - Define custom data types with named fields (like classes without inheritance).
// - Store different types of data together.
// - Create instances using `StructName { field: value, ... }`.
// - Provide shorthand with `..` to fill remaining fields.
// - Access fields directly with `.` (dot notation).
// - Make fields mutable with `mut`.
// - Implement methods using `impl` blocks.
// - Create associated functions (like constructors).
// - Derive traits (e.g., Debug, Clone, PartialEq) for comparison/printing.
// - Use tuple structs or unit-like structs for lightweight definitions.
// - Pattern match on structs to destructure fields.
// - Combine with `enum` for more expressive data models.
// - Encapsulate logic by making fields private (`pub` controls visibility).

mod struct_module;

use struct_module :: {basic_struct, impl_struct};


fn main() {
    basic_struct:: main_test();
    impl_struct:: main_test();
}
