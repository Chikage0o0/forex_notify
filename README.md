# CNH/CNY实时汇率监控机器人

## 项目简介
该项目主要用于监控CNH/CNY实时汇率，当汇率达到预设阈值时，可以通过设置的Ntfy.sh或者Telegram发送通知。

## Todo
- [x] 通过Webhook发送通知

## 使用方法

### 配置文件
```toml
log_level = "info"
warning_threshold = 0.998
api_key = "demo"     # 访问 https://twelvedata.com/ 申请免费API，获取API Key   
sleeptime = 240      # 每次轮询间隔时间，免费API有次数限制，建议设置为4分钟以上

[[notifiers]]        # 可以添加多个notifiers
type = "Telegram"
token = "token"      # @BotFather，新建一个Bot获取token
chat_id = "chat_id"  # 发送一个信息给Bot或者将Bot拉入要聊天的频道、群组，然后发送信息并访问https://api.telegram.org/bot<YourBOTToken>/getUpdates

[[notifiers]]
type = "Ntfy"
url = "url"     # https://ntfy.sh/test
token = "token" # optional
title = "title"
priority = 4

[[notifiers]]
type = "Webhook"
url = "http://example.com"          # Webhook地址
template = '''
{
    "under_threshold": {under_threshold},
    "rate": {rate}
}
'''                                 # Webhook模板    {under_threshold} 为是否低于阈值，{rate} 为当前汇率
method = "Post"                     # Webhook请求方法             GET/POST/PUT

[notifiers.headers]                 # Webhook请求头
Content-Type = "application/json"
```

### Docker
```shell
docker run -d --name=cnh_cny_rate_monitor -e -v config.toml:/app/config.toml --restart=always ghcr.io/chikage0o0/forex_notify:latest
```

### 命令行
先将config.toml 放置在当前目录下，然后执行以下命令
```shell
./forex_notify
```
