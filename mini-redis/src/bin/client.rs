use lazy_static::lazy_static;
use std::net::SocketAddr;
use std::env;
use mini_redis::LogLayer;
use std::io;
use std::io::Write;
// use volo_gen::volo::example::{GetItemResponse, get_item};
// use mini_redis::{S};

static mut ADDR_STR: String = String::new();

lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        unsafe {
            let addr: SocketAddr = ADDR_STR.parse().unwrap();
            volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
                .layer_outer(LogLayer)
                .address(addr)
                .build()
        }
    };
}

#[volo::main]
async fn main() {
    // 获取命令行参数，其为server的IP地址
    let args: Vec<String> = env::args().collect();
    unsafe { ADDR_STR = args[1].clone(); }
    tracing_subscriber::fmt::init();

    // 判断当前是否在subscribe状态，若在，则会直接进入无限循环，监听publish程序
    let mut is_subscribe: bool = false;
    let mut channel_name: String = String::new();

    loop {
        if is_subscribe {
            let subscribe_resp = CLIENT.get_item(volo_gen::volo::example::GetItemRequest { opcode: 4, key_channal: channel_name.clone().into(), value_message: " ".into() }).await;
            match subscribe_resp {
                Ok(info) => {
                    println!("{}", info.value_message);
                },
                Err(e) => tracing::error!("{:?}", e),
            }
            continue;
        }
        print!("mini-redis>  ");
        let _ = io::stdout().flush();
        // 读入传入的命令
        let mut buf: String = String::new();
        let _ = std::io::stdin().read_line(&mut buf).unwrap();
        let buf: String = buf.trim().into();
        // 将读入的命令按照空格分裂成字符串向量
        let command: Vec<String> = parse_command(&buf);
        if command.len() == 0 {
            println!("error: The command is empty");
            continue;
        }
        let mut req = volo_gen::volo::example::GetItemRequest { opcode: 0, key_channal: " ".into(), value_message: "pong".into() };
        // 判断输入的命令，设置req
        if command[0] == "exit".to_string() {
            // 退出
            println!("Goodbye!");
            break;
        }
        if command[0] == "get".to_string() {
            // get命令，则第二个参数是要搜索的key.
            req.opcode = 0;
            if command.len() < 2 {
                println!("Usage: get <key>");
                continue;
            }
            req.key_channal = command[1].clone().into();
        }
        else if command[0] == "set".to_string() {
            // set命令，则第二个参数为要设置的key，第三个参数为要设置的值
            if command.len() < 3 {
                println!("Usage: set <key> <value>");
                continue;
            }
            req.opcode = 1;
            req.key_channal = command[1].clone().into();
            req.value_message = command[2].clone().into();
        }
        else if command[0] == "del".to_string() {
            // del命令，则第二个参数为要删去的key
            if command.len() < 2 {
                println!("Usage: del <key>");
                continue;
            }
            req.opcode = 2;
            req.key_channal = command[1].clone().into();
        }
        else if command[0] == "ping".to_string() {
            // ping命令
            req.opcode = 3;
            // 要是有message要返回message
            if command.len() > 1 {
                req.value_message = command[1].clone().into();
            }
        }
        else if command[0] == "subscribe".to_string() {
            if command.len() < 2 {
                println!("Usage: subscribe <channal_name> ");
                continue;
            }
            is_subscribe = true;
            req.opcode = 4;
            req.key_channal = command[1].clone().into();
            channel_name = command[1].clone();
            println!("The message is as follow: ");
        }
        else if command[0] == "publish".to_string() {
            if command.len() < 3 {
                println!("Usage: publish <channel_name> <message>");
                continue;
            }
            req.opcode = 5;
            req.key_channal = command[1].clone().into();
            req.value_message = command[2].clone().into();
        }
        else {
            println!("Can't not find the command: {}", command[0]);
            continue;
        }

        // 将信息传递出去并得到返回的结果
        let resp = CLIENT.get_item(req).await;
        match resp {
            Ok(info) => {
                if info.opcode == 0 {
                    if info.success {
                        println!("{}", info.value_message);
                    }
                    else {
                        println!("The key: {} is not in the database", info.key_channal);
                    }
                }
                if info.opcode == 1 {
                    if info.success {
                        println!("Set success!");
                    } else {
                        println!("The key: {} is already in the database", info.key_channal);
                    }
                }
                if info.opcode == 2 {
                    if info.success {
                        println!("Del success!");
                    }
                    else {
                        println!("The key: {} is not found in the database", info.key_channal);
                    }
                }
                if info.opcode == 3 {
                    if info.success {
                        println!("{}", info.value_message.clone().to_string());
                    } else {
                        println!("The connect is fail");
                    }
                }
                if info.opcode == 4 {
                    if info.success {
                        println!("{}", info.value_message);
                    }
                    else {
                        println!("no publish");
                    }
                }
                if info.opcode == 5 {
                    // 若使用publish，则value_message会返回subcribe的数量
                    let message: String = info.value_message.clone().into();
                    let v : Vec<char> = message.chars().collect();
                    if info.success {
                        println!("publish success. The number of subscriber is {}", get_num(&v));
                    }
                    else {
                        println!("No subscriber found");
                    }
                }
            },
            Err(e) => tracing::error!("{:?}", e),
        }
    }
}

fn parse_command(buf: &String) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let v1: Vec<&str> = buf.split(" ").collect();
    for s in v1 {
        v.push(s.into());
    }
    v
}

fn get_num(v: &Vec<char>) -> i32 {
    let mut index = 0;
    let mut res = 0;
    while index < v.len() {
        res = res * 10 + (v[index] as i32 - '0' as i32);
        index += 1;
    }
    res
}
