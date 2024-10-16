use core::str;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Connecting to server...");

    let tcp_stream: TcpStream = TcpStream::connect("127.0.0.1:1025").await.unwrap();

    println!("connected!");

    // 0.読み取り/書き込みで別のstreamに分割
    let (mut reader, mut writer) = tcp_stream.into_split();

    // 1.tokioで別スレッドを作成
    let reading_task = tokio::task::spawn(async move {
        let mut buffer = [0; 1024 * 4];
        loop {
            // 2.SMTPサーバーからデータが送られると`buffer`にデータを代入、データが送られるまではずっとハングする
            // 5."3"でコネクションがcloseされたため、0が返却される
            let bytes_read = reader.read(&mut buffer).await.unwrap();
            dbg!(bytes_read);
            let response = str::from_utf8(&buffer[..bytes_read]).unwrap();
            dbg!(response);
            if bytes_read == 0 {
                break;
            }
        }
    });

    writer
        .write("EHLO example.com\r\n".as_bytes())
        .await
        .unwrap();
    println!("send EHLO");

    writer
        .write("MAIL FROM:<hogehoge@example.com>\r\n".as_bytes())
        .await
        .unwrap();
    println!("send MAIL FROM");

    writer
        .write("RCPT TO:<alice@foo.com>\r\n".as_bytes())
        .await
        .unwrap();
    println!("send RCPT TO");

    writer.write("DATA\r\n".as_bytes()).await.unwrap();

    let body = vec![
        "From: Bob Example <bob@example.com>",
        "To: Alice Example <alice@foo.com>",
        "Date: Tue, 15 Jan 2008 16:02:43 -0500",
        "Subject: Test message",
        "",
        "Hello Alice",
        ".",
        "",
    ]
    .join("\r\n");
    writer.write(body.as_bytes()).await.unwrap();
    println!("send DATA");

    writer.write("QUIT".as_bytes()).await.unwrap();
    println!("send QUIT");

    // 3.書き込み側を閉じる
    writer.shutdown().await.unwrap();

    // 4.別スレッドの終了を待つ（こうしないとログ出力の前にプログラムが終了する）
    reading_task.await.unwrap();

    Ok(())
}
