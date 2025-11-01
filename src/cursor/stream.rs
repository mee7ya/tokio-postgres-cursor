use std::{
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures_core::Stream;
use tokio_postgres::{Error, Row, Transaction};

pub struct CursorStream<'a> {
    tx: Rc<&'a Transaction<'a>>,
    cursor: Rc<String>,
    batch_size: usize,
    future: Option<Pin<Box<dyn Future<Output = Result<Vec<Row>, Error>> + 'a>>>,
}

impl<'a> CursorStream<'a> {
    pub(crate) async fn new(
        tx: &'a Transaction<'a>,
        query: &str,
        batch_size: usize,
    ) -> Result<Self, Error> {
        let cursor = "purrrsor".to_string();
        tx.execute(&format!("DECLARE {} CURSOR FOR {}", cursor, query), &[])
            .await?;
        Ok(Self {
            tx: Rc::new(tx),
            cursor: Rc::new(cursor),
            batch_size,
            future: None,
        })
    }
}

impl<'a> Stream for CursorStream<'a> {
    type Item = Result<Vec<Row>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.future.is_none() {
            let tx = Rc::clone(&self.tx);
            let cursor = Rc::clone(&self.cursor);
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

        match self.future.as_mut().unwrap().as_mut().poll(cx) {
            Poll::Ready(Ok(rows)) => {
                self.future = None;
                if rows.is_empty() {
                    let tx = Rc::clone(&self.tx);
                    let cursor = Rc::clone(&self.cursor);
                    let _ = Box::pin(
                        async move { tx.execute(&format!("CLOSE {}", cursor), &[]).await },
                    )
                    .as_mut()
                    .poll(cx);
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(Ok(rows)))
                }
            }
            Poll::Ready(Err(e)) => {
                self.future = None;
                Poll::Ready(Some(Err(e)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
