use crate::{
    AppTrait, CmdMetaAppTrait, CourseDataFilterAndSorterTrait, CoursesCmdApp,
    DefaultCourseDataSorter,
};
use clap::{ArgMatches, FromArgMatches, Parser};
use cxlib_internal::store::AppInfo;
use cxlib_internal::{
    default_impl::{
        sign::Sign,
        signner::{
            DefaultGestureOrSigncodeSignner, DefaultLocationInfoGetter, DefaultLocationSignner,
            DefaultNormalOrRawSignner, DefaultPhotoSignner, DefaultQrCodeSignner,
            LocationInfoGetterTrait,
        },
        store::{AccountTable, CourseData, CourseTable, DataBase},
    },
    error::Error,
    sign::{SignResult, SignTrait, SignnerTrait},
    types::{ext::ActivityExt, Activity, RawSign, Session},
};
use log::{debug, error, info, warn};
use std::marker::PhantomData;
use std::{cmp, collections::HashMap, path::PathBuf, time::Duration};

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
/// 进行签到。
pub struct SignParser {
    /// 签到 ID.
    /// 默认以最近起对所有有效签到顺序进行签到，且缺少参数时会跳过并继续。
    id: Option<i64>,
    /// 签到账号，格式为以半角逗号隔开的 uid (可通过 accounts 子命令查看).
    /// 默认以一定顺序对所有用户进行签到。
    #[arg(short, long)]
    uid: Option<String>,
    /// 指定位置。
    /// 教师未指定位置的位置签到或需要位置的二维码签到需要提供。
    /// 格式为：`地址,经度,纬度,海拔`, 不满足格式的字符串将被视为别名。
    /// 如果该别名不存在，则视为位置 ID.
    /// 其余情况将视为自动获取位置时指定的地址名。
    /// 如未指定或错误指定则按照先课程位置后全局位置的顺序依次尝试。
    #[arg(short = 'L', long)]
    location: Option<String>,
    /// 本地图片路径。
    /// 拍照签到需要提供，二维码签到可选提供。
    /// 如果是文件，则直接使用该文件作为拍照签到图片或二维码图片文件。
    /// 如果是目录，则会选择在该目录下修改日期最新的图片作为拍照签到图片或二维码图片。
    #[arg(short, long)]
    image: Option<PathBuf>,
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    /// 精确地截取二维码。
    /// 如果二维码识别过慢可以尝试添加添加此选项。
    #[arg(short, long)]
    precisely: bool,
    /// 签到码。
    /// 签到码签到时需要提供。
    #[arg(short = 'C', long)]
    code: Option<String>,
    /// 获取签到时限制课程数量。默认无限制。该数量限制作用在初步过滤无效课程后。
    #[arg(short = 'N', long)]
    limit: Option<usize>,
    /// 列出签到而不处理。
    #[arg(short, long)]
    list: bool,
    /// 处理或列出指定课程的签到。
    #[arg(short, long)]
    course: Option<i64>,
    /// 处理或列出所有签到（包括无效签到）。
    #[arg(short, long)]
    all: bool,
}

