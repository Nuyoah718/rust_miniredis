#![feature(impl_trait_in_assoc_type)]
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::sync::broadcast;


use anyhow::Error;

pub const DEFAULT_ADDR: &str = "127.0.0.1:8080";

pub struct SBox {
	kv_pairs: HashMap<String, String>,
	channels: HashMap<String, broadcast::Sender<String>>,
}

pub struct S {
	sb: RwLock<SBox>,
}

impl S {
	pub fn new() -> S {
		S {
			sb: RwLock::new(SBox{kv_pairs: HashMap::new(), channels: HashMap::new()})
		}
	}
}

unsafe impl Send for S {}
unsafe impl Sync for S {}

#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
	async fn get_item(&self, _req: volo_gen::volo::example::GetItemRequest) -> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError>{
		let mut resp = volo_gen::volo::example::GetItemResponse {opcode: 0, key_channal: _req.key_channal.clone(), value_message: " ".into(), success: false};
		match _req.opcode {
			0 => {
				let key: String = _req.key_channal.into();
				match self.sb.read().unwrap().kv_pairs.get(&key) {
					Some(value) => {
						resp.opcode = 0;
						resp.value_message = value.clone().into();
						resp.success = true;
					},
					None => {
						resp.opcode = 0;
						resp.success = false;
					}
				}
			},
			1 => {
				let key: String = _req.key_channal.into();
				let val: String = _req.value_message.into();
				let mut is_in: bool = false;
				{
					match self.sb.read().unwrap().kv_pairs.get(&key) {
						Some(_) => {
							is_in = true;
						},
						None => {

						}
					}
				}
				if is_in {
					resp.opcode = 1;
					resp.success = false;
				}
				else {
					self.sb.write().unwrap().kv_pairs.insert(key, val);
					resp.opcode = 1;
					resp.success = true;
				}
			},
			2 => {
				let key: String = _req.key_channal.into();
				match self.sb.write().unwrap().kv_pairs.remove(&key) {
					Some(_v) => {
						resp.opcode = 2;
						resp.success = true;
					},
					None => {
						resp.opcode = 2;
						resp.success = false;
					}
				} 
			},
			3 => {
				resp.opcode = 3;
				resp.value_message = _req.value_message.clone();
				resp.success = true;
			},
			4 => {
				let key: String = _req.key_channal.into();
				let (mut _tx, mut rx) = broadcast::channel(16);
				let mut has_channel: bool = false;
				{
					match self.sb.read().unwrap().channels.get(&key) {
						Some(get_tx) => {
							has_channel = true;
							rx = get_tx.subscribe();
						},
						None => {
							
						},
					}
				}
				if has_channel {
					let mes = rx.recv().await;
					match mes {
						Ok(m) => {
							resp.opcode = 4;
							resp.value_message = m.clone().into();
							resp.success = true;
						},
						Err(_e) => {
							resp.opcode = 4;
							resp.success = false;
						}
					}
				} else {
					self.sb.write().unwrap().channels.insert(key, _tx);
					let mes = rx.recv().await;
					match mes {
						Ok(m) => {
							resp.opcode = 4;
							resp.value_message = m.clone().into();
							resp.success = true;
						},
						Err(_e) => {
							resp.opcode = 4;
							resp.success = false;
						}
					}
				}
			}
			5 => {
				let key: String = _req.key_channal.into();
				if let Some(tx) = self.sb.read().unwrap().channels.get(&key) {
					let info = tx.send(_req.value_message.into_string());
					match info {
						Ok(num) => {
							resp.opcode = 5;
							resp.success = true;
							resp.value_message = get_string(num as u8).into();
						},
						Err(_) => {
							resp.opcode = 5;
							resp.success = false;
						}
					}
				}
				else {
					resp.opcode = 5;
					resp.success = false;
				}
			},
			_ => {
				tracing::info!("Invalic opcode");
			},
		}
		Ok(resp)
	}
}

fn get_string(num: u8) -> String {
	let mut num: u8 = num;
	let mut res = String::new();
	let mut pow: u8 = 1;
	while pow <= num {
		pow *= 10;
	}
	pow /= 10;
	while pow != 0 {
		res.push((num / pow + '0' as u8) as char);
		num = num % pow;
		pow = pow / 10;
	}
	res
}

pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}


#[derive(Clone)]
pub struct LogService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug + From<Error>,
    Cx: Send + 'static,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let now = std::time::Instant::now();
        tracing::debug!("Received request {:?}", &req);
		// 中间件希望过滤掉kv对中涉及的不文明词汇，由于不文明词汇的判断比较复杂，此处演示过滤词汇"傻逼"
		let info: Vec<char> = format!("{req:?}").chars().collect();
		let mut can_send: bool = true;
		for i in 0..(info.len() - 1) {
			if info[i] == '傻' && info[i + 1] == '逼' {
				can_send = false;
				break;
			}
		}
		if can_send {
			let resp = self.0.call(cx, req).await;
			tracing::debug!("Sent response {:?}", &resp);
			tracing::info!("Request took {}ms", now.elapsed().as_millis());	
			return resp;
		}
		// panic!("命令中有敏感词“傻逼");
		Err(S::Error::from(Error::msg("命令中有敏感词'傻逼'")))
    }
}

