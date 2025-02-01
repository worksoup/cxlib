use crate::{AppTrait, CmdApp, CmdMetaAppTrait};
use clap::{ArgMatches, FromArgMatches, Parser};
use cxlib_internal::default_impl::signner::LocationInfoGetterTrait;
use cxlib_internal::{
    activity::{Activity, RawSign},
    default_impl::{
        sign::Sign,
        signner::{
            DefaultGestureOrSigncodeSignner, DefaultLocationInfoGetter, DefaultLocationSignner,
            DefaultNormalOrRawSignner, DefaultPhotoSignner, DefaultQrCodeSignner,
        },
        store::{AccountTable, DataBase},
    },
    error::Error,
    sign::{SignResult, SignTrait, SignnerTrait},
    user::Session,
};
use log::{error, info, warn};
use std::marker::PhantomData;
use std::task::Context;
use std::{collections::HashMap, path::PathBuf, time::Duration};

#[derive(Clone)]
pub struct CliArgs {
    pub location_str: Option<String>,
    pub image: Option<PathBuf>,
    // pub capture: bool,
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    pub precisely: bool,
    pub signcode: Option<String>,
}

#[derive(Debug, Parser, Clone)]
#[command(
    author,
    version,
    about = "进行签到。",
    long_about = r#"
进行签到。

关于签到行为：

普通签到不需要指定任何选项。
拍照签到可指定 `-p, --pic` 选项，提供照片位置。如不提供则从云盘上获取图片。
二维码签到可指定 `-i, --image` 选项，提供照片位置。如不提供则从屏幕上截取。
位置签到可指定 `-l, --location` 选项。如不提供则根据教师设置的签到范围或数据库中获取。
手势或签到码签到须指定 `-c, --code` 选项，提供签到码。
"#
)]
#[clap(about = "列出有效签到。")]
pub struct SignParser {
    /// 签到 ID.
    /// 默认以最近起对所有有效签到顺序进行签到，且缺少参数时会跳过并继续。
    pub id: Option<i64>,
    /// 签到账号，格式为以半角逗号隔开的 uid (可通过 accounts 子命令查看).
    /// 默认以一定顺序对所有用户进行签到。
    #[arg(short, long)]
    pub uid: Option<String>,
    /// 指定位置。
    /// 教师未指定位置的位置签到或需要位置的二维码签到需要提供。
    /// 格式为：`地址,经度,纬度,海拔`, 不满足格式的字符串将被视为别名。
    /// 如果该别名不存在，则视为位置 ID.
    /// 其余情况将视为自动获取位置时指定的地址名。
    /// 如未指定或错误指定则按照先课程位置后全局位置的顺序依次尝试。
    #[arg(short, long)]
    pub location: Option<String>,
    /// 本地图片路径。
    /// 拍照签到需要提供，二维码签到可选提供。
    /// 如果是文件，则直接使用该文件作为拍照签到图片或二维码图片文件。
    /// 如果是目录，则会选择在该目录下修改日期最新的图片作为拍照签到图片或二维码图片。
    #[arg(short, long)]
    pub image: Option<PathBuf>,
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    /// 精确地截取二维码。
    /// 如果二维码识别过慢可以尝试添加添加此选项。
    #[arg(long)]
    pub precisely: bool,
    /// 签到码。
    /// 签到码签到时需要提供。
    #[arg(short, long)]
    pub code: Option<String>,
}
impl SignParser {
    pub(crate) const NOTICE: &'static str = r#"

++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

    列出签到时会默认排除从未发过签到或最后一次签到在 160 天
    之前的课程。

    如有需要，请使用 `cxsign list -a` 命令强制列出所有签到
    或使用 `cxsign list -c <COURSE_ID>` 列出特定课程的签
    到，此时将会刷新排除列表。

