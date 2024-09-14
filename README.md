# CNH/CNY实时汇率监控机器人

## 项目简介
该项目主要用于监控CNH/CNY实时汇率，当汇率达到预设阈值时，可以通过设置的Ntfy.sh或者Telegram发送通知。

## Todo
- [] 通过Webhook发送通知

## 使用方法
### Docker
```shell
docker run -d --name=cnh_cny_rate_monitor -e -v config.toml:/app/config.toml --restart=always ghcr.io/chikage0o0/forex_notify:latest
```