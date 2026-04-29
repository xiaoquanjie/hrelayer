# hrelayer
一个支持http1，http2的路由器，支持服务路由、服务注册与发现

```
需要准备的东西：
1. cmake：内部依赖librdkafka v1.9.2+.库，所以需要安装cmake进行编译
2. etcd：使用etcd作为服务注册与发现的组件

运行辅助：
1. hrealyer.toml是配置文件
2. 服务注册的管理后台： https://github.com/xiaoquanjie/detector-web

启动服务：
 hrelayer.exe --config hrelayer.toml
