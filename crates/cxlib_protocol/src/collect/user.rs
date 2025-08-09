use cxlib_error::AgentError;
use cxlib_error_utils::CxlibResultUtils;
use ureq::{Agent, Body, http::Response};

use crate::ProtocolError;

mod types {
    use cxlib_base_types::{Class, ClassInfo, RawCourse};
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Default)]
    pub struct Courses {
        data: Vec<RawCourse>,
    }

    #[derive(Deserialize, Debug)]
    pub struct RawClass {
        #[serde(rename = "clazzId")]
        id: i64,
        // 0 | 1, 代表班级开课或结束。
        state: Option<u8>,
    }
    #[derive(Deserialize, Debug)]
    pub struct CourseContent {
        // TODO: 该字段不应为 Option.
        //       深层原因为，Class 与 Course 并非一对多的关系。
        //       而是多对多的关系。
        //       同时，当前实现混淆了两者的 ID, 可能会出现一些错误。
        course: Option<Courses>,
        #[serde(rename = "clazz")]
        classes: Option<Vec<RawClass>>,
        // TODO: 需要测试 `Option` 是否可以为 `flatten`.
        #[serde(flatten)]
        raw_course: Option<RawCourse>,
        // 0 | 1, 代表班级开课或结束。
        state: Option<u8>,
    }

    #[derive(Deserialize, Debug)]
    pub struct ClassRaw {
        // 可能为 ClassId, 也可能是 CourseId.
        #[serde(rename = "key")]
        id: serde_json::Value,
        content: CourseContent,
    }
    impl ClassRaw {
        pub fn into_class(self) -> Vec<Class> {
            let Self { id, content } = self;
            if id.is_i64() {
                let class_id = id.as_i64().unwrap();
                let CourseContent { course, state, .. } = content;
                let courses = course.unwrap_or_default().data;
                vec![Class::new(
                    courses,
                    ClassInfo::new_with_state(class_id, state),
                )]
            } else if id.is_string() {
                // let course_id = id.as_str()
                //     .unwrap()
                //     .strip_prefix("tea_")
                //     .expect(
                //         "CourseId 格式不正确（不以 `tea_` 开头），请检查 API 是否存在更新。",
                //     )
                //     .parse()
                //     .unwrap();
                let CourseContent {
                    classes,
                    raw_course,
                    ..
                } = content;
                classes
                    .map(|classes| {
                        classes.into_iter().map(|clazz| {
                            Class::new(
                                raw_course.clone().into_iter().collect(),
                                ClassInfo::new_with_state(clazz.id, clazz.state),
                            )
                        })
                    })
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
            } else {
                panic!(
                    "JSON 解析失败：存在意外类型的 ClassId 或 CourseId(json 键为 \"key\"), 请检查 API 是否存在更新。"
                )
            }
        }
    }
}
pub use types::*;
pub trait UserProtocolTrait {
    #[inline]
    fn login_page_url() -> &'static str {
        UserProtocol::LOGIN_PAGE
    }
    #[inline]
    fn login_enc_url() -> &'static str {
        UserProtocol::LOGIN_ENC
    }
    #[inline]
    fn account_manage_url() -> &'static str {
        UserProtocol::ACCOUNT_MANAGE
    }
    #[inline]
    fn back_clazz_data_url() -> &'static str {
        UserProtocol::BACK_CLAZZ_DATA
    }
    // 登录页
    #[inline]
    fn login_page(client: &Agent) -> Result<Response<Body>, AgentError> {
        let url = Self::login_page_url();
        Ok(client.get(url).call()?)
    }

    // 非明文密码登录
    #[inline]
    fn login_enc(client: &Agent, uname: &str, pwd_enc: &str) -> Result<Response<Body>, AgentError> {
        let url = Self::login_enc_url();
        Ok(client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("X-Requested-With", "XMLHttpRequest")
            .send(format!("uname={uname}&password={pwd_enc}&fid=-1&t=true&refer=https%253A%252F%252Fi.chaoxing.com&forbidotherlogin=0&validate="))?)
    }

    // 账号设置页
    #[inline]
    fn find_name_in_account_manage_page(client: &Agent) -> Result<String, ProtocolError> {
        fn data_err(err_msg: &'static str) -> impl FnOnce() -> ProtocolError {
            || ProtocolError::DataParseError(err_msg.to_owned())
        }
        let url = Self::account_manage_url();
        let html_content = client
            .get(url)
            .call()?
            .into_body()
            .read_to_string()
            .log_unwrap();
        log::trace!("{html_content}");
        let e = html_content.find("colorBlue").ok_or_else(data_err(
            "无法获取姓名！锚点属性 `colorBlue` 无法找到，可能是登录过期或页面已更新。",
        ))?;
        let html_content = html_content[e..html_content.len()].to_owned();
        let e = html_content
            .find('>')
            .ok_or_else(data_err(
                "无法获取姓名！锚点文本 `>` 无法找到，可能是登录页面已更新。",
            ))
            .log_unwrap()
            + 1;
        let html_content = html_content[e..html_content.len()].to_owned();
        let name = html_content[0..html_content
            .find('<')
            .ok_or_else(data_err(
                "无法获取姓名！锚点文本 `<` 无法找到，可能是登录页面已更新。",
            ))
            .log_unwrap()]
            .trim();
        Ok(name.to_owned())
    }

    // 获取课程
    #[inline]
    fn get_classes(client: &Agent) -> Result<Vec<ClassRaw>, ProtocolError> {
        #[derive(serde::Deserialize, Debug)]
        struct GetCoursesR {
            #[serde(rename = "channelList")]
            channel_list: Option<Vec<ClassRaw>>,
        }
        let url = Self::back_clazz_data_url();
        let r = client.get(&format!("{url}?view=json&rss=1")).call()?;
        r.into_body()
            .read_json::<GetCoursesR>()
            .log_unwrap()
            .channel_list
            .ok_or_else(|| ProtocolError::DataParseError("clazz_list is empty.".to_owned()))
    }
}
pub struct UserProtocol;
impl UserProtocol {
    /// 登录页
    pub const LOGIN_PAGE: &'static str = "https://passport2.chaoxing.com/mlogin?fid=&newversion=true&refer=http%3A%2F%2Fi.chaoxing.com";
    /// 非明文密码登录
    pub const LOGIN_ENC: &'static str = "https://passport2.chaoxing.com/fanyalogin";
    /// 账号设置页
    pub const ACCOUNT_MANAGE: &'static str = "https://passport2.chaoxing.com/mooc/accountManage";
    /// 获取课程
    pub const BACK_CLAZZ_DATA: &'static str =
        "https://mooc1-api.chaoxing.com/mycourse/backclazzdata";
}
impl UserProtocolTrait for UserProtocol {}
