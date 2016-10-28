use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

#[macro_use]
extern crate lazy_static;


lazy_static! {
    // 各スレッドのTcpStreamをスレッド間で共有する
    // thread_idと(name, TcpStream)を括り付ける
    #[derive(Debug)]
    static ref USER_STREAMS: RwLock<HashMap<u8, (String, Arc<Mutex<TcpStream>>)>> = {
        let user_streams = HashMap::new();
        RwLock::new(user_streams)
    };
}

fn login(mut stream: Arc<Mutex<TcpStream>>, thread_id: u8){
    /// USER_STREAMSにthreadIDとUser_name, TcpStreamを保存する
    let banner = b"Input your UserID: ";
    let _ = stream.lock().unwrap().write(banner);
    let mut user_name = [0;128];
    let res = stream.lock().unwrap().read(&mut user_name);
    match res {
        Ok(n) => {
            let user_name = std::str::from_utf8(&user_name[0..n]).unwrap().to_owned();
            {
                let mut user_streams = USER_STREAMS.write().unwrap();
                // threadIDをUserIDとStreamのタプルと結びつけ、Thread間で共有する
                user_streams.insert(thread_id, (user_name, stream));
            }
            println!("{:?}", USER_STREAMS);
        },
        Err(why) => panic!("{:?}", why),
    }
}
fn logout(mut stream: Arc<Mutex<TcpStream>>){
    /// この関数を呼んだスレッドのTcpStreamをShutdownしてIDSから削除する
    stream.lock().unwrap().shutdown(Shutdown::Both);
    println!("insert logout");
}
fn send_all(message: &str){
    /// IDSに格納されたすべてのTcpStreamにmessageをwriteする
    /// idの有無を確認してない場合はloginを呼ぶ
    println!("insert send_all");
}


fn handle_client(mut stream: TcpStream, thread_id: u8) {
    /// クライアントから送られてきたデータを受信して、対応する関数を呼ぶ
    let stream = Arc::new(Mutex::new(stream));
    loop{
        let mut buffer = [0;1024];
        let res = stream.lock().unwrap().read(&mut buffer);
        match res {
            Ok(n) => {
                if 0 < n {
                    // 受信したデータを&str型に変換
                    let message = std::str::from_utf8(&buffer[0..n]).unwrap().split("\r\n").next().unwrap();
                    // \r\nで分割してmethodを取り出す
                    match message {
                        "LOGIN" => login(stream.clone(), thread_id),
                        "LOGOUT" => logout(stream.clone()),
                        _ => send_all(message),
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
        println!("Accepted");
        match stream {
            // incoming()がOk返したらhandle_clientのスレッドを生成
            Ok(stream) => {
                let mut stream = stream;
                thread::spawn(move|| {
                    handle_client(stream, thread_id)
                });
                thread_id += 1;
            }
            // Errが帰ってきたら頑張る
            Err(e) => { println!("Error: {:?}", e); }
        }
    }
    // ソケットを閉じる
    drop(listener);
}
