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

// TODO: thread_idをthread_localで管理したほうが良い

/// TcpStreamからの入力ストリームを結合しStringとして返す
fn read_stream(mut stream: Arc<Mutex<TcpStream>>) -> String {
    let mut message = String::new();
    while {
        let mut buffer = [0;1];
        let n = stream.lock().unwrap().read(&mut buffer).unwrap();
        let response = std::str::from_utf8(&buffer).unwrap();
        message.push_str(response);
        println!("{:?}, {:?}", response, message);
        response != "\n"    // TODO: これだと\nが残って悲惨なことになる
    } {}
    message
}

/// ユーザーがログインしているとき真、ログインしていなければ偽を返す
fn exist_user(thread_id: u8) -> bool{
    let mut user_streams = USER_STREAMS.read().unwrap();
    // ある要素がこのスレッドのthread_idと等しいならばそのユーザーはログインしている
    user_streams.keys().any(|&x| x == thread_id)
}
    
/// USER_STREAMSにthreadIDとUser_name, TcpStreamを保存する
fn login(mut stream: Arc<Mutex<TcpStream>>, thread_id: u8){
    if exist_user(thread_id) {
        let logined_message = b"you already login this chat!\r\n";
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
fn logout(mut stream: Arc<Mutex<TcpStream>>, thread_id: u8){
    let logout_message = "-*-system_message::退室しました-*-";
    send_all(logout_message, thread_id);
    {
        let mut user_streams = USER_STREAMS.write().unwrap();
        user_streams.remove(&thread_id);
    }
    stream.lock().unwrap().shutdown(Shutdown::Both);
}

/// IDSに格納されたすべてのTcpStreamにmessageをwriteする -> HashMap.valuesで値出してｳｪｲッ
/// idの有無を確認してない場合はloginを呼ぶ
fn send_all(message: &str, thread_id: u8){
    let mut user_streams = USER_STREAMS.read().unwrap();
    let (ref sendbyname, _): (String, _) = *user_streams.get(&thread_id).unwrap();
    let send_message = sendbyname.split("\r\n").next().unwrap().to_string() + ": " + message;

    println!("{}", send_message);
}

/// クライアントから送られてきたデータを受信して、対応する関数を呼ぶ
fn handle_client(mut stream: TcpStream, thread_id: u8) {
    let stream = Arc::new(Mutex::new(stream));
    loop{
        let mut message = read_stream(stream.clone());
        // let message = std::str::from_utf8(&buffer[0..n]).unwrap().split("\r\n").next().unwrap();
        match message.as_str() {
            "LOGIN\r\n" => login(stream.clone(), thread_id),
            "LOGOUT\r\n" => {
                logout(stream.clone(), thread_id);
                break;
            },
            _ => {
                if exist_user(thread_id) {
                    send_all(message.as_str(), thread_id);
                } else {
                    let reminder_message = b"prease login to Chataro.\r\n";
                    stream.lock().unwrap().write(reminder_message);
                    login(stream.clone(), thread_id);
                }
            },
        }
        /*
        match res {
            Ok(n) => {
                if 0 < n {
                    let message = std::str::from_utf8(&buffer[0..n]).unwrap().split("\r\n").next().unwrap();
                    match message {
                        "LOGIN" => login(stream.clone(), thread_id),
                        "LOGOUT" => logout(stream.clone(), thread_id),
                        _ => {
                            if exist_user(thread_id) {
                                send_all(message, thread_id);
                            } else {
                                let reminder_message = b"prease login to Chataro.\r\n";
                                stream.lock().unwrap().write(reminder_message);
                                login(stream.clone(), thread_id);
                            }
                        },
                    }
                }
            },
            Err(why) => panic!("{:?}", why),
        }*/
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
