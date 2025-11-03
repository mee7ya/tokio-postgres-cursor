use tokio_postgres::{Error, Transaction};

use crate::cursor::stream::CursorStream;

/// Extension trait for 
/// [`Transaction`](https://docs.rs/tokio-postgres/latest/tokio_postgres/struct.Transaction.html) 
/// to add cursor support.
pub trait TransactionExt {
    /// Method to create a new [`CursorStream`] for the given query.
    /// 
    /// Parameters:
    /// - `query`: The SQL query for which the cursor will be declared.
    /// - `batch_size`: The number of rows to fetch in each batch.
    fn query_cursor<'a>(
        &'a self,
        query: &str,
        batch_size: usize,
    ) -> impl std::future::Future<Output = Result<CursorStream<'a>, Error>> + Send
    where
        Self: 'a;
}

/// Implementation of [`TransactionExt`] for 
/// [`Transaction`](https://docs.rs/tokio-postgres/latest/tokio_postgres/struct.Transaction.html)
impl<'t> TransactionExt for Transaction<'t> {
    /// Method to create a new [`CursorStream`] for the given query.
    /// 
    /// Parameters:
    /// - `query`: The SQL query for which the cursor will be declared.
    /// - `batch_size`: The number of rows to fetch in each batch.
    /// 
    /// Errors:
    /// - Propagates 
    /// [`tokio_postgres::Error`](https://docs.rs/tokio-postgres/latest/tokio_postgres/error/struct.Error.html) 
    /// if the cursor declaration fails
    async fn query_cursor<'a>(
        &'a self,
        query: &str,
        batch_size: usize,
    ) -> Result<CursorStream<'a>, Error>
    where
        Self: 'a,
    {
        CursorStream::new(self, query, batch_size).await
    }
}
