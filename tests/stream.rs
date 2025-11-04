use futures::StreamExt;
use tokio_postgres::NoTls;
use tokio_postgres_cursor::TransactionExt;

#[tokio::test]
async fn test_cursor_stream() {
    let (mut client, connection) =
        tokio_postgres::connect("host=localhost user=postgres dbname=test_db", NoTls)
            .await
            .unwrap();

    tokio::spawn(async move {
        connection.await.unwrap();
    });

    let tx = client.transaction().await.unwrap();
    let mut cursor_stream = tx
        .query_cursor("SELECT generate_series(1, 25)", 10)
        .await
        .unwrap();

    let mut total_rows = 0;
    let mut entered: u8 = 0;
    while let Some(result) = cursor_stream.next().await {
        match result {
            Ok(rows) => {
                total_rows += rows.len();
                entered += 1;
            }
            Err(e) => panic!("Error fetching rows: {}", e),
        }
    }

    assert_eq!(total_rows, 25);
    assert_eq!(entered, 3);

    cursor_stream.close().await.unwrap();
    tx.commit().await.unwrap();
}
