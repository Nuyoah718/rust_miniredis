# mini-redis

本次实验实现了一个mini-redis的客户端和服务端，支持键和值的类型都是字符串的kv数据库，其支持`get, set, del, ping, subscribe, publish`几种方法。并使用中间件对命令进行过滤，本次实验中为敏感词检测，检测到敏感词“傻逼”后会阻止命令执行并返回错误信息。信息格式在readme末尾
## 编译运行的方法

在工程目录下运行如下命令编译
```
cargo build
```
由于实验时为本机跑，所以服务端ip地址默认为"127.0.0.1:8080"，若需要多机运行可以在代码中修改，使用如下命令运行服务端
```
cargo run --bin server
```

此时服务端进入运行状态

使用如下代码运行客户端
```
cargo run --bin client 127.0.0.1:8080
```

运行时携带了命令行参数，其为要连接的服务端的ip地址。运行后服务端会进入一个交互端口，会出现以下提示符，直接在其后输入命令即可
```
mini-redis> 
```

## get指令

其使用格式为
```
get <key>
```

若存在，则会返回其对应的值，例如
```s
mini-redis>  get 123
2023-09-11T16:34:49.176474Z  INFO mini_redis: Request took 1ms
456
```

不存在则会报错

```s
mini-redis>  get 3
2023-09-11T16:40:45.471091Z  INFO mini_redis: Request took 2ms
The key: 3 is not in the database
```

## set指令

其使用格式为
```
set <key> <value>
```

若数据库中原来不存在键值，则插入成功
```s
mini-redis>  set 456 789
2023-09-11T16:42:09.488153Z  INFO mini_redis: Request took 1ms
Set success!
```

若已经存在键，则插入失败
```s
mini-redis>  set 456 7
2023-09-11T16:42:42.501647Z  INFO mini_redis: Request took 1ms
The key: 456 is already in the database
```

## del指令

其使用格式为
```
del <key>
```

若成功删除，则会有如下信息
```s
mini-redis>  del 456
2023-09-11T16:44:18.675583Z  INFO mini_redis: Request took 1ms
Del success!
```

若不存在键，则输出如下信息
```s
mini-redis>  del 456
2023-09-11T16:45:01.661540Z  INFO mini_redis: Request took 1ms
The key: 456 is not found in the database
```

## ping

用法
```
ping [message]
```

若连接成功，且没有指定输出内容，则输出"pong"，若连接已经失效，则直接报error
```s
mini-redis>  ping    
2023-09-11T16:45:51.690397Z  INFO mini_redis: Request took 1ms
pong
```

若指定输出内容，则会把输出内容输出
```s
mini-redis>  ping 123
2023-09-12T13:12:47.359970Z  INFO mini_redis: Request took 1ms
123
```



## subscribe

开启此命令后会进入监听channel的状态，除非主动ctrl-c，不然程序会一直监听，语法如下
```
subscribe <channal_name>
```

进入监听后会进入如下状态，等待publish
```s
mini-redis>  subscribe 456
The message is as follow: 

```

## publish指令

publish指令格式如下
```
publish <channel_name> <message>
```

当publish后会返回收到信息的客户端的个数
```s
mini-redis>  publish 456 shabi
2023-09-11T16:49:44.928856Z  INFO mini_redis: Request took 2ms
publish success. The number of subscriber is 1
```

而监听端也会收到相应信息
```s
mini-redis>  subscribe 456
The message is as follow: 
2023-09-11T16:49:44.929214Z  INFO mini_redis: Request took 121340ms
shabi

```

若没有subscriber时则会输出以下信息
```s
mini-redis>  publish 456 7
2023-09-11T16:50:48.907189Z  INFO mini_redis: Request took 1ms
No subscriber found
```

## exit

输入该指令退客户端

## 敏感词过滤

```s
mini-redis>  set 123 傻逼
2023-09-11T17:09:13.997962Z ERROR client: Application(ApplicationError { kind: ApplicationErrorKind(0), message: "命令中有敏感词'傻逼'" })
mini-redis>  get 123
2023-09-11T17:09:18.116990Z  INFO mini_redis: Request took 2ms
The key: 123 is not in the database
```