impl SignParser {
    pub fn notice_content() -> String {
        let app = AppInfo::get_instance().application();
        format!(
            r#"

++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

    列出签到时会默认排除从未发过签到或最后一次签到在 160 天
    之前的课程。

    如有需要，请使用 `{app} list -a` 命令强制列出所有签到
    或使用 `{app} list -c <COURSE_ID>` 列出特定课程的签
    到，此时将会刷新排除列表。

    注意，`{app} list -a` 耗时十几秒到数分钟不等。不过后者
    耗时较短。

++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

"#
        )
    }
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
                    warn!("\t用户[{}]签到失败！失败信息：[{:?}]", session.name(), msg);
                } else {
                    info!("\t用户[{}]签到成功！", session.name(),);
                }
            }
        }
        Ok(())
    }
    pub fn do_sign<T: LocationInfoGetterTrait + Copy>(
        self,
        db: &DataBase,
        location_getter: T,
        mut filter: impl FnMut(&CourseData) -> bool,
        mut sorter: impl FnMut(&CourseData, &CourseData) -> cmp::Ordering,
    ) -> Result<(), Error> {
        let Self {
            id: active_id,
            uid: uid_list_str,
            location,
            image,
            precisely,
            code,
            limit,
            list,
            course,
            all,
        } = self;
        let arg = CliArgs {
            location_str: location,
            image,
            precisely,
            signcode: code,
        };
        let has_uid_arg = uid_list_str.is_some();
        let sessions = if let Some(uid_list_str) = &uid_list_str {
            AccountTable::get_sessions_by_uid_list_str(db, uid_list_str)
        } else {
            AccountTable::get_sessions(db)
        };
        let mut courses = CoursesCmdApp::update_course_table(db);
        let courses = if let Some(course) = course {
            Some((course, courses.remove(&course).unwrap()))
                .into_iter()
                .collect()
        } else {
            courses
        };
        let mut courses = CourseTable::courses_to_course_sessions_map_with_current_sessions(
            courses.into_values(),
            sessions,
        )
        .filter(|(c, _s)| filter(c))
        .collect::<Vec<_>>();
        courses.sort_by(|(a, _), (b, _)| sorter(a, b));
        {
            let debug_courses = courses.iter().map(|(a, _b)| a).collect::<Vec<_>>();
            debug!("{debug_courses:?}");
        }
        let iter = courses.into_iter().map(|(c, s)| (c.into_inner(), s));
        let activities_receiver = if let Some(limit) = limit {
            Activity::get_from_courses(iter.take(limit))
        } else {
            Activity::get_from_courses(iter)
        };
        let signs = activities_receiver.into_iter().map(|(a, s)| {
            let max = a.iter().max_by_key(|a| a.start_time_mills()).unwrap();
            let _ = CourseTable::update_recently_used_time(
                db,
                max.course().id(),
                max.start_time_mills(),
            );
            (
                a.into_iter().filter_map(|a| match a {
                    Activity::RawSign(k) => Some(k),
                    Activity::Other(_) => None,
                }),
                s,
            )
        });
        if list {
            if let Some(active_id) = active_id {
                warn!("将忽略活动 ID 参数（{active_id}）。")
            }
            let signs = signs.flat_map(|(activities, sessions)| {
                activities.map(move |a| {
                    let s = sessions
                        .iter()
                        .map(|s| s.name().to_owned())
                        .collect::<Vec<_>>();
                    (a, s)
                })
            });
            for (sign, names) in signs {
                if all || sign.is_valid() {
                    println!("{names:?}:{sign}");
                }
            }
        } else {
            let mut have = false;
            for (raw_signs, sessions) in signs {
                for raw_sign in raw_signs {
                    // 相信分支预测。
                    if (all || raw_sign.is_valid())
                        && active_id.is_none_or(|id| raw_sign.active_id == id.to_string())
                    {
                        have = true;
                        info!(
                            "即将处理签到：[{}], id 为 {}, 开始时间为 {}, 课程为 {} / {} / {}",
                            raw_sign.name,
                            raw_sign.active_id,
                            chrono::DateTime::<chrono::Local>::from(
                                std::time::UNIX_EPOCH
                                    + Duration::from_millis(raw_sign.start_time_mills)
                            )
                            .format("%+"),
                            raw_sign.course.class_id(),
                            raw_sign.course.id(),
                            raw_sign.course.name()
                        );
                        let names = sessions.iter().map(|s| s.name()).collect::<Vec<_>>();
                        info!("签到者：{names:?}");
                        Self::match_signs(raw_sign, location_getter, &sessions, &arg)
                            .unwrap_or_else(|e| warn!("{e}"));
                    }
                }
            }
            if !have {
                if active_id.is_some() {
                    if has_uid_arg {
                        panic!(
                            "没有该签到活动！请检查签到活动 ID 是否正确或所指定的账号是否存在该签到活动！"
                        );
                    } else {
                        panic!("没有该签到活动！请检查签到活动 ID 是否正确！");
                    }
                } else {
                    warn!("签到列表为空。");
                }
            }
        }
        Ok(())
    }
}
pub struct SignMainApp<T = DefaultCourseDataSorter> {
    _t: PhantomData<T>,
}
impl<T> Default for SignMainApp<T> {
    #[inline]
    fn default() -> SignMainApp<T> {
        SignMainApp {
            _t: Default::default(),
        }
    }
}

impl<Context, T> AppTrait<Context> for SignMainApp<T>
where
    Context: AsRef<DataBase>,
    T: CourseDataFilterAndSorterTrait,
{
    type OwnedData = SignParser;

    fn run(&self, db: &Context, data: Self::OwnedData) {
        warn!("{}", SignParser::notice_content());
        data.do_sign(
            db.as_ref(),
            DefaultLocationInfoGetter::from(db.as_ref()),
            T::filter,
            T::sorter,
        )
        .unwrap_or_else(|e| error!("签到失败！错误信息：{e}."));
    }
}

impl<Context, OwnedData, T> CmdMetaAppTrait<Context, OwnedData> for SignMainApp<T>
where
    Context: AsRef<DataBase> + 'static,
    OwnedData: 'static,
    T: CourseDataFilterAndSorterTrait + 'static,
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        SignParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
