use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0;128];
    println!("{:p}", &stream);
    match stream.read(&mut buffer) {
        Ok(n) => {
            if 0 < n {
                
            }
        },
    }
    if buffer == "ENTER\r\n".as_bytes() {
        println!("{:?}", buffer);
        let banner = b"prease input your ID";
        println!("{:?}", banner);
        stream.write(banner);
    };
}

fn main() {
    // ソケットを生成
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    println!("Start Listenig...");
    // accept()してるっぽい
    for stream in listener.incoming() {
        println!("Accepted!");
        match stream {
            // incoming()がOk返したらhandle_clientのスレッドを生成
            Ok(mut stream) => {
                println!("{:p}", &stream);
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            // Errが帰ってきたら頑張る
            Err(e) => { println!("Error: {:?}", e); }
        }
    }
    // ソケットを閉じる
    drop(listener);
}
