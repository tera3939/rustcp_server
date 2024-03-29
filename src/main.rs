use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    // 各スレッドのTcpStreamをスレッド間で共有する
    // thread_idと(UserName, TcpStream)を括り付ける
    #[derive(Debug)]
    static ref USER_STREAMS: RwLock<HashMap<u8, (String, Arc<Mutex<TcpStream>>)>> = {
        let user_streams = HashMap::new();
        RwLock::new(user_streams)
    };
}

// TODO: thread_idをthread_localで管理したほうが良い

/// TcpStreamからの入力ストリームを結合しStringとして返す
fn read_stream(stream: Arc<Mutex<TcpStream>>) -> String {
    let mut message = String::new();
    while {
        let mut buffer = [0;1];
        let _ = stream.lock().unwrap().read(&mut buffer).unwrap();
        let response = std::str::from_utf8(&buffer).unwrap();
        message.push_str(response);
        response != "\n"
    } {}
    message
}

/// ユーザーがログインしているとき真、ログインしていなければ偽を返す
fn exist_user(thread_id: u8) -> bool{
    let user_streams = USER_STREAMS.read().unwrap();
    // ある要素がこのスレッドのthread_idと等しいならばそのユーザーはログインしている
    user_streams.keys().any(|&x| x == thread_id)
}
    
/// USER_STREAMSにthreadIDとUser_name, TcpStreamを保存する
fn login(stream: Arc<Mutex<TcpStream>>, thread_id: u8){
    if exist_user(thread_id) {
        let logined_message = b"-*-system_message::you already login this chat!-*-\r\n";
        let _ = stream.lock().unwrap().write(logined_message);
        return;
    }
    let banner = b"Input your UserID: ";
    let _ = stream.lock().unwrap().write(banner);
    let user_name = read_stream(stream.clone());
    let wellcome_message = b"------*-Wellcome to Chataro!!-*------\r\n";
    let _ = stream.lock().unwrap().write(wellcome_message);
    {
        let mut user_streams = USER_STREAMS.write().unwrap();
        // threadIDをUserIDとStreamのタプルと結びつけ、Thread間で共有する
        user_streams.insert(thread_id, (user_name, stream));
    }
}

/// この関数を呼んだスレッドのTcpStreamをShutdownしてUserStreamsから削除する
fn logout(stream: Arc<Mutex<TcpStream>>, thread_id: u8){
    let logout_message = "-*-system_message::this user logouted-*-\r\n";
    send_all(logout_message, thread_id);
    {
        let mut user_streams = USER_STREAMS.write().unwrap();
        user_streams.remove(&thread_id);
    }
    let _ = stream.lock().unwrap().shutdown(Shutdown::Both);
}

/// IDSに格納されたすべてのTcpStreamにmessageをwriteする -> HashMap.valuesで値出してｳｪｲッ
/// idの有無を確認してない場合はloginを呼ぶ
fn send_all(message: &str, thread_id: u8){
    let user_streams = USER_STREAMS.read().unwrap();
    let (ref sendbyname, _): (String, _) = *user_streams.get(&thread_id).unwrap();
    let send_message = sendbyname.split("\r\n").next().unwrap().to_string() + ": " + message;
    // TODO: spmcで各スレッドに送信したい
    for &(_, ref stream) in user_streams.values() {
        let _ = stream.lock().unwrap().write(send_message.as_bytes());
    }
    println!("send_message: {}", send_message);
}

/// クライアントから送られてきたデータを受信して、対応する関数を呼ぶ
fn handle_client(stream: TcpStream, thread_id: u8) {
    let stream = Arc::new(Mutex::new(stream));
    loop{
        let message = read_stream(stream.clone());
        match message.as_str() {
            "LOGIN\r\n" | "Login\r\n" | "login\r\n" => login(stream.clone(), thread_id),
            "LOGOUT\r\n" | "Logout\r\n" | "logout\r\n" => {
                logout(stream.clone(), thread_id);
                break;
            },
            _ => {
                if exist_user(thread_id) {
                    send_all(message.as_str(), thread_id);
                } else {
                    let reminder_message = b"-*-system_message::prease login to Chataro.-*-\r\n";
                    let _ = stream.lock().unwrap().write(reminder_message);
                    login(stream.clone(), thread_id);
                }
            },
        }
    }
}

fn main() {
    // ソケットを生成
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    // TODO: thread_idをハッシュにする
    let mut thread_id = 0;
    println!("Start Listenig...");
    // accept()してるっぽい
    for stream in listener.incoming() {
        println!("Accepted");
        match stream {
            // incoming()がOk返したらhandle_clientのスレッドを生成
            Ok(stream) => {
                let stream = stream;
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
