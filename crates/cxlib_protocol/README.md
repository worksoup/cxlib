# cxlib_protocol

提供了 `ProtocolTrait`:

```rust
pub trait ProtocolTrait<Protocol>: Sync {
    fn get(&self, t: &Protocol) -> String;

    fn set(&self, t: &Protocol, value: &str);
    fn store(&self) -> Result<(), cxlib_error::Error>;
    fn update(&self, t: &Protocol, value: &str) -> bool;
}
```

使用时，定义一个全局变量以及相关方法：

```rust
static PROTOCOL: OnceInit<dyn ProtocolTrait<ProtocolList>> = OnceInit::new();

pub fn set_protocol(
    protocol: &'static impl ProtocolTrait<DefaultProtocolList>,
) -> Result<(), InitError> {
    PROTOCOL.set_data(protocol)
}

pub fn set_boxed_protocol(
    protocol: Box<impl ProtocolTrait<DefaultProtocolList> + 'static>,
) -> Result<(), InitError> {
    PROTOCOL.set_boxed_data(protocol)
}
```

然后设法公开 `get`, `set` 等方法，即可方便管理。

同时在 `cxlib_default_impl` 中提供了 `ProtocolDataTrait<ProtocolList>`, 按照规范实现，即可方便地使用
`DefaultCXProtocol<ProtocolData> as dyn ProtocolTrait<ProtocolList>`, 其可以自动从磁盘加载配置文件，并可持久化内容。 