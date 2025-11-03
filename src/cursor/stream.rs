use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
};

use rand::{
    distr::{Alphanumeric, Distribution},
    rng,
};

use futures_core::Stream;
use tokio::runtime::Handle;
use tokio_postgres::{Error, Row, Transaction};

/// A stream that fetches rows from a PostgreSQL cursor in batches.
pub struct CursorStream<'a> {
    tx: Arc<&'a Transaction<'a>>,
    cursor: Arc<String>,
    batch_size: usize,
    future: Option<Pin<Box<dyn Future<Output = Result<Vec<Row>, Error>> + Send + 'a>>>,
    done: bool,
}

impl<'a> CursorStream<'a> {
    /// Creates a new [`CursorStream`] and declares a cursor for the given query.
    /// 
    /// Parameters:
    /// - `tx`: A reference to the transaction in which the cursor will be declared.
    /// - `query`: The SQL query for which the cursor will be declared.
    /// - `batch_size`: The number of rows to fetch in each batch.
    /// 
    /// Errors:
    /// - Propagates 
    /// [`tokio_postgres::Error`](https://docs.rs/tokio-postgres/latest/tokio_postgres/error/struct.Error.html) 
    /// if the cursor declaration fails.
    pub(crate) async fn new(
        tx: &'a Transaction<'a>,
        query: &str,
        batch_size: usize,
    ) -> Result<Self, Error> {
        let cursor = format!(
            "cursor_{}",
            Alphanumeric
                .sample_iter(rng())
                .take(3)
                .map(|x| x as char)
                .collect::<String>()
        );
        tx.execute(
            &format!("DECLARE {} NO SCROLL CURSOR FOR {}", cursor, query),
            &[],
        )
        .await?;

        Ok(Self {
            tx: Arc::new(tx),
            cursor: Arc::new(cursor),
            batch_size,
            future: None,
            done: false,
        })
    }

    /// Closes the cursor associated with this stream.
    /// 
    /// Errors:
    /// - Propagates 
    /// [`tokio_postgres::Error`](https://docs.rs/tokio-postgres/latest/tokio_postgres/error/struct.Error.html) 
    /// if the cursor closing fails
    pub async fn close(mut self) -> Result<u64, Error> {
        self.done = true;
        self.tx
            .execute(&format!("CLOSE {}", self.cursor), &[])
            .await
    }
}

impl<'a> Stream for CursorStream<'a> {
    type Item = Result<Vec<Row>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        if self.future.is_none() {
            let tx = Arc::clone(&self.tx);
            let cursor = Arc::clone(&self.cursor);
            let batch_size = self.batch_size;

            let future = Box::pin(async move {
                tx.query(
                    &format!("FETCH FORWARD {} FROM {}", batch_size, cursor),
                    &[],
                )
                .await
            });

            self.future = Some(future);
        }

        match ready!(self.future.as_mut().unwrap().as_mut().poll(cx)) {
            Ok(rows) => {
                self.future = None;
                if rows.is_empty() {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(rows)))
                }
            }
            Err(e) => {
                self.future = None;
                Poll::Ready(Some(Err(e)))
            }
        }
    }
}

impl Drop for CursorStream<'_> {
    fn drop(&mut self) {
        if !self.done {
            Handle::current().block_on(async {
                let _ = self
                    .tx
                    .execute(&format!("CLOSE {}", self.cursor), &[])
                    .await;
            });
        }
    }
}
