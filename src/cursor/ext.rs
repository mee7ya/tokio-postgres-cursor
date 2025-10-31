use tokio_postgres::{Error, Transaction};

use crate::cursor::stream::CursorStream;

pub trait TransactionExt {
    fn query_cursor<'a>(
        self,
        query: &str,
        batch_size: usize,
    ) -> impl std::future::Future<Output = Result<CursorStream<'a>, Error>> + Send
    where
        Self: 'a;
}

impl<'t> TransactionExt for Transaction<'t> {
    async fn query_cursor<'a>(
        self,
        query: &str,
        batch_size: usize,
    ) -> Result<CursorStream<'a>, Error>
    where
        Self: 'a,
    {
        CursorStream::new(self, query, batch_size).await
    }
}
