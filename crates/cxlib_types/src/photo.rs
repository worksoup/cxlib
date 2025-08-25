use crate::session::Session;
use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::collect::{CloudItem, NetdiskProtocolTrait};
use derive_where::derive_where;
use std::marker::PhantomData;

#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct Photo<NetdiskProtocol> {
    item: CloudItem,
    _p: PhantomData<NetdiskProtocol>,
}
impl<T> Photo<T> {
    #[inline]
    pub fn get_object_id(&self) -> &String {
        self.item.object_id()
    }
}
impl<P> From<CloudItem> for Photo<P> {
    #[inline]
    fn from(value: CloudItem) -> Self {
        Self {
            item: value,
            _p: PhantomData,
        }
    }
}
impl<NetdiskProtocol: NetdiskProtocolTrait> Photo<NetdiskProtocol> {
    #[cfg(feature = "upload-file")]
    #[inline]
    pub fn new<R: std::io::Read>(
        session: &Session,
        file: R,
        file_name: impl AsRef<std::path::Path>,
    ) -> Result<Self, AgentError> {
        let item = CloudItem::upload_temporary::<NetdiskProtocol, _>(
            session,
            session.uid(),
            file_name,
            file,
        )?;
        Ok(item.into())
    }
    #[inline]
    pub fn default(session: &Session) -> Option<Self> {
        Self::find_in_cxpan(session, |a| a == "1.png" || a == "1.jpg")
            .log_ok()
            .flatten()
    }
    #[inline]
    pub fn find_in_cxpan(
        session: &Session,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<Self>, AgentError> {
        let r = NetdiskProtocol::chaoxing_netdisk_root(session)?;
        let mut r = r.ls::<NetdiskProtocol>(session)?;
        Ok(r.find(|item| p(item.name())).map(Into::into))
    }
    #[cfg(feature = "upload-file")]
    #[inline]
    pub fn get_from_file(
        session: &Session,
        file_path: impl AsRef<std::path::Path>,
    ) -> Result<Self, AgentError> {
        let f = std::fs::File::open(&file_path).log_unwrap();
        let file_name = file_path
            .as_ref()
            .file_name()
            .expect("路径中不包含图片文件的名称");
        Self::new(session, &f, file_name)
    }
}
