//! [`tokio-postgres`](https://docs.rs/tokio-postgres) extension to support forward fetching cursors.
//!
//! # Example
//! ```no_run
//! use tokio_postgres::{NoTls, Error};
//! use tokio_postgres_cursor::TransactionExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!    let (mut client, connection) = 
//!         tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
//! 
//!    // Cursors require to be declared inside a transaction
//!    let tx = client.transaction().await?;
//! 
//!    // Following line will declare cursor inside transaction and return CursorStream
//!    let cursor_stream = tx.query_cursor("SELECT * FROM my_table", 10).await?;
//! 
//!    // Fetch rows in batches of 10
//!    while let Some(rows) = cursor_stream.next().await? {
//!        for row in rows {
//!           println!("{row:?}");
//!        }
//!    }
//! 
//!   // Explicitly closing cursor is highly recommended
//!   cursor_stream.close().await?;
//!   tx.commit().await?;
//! }
//! ```
//! 
//! # Queries
//! [`query_cursor`](crate::cursor::ext::TransactionExt::query_cursor) will create 
//! a new instance of [`CursorStream`]
//! which will execute the following query to declare a cursor:
//! ```
//! DECLARE <cursor> NO SCROLL CURSOR FOR <query>
//! ```
//! 
//! [`CursorStream`] implements 
//! [`Stream`](https://docs.rs/futures/latest/futures/stream/trait.Stream.html) trait 
//! and will execute the following query to fetch rows in batches:
//! ```
//! FETCH FORWARD <batch_size> FROM <cursor>
//! ```
//! Cursor won't be closed automatically when the stream is exhausted,
//! subsequent polls of [`CursorStream`] will return [`None`].
//! 
//! It is highly recommended to explicitly close the cursor when it is no longer needed
//! by calling [`close`](crate::cursor::stream::CursorStream::close).
//! Even though the cursor will be closed automatically when it goes out of scope,
//! explicit closing can do it in asynchronous manner.
//! 
//! # Vulnerabilities
//! It's up to the user to ensure that the query passed to
//! [`query_cursor`](crate::cursor::ext::TransactionExt::query_cursor) is safe
//! from SQL injection vulnerabilities.

mod cursor;

pub use crate::cursor::ext::TransactionExt;
pub use crate::cursor::stream::CursorStream;