    注意，`cxsign list -a` 耗时十几秒到数分钟不等。不过后者
    耗时较短。

++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

"#;
    pub fn match_signs<T: LocationInfoGetterTrait>(
        raw_sign: RawSign,
        location_getter: T,
        sessions: &[Session],
        cli_args: &CliArgs,
    ) -> Result<(), Error> {
        let sign_name = raw_sign.name.clone();
        let mut sign = if sessions.is_empty() {
            warn!("无法判断签到[{sign_name}]的签到类型。");
            Sign::Unknown(raw_sign)
        } else {
            info!("成功判断签到[{sign_name}]的签到类型。");
            Sign::from_raw(raw_sign, &sessions[0])
        };
        let sign = &mut sign;
        let CliArgs {
            location_str,
            image,
            signcode,
            #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
            precisely,
        } = cli_args;
        #[allow(clippy::mutable_key_type)]
        let mut sign_results = HashMap::new();
        let sessions = sessions.iter();
        match sign {
            Sign::Photo(ps) => {
                info!("签到[{sign_name}]为拍照签到。");
                sign_results = DefaultPhotoSignner::new(image).sign(ps, sessions)?;
            }
            Sign::Normal(ns) => {
                info!("签到[{sign_name}]为普通签到。");
                sign_results = DefaultNormalOrRawSignner.sign(ns, sessions)?;
            }
            Sign::QrCode(qs) => {
                info!("签到[{sign_name}]为二维码签到。");
                sign_results = DefaultQrCodeSignner::new(
                    location_getter,
                    location_str,
                    image,
                    &None,
                    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
                    *precisely,
                )
                .sign(qs, sessions)?;
            }
            Sign::Gesture(gs) => {
                info!("签到[{sign_name}]为手势签到。");
                if let Some(signcode) = signcode {
                    sign_results =
                        DefaultGestureOrSigncodeSignner::new(signcode).sign(gs, sessions)?;
                } else {
                    warn!(
                        "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                        gs.as_inner().name
                    )
                }
            }
            Sign::Location(ls) => {
                info!("签到[{sign_name}]为位置签到。");
                sign_results = DefaultLocationSignner::new(location_getter, location_str)
                    .sign(ls, sessions)?;
            }
            Sign::Signcode(ss) => {
                info!("签到[{sign_name}]为签到码签到。");
                if let Some(signcode) = signcode {
                    sign_results =
                        DefaultGestureOrSigncodeSignner::new(signcode).sign(ss, sessions)?;
                } else {
                    warn!(
                        "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                        ss.as_inner().name
                    )
                }
            }
            Sign::Unknown(us) => {
                warn!("签到[{}]为无效签到类型！", us.name);
                sign_results = DefaultNormalOrRawSignner.sign(us, sessions)?;
            }
        }
        if !sign_results.is_empty() {
            info!("签到活动[{}]签到结果：", sign.as_raw().name);
            for (session, sign_result) in sign_results {
                if let SignResult::Fail { msg } = sign_result {
                    warn!(
                        "\t用户[{}]签到失败！失败信息：[{:?}]",
                        session.get_stu_name(),
                        msg
                    );
                } else {
                    info!("\t用户[{}]签到成功！", session.get_stu_name(),);
                }
            }
        }
        Ok(())
    }
    pub fn do_sign<T: LocationInfoGetterTrait + Copy>(
        self,
        db: &DataBase,
        location_getter: T,
    ) -> Result<(), Error> {
        let Self {
            id: active_id,
            uid: uid_list_str,
            location,
            image,
            precisely,
            code,
        } = self;
        let arg = CliArgs {
            location_str: location,
            image,
            precisely,
            signcode: code,
        };
        let (sessions, has_uid_arg) = if let Some(uid_list_str) = &uid_list_str {
            (
                AccountTable::get_sessions_by_uid_list_str(db, uid_list_str),
                true,
            )
        } else {
            (AccountTable::get_sessions(db), false)
        };
        let activities = Activity::get_all_activities(db, sessions.values(), false)?;
        let (valid_signs, other_signs): (
            HashMap<RawSign, Vec<Session>>,
            HashMap<RawSign, Vec<Session>>,
        ) = activities
            .into_iter()
            .filter_map(|(k, v)| match k {
                Activity::RawSign(k) => Some((k, v)),
                Activity::Other(_) => None,
            })
            .partition(|(k, _)| k.is_valid());
        let signs = if let Some(active_id) = active_id {
            let (sign, sessions) = {
                if let Some(s1) = valid_signs
                    .into_iter()
                    .find(|kv| kv.0.as_inner().active_id == active_id.to_string())
                {
                    s1
                } else if let Some(s2) = other_signs
                    .into_iter()
                    .find(|kv| kv.0.as_inner().active_id == active_id.to_string())
                {
                    s2
                } else if has_uid_arg {
                    panic!(
                        "没有该签到活动！请检查签到活动 ID 是否正确或所指定的账号是否存在该签到活动！"
                    );
                } else {
                    panic!("没有该签到活动！请检查签到活动 ID 是否正确！");
                }
            };
            let mut map = HashMap::new();
            map.insert(sign, sessions);
            map
        } else {
            let mut signs = HashMap::new();
            for (sign, sessions) in valid_signs {
                signs.insert(sign, sessions);
            }
            signs
        };
        if signs.is_empty() {
            warn!("签到列表为空。");
        }
        for (sign, sessions) in signs {
            info!(
                "即将处理签到：[{}], id 为 {}, 开始时间为 {}, 课程为 {} / {} / {}",
                sign.name,
                sign.active_id,
                chrono::DateTime::<chrono::Local>::from(
                    std::time::UNIX_EPOCH + Duration::from_millis(sign.start_time_mills)
                )
                .format("%+")
                .to_string(),
                sign.course.get_class_id(),
                sign.course.get_id(),
                sign.course.get_name()
            );
            let mut names = Vec::new();
            for s in sessions.iter() {
                names.push(s.get_stu_name().to_string())
            }
            info!("签到者：{names:?}");
            Self::match_signs(sign, location_getter, &sessions, &arg)
                .unwrap_or_else(|e| warn!("{e}"));
        }
        Ok(())
    }
}
pub struct SignMainApp<T: LocationInfoGetterTrait, Context, F: Fn(&Context) -> T> {
    _context: PhantomData<Context>,
    f: F,
}
impl<T: LocationInfoGetterTrait, Context, F: Fn(&Context) -> T> SignMainApp<T, Context, F> {
    pub fn new(f: F) -> Self {
        Self {
            _context: PhantomData,
            f,
        }
    }
}
impl<Context: AsRef<DataBase>, T: LocationInfoGetterTrait + Copy, F: Fn(&Context) -> T>
    AppTrait<Context> for SignMainApp<T, Context, F>
{
    type OwnedData = SignParser;

    fn run(&self, db: &Context, data: Self::OwnedData) {
        warn!("{}", SignParser::NOTICE);
        data.do_sign(db.as_ref(), (self.f)(db))
            .unwrap_or_else(|e| error!("签到失败！错误信息：{e}."));
    }
}

impl<
        Context: AsRef<DataBase> + 'static,
        T: LocationInfoGetterTrait + Copy + 'static,
        F: Fn(&Context) -> T + 'static,
    > CmdMetaAppTrait<CmdApp<Context>, Context> for SignMainApp<T, Context, F>
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        SignParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
