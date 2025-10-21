use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let mut conn = Connection::open("cats.db")?;

    // Create the table if it doesn't exist
    create_table(&conn)?;

    successful_tx(&mut conn)?;

    let res = rolled_back_tx(&mut conn);
    assert!(res.is_err());

    let _ = print_colors(&mut conn);

    Ok(())
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cat_colors (
            id     INTEGER PRIMARY KEY AUTOINCREMENT,
            name   TEXT NOT NULL UNIQUE
        )",
        [],
    )?;
    Ok(())
}

fn successful_tx(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute("delete from cat_colors", [])?;
    tx.execute("insert into cat_colors (name) values (?1)", ["lavender"])?;
    tx.execute("insert into cat_colors (name) values (?1)", ["blue"])?;

    tx.commit()
}

fn rolled_back_tx(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute("delete from cat_colors", [])?;
    tx.execute("insert into cat_colors (name) values (?1)", ["lavender"])?;
    tx.execute("insert into cat_colors (name) values (?1)", ["blue"])?;
    tx.execute("insert into cat_colors (name) values (?1)", ["lavender"])?;

    tx.commit()
}


fn fetch_colors(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM cat_colors ORDER BY id")?;
    let rows = stmt.query_map([], |row| row.get(0))?;

    let mut colors = Vec::new();
    for color in rows {
        colors.push(color?);
    }

    Ok(colors)
}

fn print_colors(conn: &Connection) -> Result<()> {
    let colors = fetch_colors(conn)?;
    if colors.is_empty() {
        println!("No colors in table.");
    } else {
        println!("Colors in table: {:?}", colors);
    }
    Ok(())
}