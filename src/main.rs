use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::mpsc;
use std::io::BufReader;
use std::thread;

fn main() {
    // 打印日志
    println!("program initializing");
    println!("Ctrl+C to exit");

    // 创建一个通道，包含了发送及接收
    let (sender, receiver) = mpsc::channel();

    // 复制一个接收者用于侦听处理
    let s = sender.clone();

    // 新开一个线程
    thread::spawn(move || { listen(s); });

    // 统计连接数量
    let mut conection_count = 0;

    // 新开一个无限循环
    loop {
        // 错误处理
        match receiver.recv() {
            Ok(signal) => {
                // 模式匹配接收信息
                match signal {
                    1 => {
                        // 增加连接数量
                        conection_count += 1;
                    },
                    2 => {
                        // 退出
                        break;
                    },
                    _  => {
                        // 打印错误
                        println!("invalid signal recevied: {}", signal)
                    }
                }
            }
            Err(e) => {
                // 打印错误信息
                println!("Receive error, {}", e)
            }
        }
    }

    // 打印连接总数及退出提示
    println!("Total connections: {}", conection_count);
    println!("exit");
}

// 侦听请求
fn listen(sender: mpsc::Sender<u32>) {
    // 绑定侦听地址
    let listener = TcpListener::bind("127.0.0.1:9991").unwrap();

    for stream in listener.incoming() {
        // 模式匹配，处理请求
        match stream {
            Ok(stream) => {
                // 打印请求客户端地址信息
                println!("Received a connection! {}:{}", stream.peer_addr().unwrap().ip(), stream.peer_addr().unwrap().port());
                let txp = sender.clone();

                // 新开一个线程处理客户端请求
                thread::spawn(move || {
                    connect_handler(stream, txp);
                });

                // 响应信息
                sender.send(1).unwrap();
            }
            Err(e) => {
                println!("listen incoming error, {}", e.to_string());
            }
        }
    }

    drop(listener);
}

// 处理连接
fn connect_handler(stream: TcpStream, sender: mpsc::Sender<u32>) {
    let mut buf = BufReader::new(stream.try_clone().unwrap());

    // 无限循环,处理客户端请求
    loop {
        // 声明一个字符串，用于接收数据
        let mut s = String::new();

        // 读取请求数据并错误处理
        match buf.read_line(&mut s) {
            Ok(b) => {
                // 如果没有数据则不处理
                if b == 0 {
                    break;
                }

                // 打印数据长度
                print!("Received data ({} bytes): {}", b, s);

                // 检测是否是退出命令
                if s.contains("quit") {
                    // 响应信息
                    sender.send(2).unwrap();
                    break;
                }
            }
            Err(e) => {
                // 打印错误信息
                println!("Receive data error, {}", e)
            }
        }
    }

    // 打印退出信息
    println!("Client {}:{} exit", stream.peer_addr().unwrap().ip(), stream.peer_addr().unwrap().port());

    // 关闭连接, 同时关闭发送与接收
    stream.shutdown(Shutdown::Both).unwrap();
}