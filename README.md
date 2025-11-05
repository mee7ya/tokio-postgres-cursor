# tokio-postgres-cursor
PostgreSQL forward fetch cursor support for [tokio-postgres](https://github.com/rust-postgres/rust-postgres).

## Example
```rust
use futures::StreamExt;

use tokio_postgres::{Error, NoTls};
use tokio_postgres_cursor::TransactionExt;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (mut client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Cursors require to be declared inside a transaction
    let tx = client.transaction().await?;

    // Following line will declare cursor inside transaction and return CursorStream
    let mut cursor_stream = tx.query_cursor("SELECT * FROM my_table", 10).await?;
    
    // Fetch rows in batches of 10
    while let Some(result) = cursor_stream.next().await {
        match result {
            Ok(rows) => {
                for row in rows {
                    println!("{row:?}");
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }
    
    // Explicitly closing cursor
    cursor_stream.close().await?;
    tx.commit().await?;
    Ok(())
}
```
