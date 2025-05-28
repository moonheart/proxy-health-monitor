use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct HistoryEntry {
    pub time: String, // 考虑使用 chrono::DateTime<chrono::FixedOffset> 或直接解析为 DateTime
    pub delay: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TestData {
    pub alive: bool,
    pub history: Vec<HistoryEntry>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProxyDetail {
    pub name: String,
    pub alive: bool,
    pub all: Option<Vec<String>>, // 用于策略组
    pub history: Option<Vec<HistoryEntry>>, // 用于单个代理节点
    #[serde(rename = "type")]
    pub proxy_type: String,
    // extra 字段结构比较灵活，这里用 serde_json::Value 来接收，后续按需解析
    // 或者根据 Readme.md 中 test_url 的具体值来定义更精确的结构
    pub extra: Option<HashMap<String, TestData>>, 
    // 根据 proxies.json 补充一些可能用到的字段
    pub now: Option<String>, // 策略组当前选择的节点
    pub udp: Option<bool>,
    // 可以根据需要添加更多字段
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProxiesResponse {
    pub proxies: HashMap<String, ProxyDetail>,
}