# 代理健康监控

一个 systemd/Windows 服务，用于定期检查 Mihomo 代理的延迟。

![image](https://github.com/user-attachments/assets/335e6143-7019-4e05-9fad-6d6d1a814759)


## 监控流程

1. **发起延迟测试**
   端点：`GET /group/{group_name}/delay`
   - 测试指定组中的节点/策略组
   - 返回新的延迟信息
   - 清除自动策略组中的固定选择
   - 需要URL参数：`url={test_url}&timeout={timeout_ms}`

2. **获取测试结果**
   端点：`GET /providers/proxies`
   - 获取所有代理集合信息

3. **解析响应时间**
   - 在 `proxies` 中找到配置的组
   - 组中的 `all` 字段包含代理列表
   - 在 `proxies` 中按名称查找每个代理
   - 代理的 `history` 中的最新条目是测试结果（延迟，单位为毫秒）

## 配置

| 配置项                 | 参数               | 示例值                                 | 描述                                 |
|------------------------|--------------------|----------------------------------------|--------------------------------------|
| `api_url`              | API地址           | `http://127.0.0.1:9090/`              | Mihomo API 基础地址                 |
| `api_secret`           | API密钥           | `xxxxx`                               | 认证令牌                            |
| `groups_to_monitor`    | 监控组            | `["Japan", "USA"]`                    | 要监控的代理组                      |
| `interval_seconds`     | 检查间隔          | `15`                                  | 检查之间的时间（秒）                |
| `test_url`             | 测试URL           | `https://www.gstatic.com/generate_204` | 用于延迟测试的URL                   |
| `test_timeout_seconds` | 测试超时          | `5`                                   | 测试请求的超时时间（秒）            |
| `prometheus.mode`      | Prometheus模式    | `pull`                                | `pull` 或 `push` 报告模式           |
| `prometheus.push_url`  | 推送URL           | `http://127.0.0.1:8428/api/v1/write`  | 当模式为 `push` 时必需的            |
| `prometheus.listen_address` | 监听地址    | `0.0.0.0:9898`                        | 在 pull 模式下的 `/metrics` 端点    |

## 报告模式

1. **拉取模式**
   暴露 Prometheus 的 `/metrics` 端点

2. **推送模式**
   将指标推送到 Prometheus 的远程写入端点
   （需要配置远程写入地址）

## 服务运行方式

### Windows Service 配置

1. 创建服务：
```powershell
New-Service -Name "ProxyHealthMonitor" `
            -BinaryPathName "D:\path\to\proxy-health-monitor.exe" `
            -DisplayName "Proxy Health Monitor" `
            -StartupType Automatic
```

2. 启动服务：
```powershell
Start-Service -Name "ProxyHealthMonitor"
```

3. 查看服务状态：
```powershell
Get-Service -Name "ProxyHealthMonitor"
```

### Linux systemd 配置

创建配置文件 `/etc/systemd/system/proxy-health-monitor.service`：

```ini
[Unit]
Description=Proxy Health Monitor
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/path/to/proxy-health-monitor
ExecStart=/path/to/proxy-health-monitor
Restart=always
RestartSec=5

# 环境变量配置示例
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

启用并启动服务：
```bash
systemctl daemon-reload
systemctl enable proxy-health-monitor
systemctl start proxy-health-monitor
```

查看服务状态：
```bash
journalctl -u proxy-health-monitor -f
```

> 注意：替换示例中的路径为实际安装路径
