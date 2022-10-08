# 先在本一建立一个命令映射，假设路径为/root/cmds.json.格式如下：
```
[{"name":"ls","cmd":"ls /root"},{"name":"test","cmd":"echo 'test'"},{"name":"delete","cmd":"rm -f /root/cmd.txt"}]
```
# 设置mqtt的broker信息.大部分默认即可。目前不支持用户名和密码参数.主要设置tpoic,确保有一定的复杂度即可，随便设置目前不清楚有什么限制。

```
./pub --topic 'XXXXXXXXXXXXX' --msg 'hello'
./sub --topic 'XXXXXXXXXXXXX'
```
## pub发布端命令，需要设置msg信息即消息内容
```
USAGE:
    pub [OPTIONS]
OPTIONS:
        --client <CLIENT>    必需唯一ID [default: rust_publish_pub]
    -h, --help               Print help information
        --host <HOST>        [default: tcp://broker.emqx.io:1883]
        --msg <MSG>          [default: "hello world"]
        --topic <TOPIC>      [default: rust/mqtt]
    -V, --version            Print version information
```

## sub订阅端命令
```
USAGE:
    sub [OPTIONS]
OPTIONS:
        --client <CLIENT>    必需唯一ID [default: rust_subscribe_sub]
        --cmd <CMD>          执行命令的键值映射文件，绝对路径 [default:
                             ~/cmd.json]
    -h, --help               Print help information
        --host <HOST>        [default: tcp://broker.emqx.io:1883]
        --topic <TOPIC>      [default: rust/mqtt]
    -V, --version            Print version information
```
# 订阅发布内容,cmd如果不执行命令留空即可
```
{
  "msg": "消息",
  "cmd": "命令的name"
}
```