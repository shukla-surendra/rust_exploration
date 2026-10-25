use rust_decimal::Decimal;

/// A row to insert. `id` is left out because the database generates it
/// (`GENERATED ALWAYS AS IDENTITY`).
pub struct NewEmployee {
    pub name: String,
    pub age: Option<i32>, // column is nullable -> Option
    pub salary: Decimal,  // NUMERIC(12, 2) -> Decimal, never f64 for money
    pub department: String,
}
