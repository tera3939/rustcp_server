use std::cell::RefCell;
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::RwLock;
use std::thread;

#[macro_use]
extern crate lazy_static;

// HashMapにミュータブルなTcpStreamを格納したい or 格納したTcpStreamを使うときだけミュータブルにしたい
lazy_static! {
   static ref IDS: RwLock<HashMap<&'static str, TcpStream>> = {
       let ids = HashMap::new();
       RwLock::new(ids)
   };
}


fn login(stream: &mut TcpStream) {
    let banner = b"Input your UserID: ";
    let _ = stream.write(banner);
    let mut id = [0;128];
    match stream.read(&mut id) {
        Ok(n) => {
            let id = std::str::from_utf8(&id[0..n]).unwrap();   // `id` does not live long enough
            let mut ids = IDS.write().unwrap();
            // UserIDとソケットを結びつけ、Thread間で共有する
            ids.insert(id, *stream);                            // cannot move out of borrowed content
        },
        Err(why) => panic!("{:?}", why),
    }
}

fn logout(){
    println!("insert logout");
}

fn send_all(){
    println!("insert send_all");
}

fn handle_client(mut stream: TcpStream, counter: u8) {
    /// クライアントから送られてきたデータを受信して、methodに対応する関数を呼ぶ
    /// protocol: method CRLF value
    loop{
        let mut buffer = [0;128];
        println!("{:p}", &stream);
        match stream.read(&mut buffer) {
            Ok(n) => {
                if 0 < n {
                    // 受信したデータを&str型に変換
                    let method = std::str::from_utf8(&buffer[0..n]).unwrap();
                    println!("{:?}", method.split("\r\n").next().unwrap());
                    // \r\nで分割してmethodを取り出す
                    match method.split("\r\n").next().unwrap() {
                        "LOGIN" => login(&mut stream),
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
    let mut thread_id = 0;
    println!("Start Listenig...");
    // accept()してるっぽい
    for stream in listener.incoming() {
        println!("Accepted!");
        match stream {
            // incoming()がOk返したらhandle_clientのスレッドを生成
            Ok(stream) => {
                let mut stream = stream;
                thread_id += 1;
                println!("{:p}", &stream);
                thread::spawn(move|| {
                    handle_client(stream, thread_id)
                });
            }
            // Errが帰ってきたら頑張る
            Err(e) => { println!("Error: {:?}", e); }
        }
    }
    // ソケットを閉じる
    drop(listener);
}
