use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn login() {
    let barner = b"Input your userID";
    

}

fn logout(){
    println!("insert logout");
}

fn send_all(){
    println!("insert send_all");
}

fn handle_client(mut stream: TcpStream, counter: u8) {
    // クライアントから送られてきたデータを受信
    loop{
        let mut buffer = [0;128];
        println!("{:p}", &stream);
        // 受信できたか確認
        match stream.read(&mut buffer) {
            Ok(n) => {
                if 0 < n {
                    // プロトコルを確認
                    let method = std::str::from_utf8(&buffer[0..n]).unwrap();
                    println!("{:?}", method.split("\r\n").next().unwrap());
                    match method.split("\r\n").next().unwrap() {
                        "LOGIN" => login(),
                        "LOGOUT" => logout(),
                        "CAHT" => send_all(),
                        _ => println!("hoge:{}", counter),
                    }
                }
            },
            Err(why) => panic!("{:?}", why),
        }
    }
}

fn main() {
    // ソケットを生成
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let mut counter = 0;
    println!("Start Listenig...");
    // accept()してるっぽい
    for stream in listener.incoming() {
        println!("Accepted!");
        match stream {
            // incoming()がOk返したらhandle_clientのスレッドを生成
            Ok(mut stream) => {
                counter += 1;
                println!("{:p}", &stream);
                thread::spawn(move|| {
                    handle_client(stream, counter)
                });
            }
            // Errが帰ってきたら頑張る
            Err(e) => { println!("Error: {:?}", e); }
        }
    }
    // ソケットを閉じる
    drop(listener);
}
