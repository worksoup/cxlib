/// *本文档由 AI 生成。*
///
/// 通过 ureq Agent 获取 URL 二进制内容（支持自定义请求头）
///
/// 执行 HTTP GET 请求获取目标 URL 的完整响应体，支持自定义 Referer 请求头。
///
/// # 参数
/// - `agent`: 预配置的 ureq 客户端实例（包含代理/超时等设置）
/// - `url`: 目标资源地址（支持 HTTP/HTTPS 协议）
/// - `referer`: 设置的 Referer 请求头值
///
/// # 返回值
/// - `Ok(Vec<u8>)`: 包含完整响应体的字节向量
/// - `Err(Box<ureq::Error>)`: 网络请求失败（保留原始错误详情）
///
/// # 功能特性
/// - 全量读取响应内容到内存
/// - 自动处理重定向（遵循 agent 配置）
/// - 支持 HTTPS 和 HTTP/2（取决于 agent 配置）
///
/// # 内存安全警告
/// **响应体大小不作校验**：
/// 函数将完整响应体加载到内存，可能导致：
/// - 超大文件下载时内存溢出（OOM）
/// - 服务端恶意返回无限数据时耗尽内存
///
/// # 错误处理
/// - 网络请求失败：返回装箱的 `ureq::Error`
/// - 响应读取失败：触发 `panic!` (因内部 `unwrap()`)
///
/// # 应用场景
/// 适用于：
/// 1. 需要 Referer 头的防盗链资源访问
/// 2. 小型二进制文件下载（如图片、图标等）
/// 3. API 响应内容获取（特别是二进制格式）
///
/// # 示例
/// ```no_run
/// use my_crate::ureq_get_bytes;
///
/// let agent = ureq::AgentBuilder::new()
///     .timeout_connect(std::time::Duration::from_secs(5))
///     .build();
///
/// match ureq_get_bytes(
///     &agent,
///     "https://example.com/data.bin",
///     "https://trusted-referrer.com"
/// ) {
///     Ok(data) => println!("下载成功! 大小: {} bytes", data.len()),
///     Err(e) => eprintln!("请求失败: {}", e),
/// }
/// ```
///
/// # 替代方案
/// 对内存敏感场景，建议改用流式读取：
/// ```no_run
/// let mut reader = agent.get(url).call()?.into_reader();
/// // 分块读取处理
/// ```
pub fn ureq_get_bytes(
    agent: &ureq::Agent,
    url: &str,
    referer: &str,
) -> Result<Vec<u8>, Box<ureq::Error>> {
    use std::io::Read;
    let mut img = Vec::new();
    agent
        .get(url)
        .header("Referer", referer)
        .call()?
        .into_body()
        .into_reader()
        .read_to_end(&mut img)
        .unwrap();
    Ok(img)
}
