use crate::session::Session;
use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use cxlib_protocol::collect::{CloudItem, TypesProtocolTrait};
use derive_where::derive_where;
use std::{fs::File, marker::PhantomData, path::Path};

#[derive_where(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct Photo<TypesProtocol> {
    item: CloudItem,
    _p: PhantomData<TypesProtocol>,
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
impl<TypesProtocol: TypesProtocolTrait> Photo<TypesProtocol> {
    #[inline]
    pub fn new<U>(
        session: &Session<U>,
        file: &File,
        file_name: impl AsRef<Path>,
    ) -> Result<Self, AgentError> {
        let item = CloudItem::upload_temporary::<TypesProtocol, _>(
            session,
            session.uid(),
            file_name,
            file,
        )?;
        Ok(item.into())
    }
    #[inline]
    pub fn default<U>(session: &Session<U>) -> Option<Self> {
        Self::find_in_cxpan(session, |a| a == "1.png" || a == "1.jpg")
            .log_ok()
            .flatten()
    }
    #[inline]
    pub fn find_in_cxpan<U>(
        session: &Session<U>,
        p: impl Fn(&str) -> bool,
    ) -> Result<Option<Self>, AgentError> {
        let r = TypesProtocol::chaoxing_netdisk_root(session)?;
        let mut r = r.ls::<TypesProtocol>(session)?;
        Ok(r.find(|item| p(item.name())).map(Into::into))
    }
    #[inline]
    pub fn get_from_file<U>(
        session: &Session<U>,
        file_path: impl AsRef<Path>,
    ) -> Result<Self, AgentError> {
        let f = File::open(&file_path).log_unwrap();
        let file_name = file_path
            .as_ref()
            .file_name()
            .expect("路径中不包含图片文件的名称");
        Self::new(session, &f, file_name)
    }
}
