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
            let wellcome_message = b"------*-Wellcome to Chataro!!-*------\r\n";
            let _ = stream.lock().unwrap().write(wellcome_message);
            let user_name = std::str::from_utf8(&user_name[0..n]).unwrap().to_owned();
            {
                let mut user_streams = USER_STREAMS.write().unwrap();
                // threadIDをUserIDとStreamのタプルと結びつけ、Thread間で共有する
                user_streams.insert(thread_id, (user_name, stream));
            }
        },
        Err(why) => panic!("{:?}", why),
    }
}

fn logout(mut stream: Arc<Mutex<TcpStream>>, thread_id: u8){
    /// この関数を呼んだスレッドのTcpStreamをShutdownしてUserStreamsから削除する
    let logout_message = format!("{}が退室しました", "hoge");
    send_all(&logout_message);
    {
        let mut user_streams = USER_STREAMS.write().unwrap();
        user_streams.remove(&thread_id);
    }
    stream.lock().unwrap().shutdown(Shutdown::Both);
}

fn check_exist_user(mut stream: Arc<Mutex<TcpStream>>, thread_id: u8) {
    {
        let mut user_streams = USER_STREAMS.write().unwrap();
        let is_exist = user_streams.any(|&x| x != thread_id);
        if is_exist {
            let reminder_message = b"prease login to Chataro.\r\n";
            stream.lock().unwrap().write(reminder_message);
            login(stream.clone(), thread_id);
        }
    }
}

fn send_all(message: &str){
    /// IDSに格納されたすべてのTcpStreamにmessageをwriteする -> HashMap.valuesで値出してｳｪｲッ
    /// idの有無を確認してない場合はloginを呼ぶ
    println!("{}", message);
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
                        "LOGOUT" => logout(stream.clone(), thread_id),
                        _ => {
                            check_exist_user(stream.clone(), thread_id);
                            send_all(message);
                        },
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
