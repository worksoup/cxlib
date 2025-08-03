use crate::{
    AccountTable, ActivityTable, AppTrait, CmdMetaAppTrait, CourseDataFilterAndSorterTrait,
    CourseTable, CoursesCmdApp, DefaultCourseDataSorter, DefaultLocationInfoGetter, GlobalMultimap,
    NormalTableTrait, StoreError, database_guard::DatabaseGuard, error::Error,
};
use clap::{ArgMatches, FromArgMatches, Parser};
use cxlib_error_utils::MaybeFatalError;
use cxlib_internal::{
    captcha::CaptchaSolverTrait,
    default_impl::{
        sign::Sign,
        signner::{
            DefaultGestureOrSigncodeSignner, DefaultLocationSignner, DefaultNormalOrRawSignner,
            DefaultPhotoSignner, DefaultQrCodeSignner, LocationInfoGetterTrait,
        },
    },
    protocol::collect::{
        CaptchaProtocolTrait, SignProtocolTrait, TypesProtocolTrait, UserProtocolTrait,
    },
    sign::{SignError, SignResult, SignTrait, SignnerTrait},
    types::{
        Activity, Course, CourseWithInfo, LocationPreprocessorTrait, RawSign, Session,
        UntypedLoginSolver, ext::ActivityExt,
    },
};
use cxlib_store::AppInfo;
use log::{debug, error, info, warn};
use redb::WriteTransaction;
use ref_wrapper::Unit;
use std::{
    borrow::Borrow, collections::HashMap, marker::PhantomData, path::PathBuf, time::Duration,
};

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
    course: Option<Course>,
    /// 处理或列出所有签到（包括无效签到）。
    #[arg(short, long)]
    all: bool,
    #[arg(short, long)]
    /// 刷新
    fresh: bool,
}

type CachedActivitiesResult<'s, UserProtocol> =
    HashMap<String, (Activity, Vec<&'s Session<UserProtocol>>)>;
impl SignParser {
    pub fn notice_content(app_info: &AppInfo) -> String {
        let app = app_info.application();
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
}
/// 以下文档由 AI 生成。
///
///  签到主应用结构体
///
/// 泛型参数说明:
/// - `CaptchaProtocol`: 验证码协议实现
/// - `SignProtocol`: 签到协议实现
/// - `TypesProtocol`: 类型协议实现
/// - `UserProtocol`: 用户协议实现
/// - `Preprocessor`: 位置预处理器
/// - `T`: 课程数据处理策略
pub struct SignMainApp<
    CaptchaSolver = cxlib_internal::captcha::SlideImages,
    CaptchaProtocol = cxlib_internal::protocol::collect::CaptchaProtocol,
    SignProtocol = cxlib_internal::protocol::collect::SignProtocol,
    TypesProtocol = cxlib_internal::protocol::collect::TypesProtocol,
    UserProtocol = cxlib_internal::protocol::collect::UserProtocol,
    Preprocessor = Unit,
    T = DefaultCourseDataSorter,
> {
    _c: PhantomData<CaptchaSolver>,
    /// 以下文档由 AI 生成。
    ///
    ///  泛型标记(用于类型推导)
    _p: PhantomData<(CaptchaProtocol, SignProtocol, TypesProtocol, UserProtocol)>,
    /// 以下文档由 AI 生成。
    ///
    ///  位置处理器标记
    _lp: PhantomData<Preprocessor>,
    /// 以下文档由 AI 生成。
    ///
    ///  课程数据处理策略标记
    _t: PhantomData<T>,
}
/// 以下文档由 AI 生成。
///
///  为签到主应用提供默认实现
impl<CS, C, S, Ty, U, T> Default for SignMainApp<CS, C, S, Ty, U, T> {
    #[inline]
    fn default() -> Self {
        SignMainApp {
            _c: PhantomData,
            _p: PhantomData,
            _lp: PhantomData,
            _t: PhantomData,
        }
    }
}
impl<CaptchaSolver, CaptchaProtocol, SignProtocol, TypesProtocol, UserProtocol, Preprocessor, T>
    SignMainApp<
        CaptchaSolver,
        CaptchaProtocol,
        SignProtocol,
        TypesProtocol,
        UserProtocol,
        Preprocessor,
        T,
    >
{
    /// 以下文档由 AI 生成。
    ///
    ///  处理特定类型的签到
    ///
    /// # 参数
    /// - `sign_name`: 签到名称标识
    /// - `typed_sign`: 类型化的签到对象
    /// - `location_getter`: 位置信息获取器
    /// - `preprocessor`: 位置预处理组件
    /// - `sessions`: 用户会话集合
    /// - `captcha_solver`: 验证码解决器
    /// - `cli_args`: 命令行参数
    ///
    /// # 返回
    /// 每个会话的签到结果映射
    pub fn process_typed_sign<'s, LocationGetter>(
        sign_name: String,
        typed_sign: &mut Sign<TypesProtocol>,
        (location_getter, preprocessor): (LocationGetter, &impl LocationPreprocessorTrait),
        sessions: impl IntoIterator<Item = &'s Session<UserProtocol>>,
        cli_args: &CliArgs,
    ) -> Result<HashMap<&'s Session<UserProtocol>, SignResult>, Error>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait + Send + 'static,
        TypesProtocol: TypesProtocolTrait + 'static,
        UserProtocol: UserProtocolTrait + Send + 'static,
        LocationGetter: LocationInfoGetterTrait,
        CaptchaSolver: CaptchaSolverTrait,
    {
        let sessions = sessions.into_iter();
        let CliArgs {
            location_str,
            image,
            signcode,
            #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
            precisely,
        } = cli_args;
        #[allow(clippy::mutable_key_type)]
        let mut sign_results = HashMap::new();
        match typed_sign {
            Sign::Photo(ps) => {
                info!("签到[{sign_name}]为拍照签到。");
                sign_results = <DefaultPhotoSignner as SignnerTrait<
                    _,
                    CaptchaSolver,
                    CaptchaProtocol,
                    SignProtocol,
                >>::sign(
                    &mut DefaultPhotoSignner::new(image), ps, sessions
                )?;
            }
            Sign::Normal(ns) => {
                info!("签到[{sign_name}]为普通签到。");
                sign_results =
                    <DefaultNormalOrRawSignner as SignnerTrait<
                        _,
                        CaptchaSolver,
                        CaptchaProtocol,
                        SignProtocol,
                    >>::sign(&mut DefaultNormalOrRawSignner, ns, sessions)?;
            }
            Sign::QrCode(qs) => {
                info!("签到[{sign_name}]为二维码签到。");
                sign_results =
                    SignnerTrait::<_, CaptchaSolver, CaptchaProtocol, SignProtocol>::sign(
                        &mut DefaultQrCodeSignner::new(
                            location_getter,
                            location_str,
                            image,
                            &None,
                            #[cfg(any(
                                target_os = "linux",
                                target_os = "windows",
                                target_os = "macos"
                            ))]
                            *precisely,
                            preprocessor,
                        ),
                        qs,
                        sessions,
                    )?;
            }
            Sign::GestureOrSigncode(goss) => {
                if goss.is_gesture() {
                    info!("签到[{sign_name}]为手势签到。");
                } else {
                    info!("签到[{sign_name}]为签到码签到。");
                }
                if let Some(signcode) = signcode {
                    sign_results =
                        SignnerTrait::<_, CaptchaSolver, CaptchaProtocol, SignProtocol>::sign(
                            &mut DefaultGestureOrSigncodeSignner::new(signcode),
                            goss,
                            sessions,
                        )?;
                } else if goss.is_gesture() {
                    warn!(
                        "所有用户在手势签到[{}]中签到失败！需要提供签到码！",
                        goss.as_inner().name()
                    )
                } else {
                    warn!(
                        "所有用户在签到码签到[{}]中签到失败！需要提供签到码！",
                        goss.as_inner().name()
                    )
                }
            }
            Sign::Location(ls) => {
                info!("签到[{sign_name}]为位置签到。");
                sign_results =
                    SignnerTrait::<_, CaptchaSolver, CaptchaProtocol, SignProtocol>::sign(
                        &mut DefaultLocationSignner::new(
                            location_getter,
                            location_str,
                            preprocessor,
                        ),
                        ls,
                        sessions,
                    )?;
            }
            Sign::Unknown(us) => {
                warn!("签到[{}]为无效签到类型！", us.name());
                sign_results =
                    SignnerTrait::<_, CaptchaSolver, CaptchaProtocol, SignProtocol>::sign(
                        &mut DefaultNormalOrRawSignner,
                        us,
                        sessions,
                    )?;
            }
        }
        Ok(sign_results)
    }
    /// 以下文档由 AI 生成。
    ///
    ///  匹配并处理签到类型
    ///
    /// # 参数
    /// - `raw_sign`: 原始签到数据
    /// - `location_getter`: 位置信息获取器
    /// - `preprocessor`: 位置预处理组件
    /// - `sessions`: 用户会话集合
    /// - `captcha_solver`: 验证码解决器
    /// - `cli_args`: 命令行参数
    ///
    /// # 返回
    /// 元组包含处理后的原始签到数据和签到结果
    pub fn match_signs<'s, LocationGetter>(
        raw_sign: RawSign,
        location_cxt: (LocationGetter, &impl LocationPreprocessorTrait),
        sessions: impl IntoIterator<Item = &'s Session<UserProtocol>>,
        cli_args: &CliArgs,
    ) -> (
        RawSign,
        Result<HashMap<&'s Session<UserProtocol>, SignResult>, Error>,
    )
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait + Send + 'static,
        TypesProtocol: TypesProtocolTrait + 'static,
        UserProtocol: UserProtocolTrait + Send + 'static,
        LocationGetter: LocationInfoGetterTrait,
        CaptchaSolver: CaptchaSolverTrait,
    {
        let mut sessions = sessions.into_iter().peekable();
        let sign_name = raw_sign.name().clone();
        let mut typed_sign = if let Some(session) = sessions.peek() {
            let typed_sign = Sign::<TypesProtocol>::from_raw(raw_sign, session);
            info!("成功判断签到[{sign_name}]的签到类型。");
            typed_sign
        } else {
            return (
                raw_sign,
                Err(SignError::SignDataNotFound(format!(
                    "用户会话为空，无法处理签到[{sign_name}]."
                ))
                .into()),
            );
        };
        let r =
            Self::process_typed_sign(sign_name, &mut typed_sign, location_cxt, sessions, cli_args);
        (typed_sign.into_raw(), r)
    }
    /// 以下文档由 AI 生成。
    ///
    ///  获取签到数据并执行签到操作
    ///
    /// 核心业务逻辑：处理命令行参数，从数据库获取数据，
    /// 根据参数执行签到或显示签到信息
    pub fn get_sign_and_do_sign<Cxt, LocationGetter>(
        sign_parser: SignParser,
        cxt: &Cxt,
        location_cxt: (LocationGetter, &impl LocationPreprocessorTrait),
    ) -> Result<(), Error>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: std::marker::Send + SignProtocolTrait + 'static,
        TypesProtocol: TypesProtocolTrait + 'static,
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
        Cxt: AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + AsRef<DatabaseGuard>,
        LocationGetter: LocationInfoGetterTrait + Copy,
        CaptchaSolver: CaptchaSolverTrait,
        T: CourseDataFilterAndSorterTrait<Cxt>,
    {
        let db_g: &DatabaseGuard = cxt.as_ref();
        let mut db_g = db_g.clone();
        let SignParser {
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
            fresh,
        } = sign_parser;
        let arg = CliArgs {
            location_str: location,
            image,
            precisely,
            signcode: code,
        };
        let has_uid_arg = uid_list_str.is_some();
        let login_solvers: &GlobalMultimap<_> = cxt.as_ref();
        // 两分支均能自动新建数据表。
        let sessions = if let Some(uid_list_str) = &uid_list_str {
            AccountTable::get_sessions_by_uid_list_str(&mut db_g, uid_list_str, login_solvers)?
        } else {
            AccountTable::get_all_sessions(&mut db_g, login_solvers)?
        };
        if fresh {
            let mut courses = CourseTable::courses_to_course_sessions_map_with_current_sessions(
                CoursesCmdApp::update_sessions_courses(&mut db_g, sessions.values())?,
                &sessions,
            )
            .collect::<HashMap<_, _>>();
            let courses = if let Some(course) = course {
                vec![(course.clone(), courses.remove(&course).unwrap())]
            } else {
                let mut courses = courses
                    .into_iter()
                    .filter(|(course, (info, data, _))| T::filter((course, info, data)))
                    .collect::<Vec<_>>();
                courses.sort_by(
                    |(a_course, (a_info, a_data, _)), (b_course, (b_info, b_data, _))| {
                        T::sorter((a_course, a_info, a_data), (b_course, b_info, b_data))
                    },
                );
                courses
            };
            #[cfg(debug_assertions)]
            {
                let debug_courses = courses.iter().map(|(a, _b)| a).collect::<Vec<_>>();
                debug!("{debug_courses:?}");
            }
            let iter = courses
                .into_iter()
                .map(|(course, (info, _, sessions))| (CourseWithInfo::new(course, info), sessions))
                .take(limit.unwrap_or(usize::MAX));
            db_g.write_once_map_err::<_, _, StoreError, _, _>(
                |w_cxt| {
                    let activities = Self::get_activities(w_cxt, iter)?;
                    let activities = activities.flat_map(|(activities, users)| {
                        let sessions = users.into_iter().filter_map(|s| sessions.get(s.uid()));
                        activities
                            .into_iter()
                            .map(move |activity| (activity, sessions.clone()))
                    });

                    if list {
                        Self::display_activities(all, active_id, activities);
                    } else {
                        Self::do_sign(all, active_id, has_uid_arg, location_cxt, &arg, activities)?;
                    }
                    Ok::<_, Error>(())
                },
                Error::from,
            )?;
        } else {
            let activities = Self::get_cached_activities(cxt, sessions.values())?;
            let activities = if let Some(course) = course {
                activities
                    .into_iter()
                    .filter(|(_, (actiity, _))| actiity.course().course().eq(&course))
                    .collect()
            } else {
                activities
            }
            .into_values();
            if list {
                Self::display_activities(all, active_id, activities);
            } else {
                Self::do_sign(all, active_id, has_uid_arg, location_cxt, &arg, activities)?;
            }
        };
        Ok(())
    }
    /// 以下文档由 AI 生成。
    ///
    ///  执行实际的签到操作
    ///
    /// # 参数
    /// - `all`: 是否处理所有签到
    /// - `active_id`: 特定签到ID
    /// - `has_uid_arg`: 是否有用户ID参数
    /// - `location_cxt`: 位置上下文(获取器和预处理器)
    /// - `captcha_solver`: 验证码解决器
    /// - `cli_args`: 命令行参数
    /// - `activities`: 待处理的签到活动集合
    ///
    /// # 返回
    /// 操作结果(成功或错误)
    pub fn do_sign<'s, LocationGetter>(
        all: bool,
        active_id: Option<i64>,
        has_uid_arg: bool,
        location_cxt: (LocationGetter, &impl LocationPreprocessorTrait),
        cli_args: &CliArgs,
        activities: impl IntoIterator<
            Item = (
                Activity,
                impl IntoIterator<Item = &'s Session<UserProtocol>>,
            ),
        >,
    ) -> Result<(), Error>
    where
        CaptchaSolver: CaptchaSolverTrait,
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait + std::marker::Send + 'static,
        TypesProtocol: TypesProtocolTrait + 'static,
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
        LocationGetter: LocationInfoGetterTrait + Copy,
    {
        let signs = activities.into_iter().filter_map(|(activity, sessions)| {
            if let Activity::RawSign(raw_sign) = activity
                && (all || raw_sign.is_valid())
                && active_id.is_none_or(|id| (*raw_sign.active_id()) == id.to_string())
            {
                Some((raw_sign, sessions))
            } else {
                None
            }
        });
        let mut have = false;
        for (raw_sign, sessions) in signs {
            have = true;
            info!(
                "即将处理签到：[{}], id 为 {}, 开始时间为 {}, 课程为 {} / {} / {}",
                raw_sign.name(),
                raw_sign.active_id(),
                chrono::DateTime::<chrono::Local>::from(
                    std::time::UNIX_EPOCH + Duration::from_millis(*raw_sign.start_time_mills())
                )
                .format("%+"),
                raw_sign.course().class_id(),
                raw_sign.course().id(),
                raw_sign.course().name()
            );
            let sessions = sessions.into_iter().collect::<Vec<_>>();
            let names = sessions.iter().map(|s| s.name()).collect::<Vec<_>>();
            info!("签到者：{names:?}");
            let (raw_sign, result) = Self::match_signs(raw_sign, location_cxt, sessions, cli_args);
            match result {
                Ok(sign_results) => {
                    info!("签到活动[{}]签到结果：", raw_sign.name());
                    for (session, sign_result) in sign_results {
                        match sign_result {
                            SignResult::Success => {
                                info!("\t用户[{}]签到成功！", session.name(),);
                            }
                            SignResult::PartialSuccess { msg } => {
                                warn!("\t用户[{}]签到成功：[{:?}]。", session.name(), msg);
                            }
                            SignResult::Failure { msg } => {
                                warn!("\t用户[{}]签到失败！失败信息：[{:?}]", session.name(), msg);
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.is_fatal() {
                        Err(e)?
                    } else {
                        warn!("`{e}`.");
                        continue;
                    }
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
        Ok(())
    }
    /// 以下文档由 AI 生成。
    ///
    ///  显示签到活动信息
    ///
    /// # 参数
    /// - `all`: 是否显示所有签到
    /// - `active_id`: 特定活动ID
    /// - `activities`: 要显示的签到活动集合
    pub fn display_activities<'s>(
        all: bool,
        active_id: Option<i64>,
        activities: impl IntoIterator<
            Item = (
                Activity,
                impl IntoIterator<Item = impl Borrow<Session<UserProtocol>> + 's>,
            ),
        >,
    ) where
        UserProtocol: 's,
    {
        if let Some(active_id) = active_id {
            warn!("将忽略活动 ID 参数（{active_id}）。")
        }
        let signs = activities
            .into_iter()
            .filter_map(|(activity, sessions)| match activity {
                Activity::RawSign(raw_sign) => Some((
                    raw_sign,
                    sessions
                        .into_iter()
                        .map(|s| s.borrow().name().to_owned())
                        .collect::<Vec<_>>(),
                )),
                Activity::Other(_) => None,
            });
        for (sign, names) in signs {
            if all || sign.is_valid() {
                println!("{names:?}:{sign}");
            }
        }
    }
    /// 以下文档由 AI 生成。
    ///
    ///  更新活动数据表
    ///
    /// # 参数
    /// - `w_cxt`: 数据库写事务上下文
    /// - `courses`: 课程信息集合
    ///
    /// # 返回
    /// 迭代器(包含活动列表和对应的用户会话)
    pub fn update_activity_table(
        w_cxt: &WriteTransaction,
        courses: impl IntoIterator<
            Item = (
                CourseWithInfo,
                impl IntoIterator<Item = Session<UserProtocol>>,
            ),
        >,
    ) -> Result<impl Iterator<Item = (Vec<Activity>, Vec<Session<UserProtocol>>)>, StoreError>
    where
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
        TypesProtocol: TypesProtocolTrait,
    {
        let receiver =
            Activity::get_from_courses::<TypesProtocol, UserProtocol>(courses.into_iter());
        let r = receiver.into_iter().map(move |(activities, users)| {
            let users_str = users.iter().map(|s| s.uid().to_owned()).collect::<Vec<_>>();
            for activity in &activities {
                debug!("活动：{activity:?}");
                let key = activity.id().to_owned();
                if let Err(e) =
                    ActivityTable::merge(w_cxt, &key, (activity.clone(), users_str.clone()))
                {
                    warn!("`{e}`");
                }
            }
            (activities, users.clone())
        });
        Ok(r)
    }
    /// 以下文档由 AI 生成。
    ///
    ///  从缓存获取活动数据
    ///
    /// # 参数
    /// - `cxt`: 数据库上下文
    /// - `sessions`: 用户会话集合
    ///
    /// # 返回
    /// 缓存的活动信息映射表
    pub fn get_cached_activities<'a, Cxt>(
        cxt: Cxt,
        sessions: impl IntoIterator<Item = &'a Session<UserProtocol>>,
    ) -> Result<CachedActivitiesResult<'a, UserProtocol>, Error>
    where
        Cxt: AsRef<DatabaseGuard>,
        UserProtocol: 'a,
    {
        let database_guard = cxt.as_ref();
        let sessions = sessions
            .into_iter()
            .map(|s| (s.uid().to_owned(), s))
            .collect::<HashMap<_, _>>();
        Ok(database_guard
            .read(|r_cxt| {
                let table = ActivityTable::read(r_cxt)?;
                let activities = ActivityTable::iter(&table)?
                    .into_iter()
                    .map(|(key, (activity, users))| {
                        let sessions = users
                            .into_iter()
                            .filter_map(|uid| sessions.get(uid.as_str()))
                            .copied()
                            .collect::<Vec<_>>();
                        (key, (activity, sessions))
                    })
                    .collect::<HashMap<_, _>>();
                Ok::<_, StoreError>(activities)
            })?
            .unwrap_inner())
    }
    /// 以下文档由 AI 生成。
    ///
    ///  获取活动数据(带课程信息)
    ///
    /// # 参数
    /// - `w_cxt`: 数据库写事务上下文
    /// - `courses`: 课程信息集合
    ///
    /// # 返回
    /// 活动数据和对应的用户会话
    #[inline]
    pub fn get_activities(
        w_cxt: &WriteTransaction,
        courses: impl IntoIterator<Item = (CourseWithInfo, Vec<Session<UserProtocol>>)>,
    ) -> Result<impl Iterator<Item = (Vec<Activity>, Vec<Session<UserProtocol>>)>, StoreError>
    where
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
        TypesProtocol: TypesProtocolTrait,
    {
        Self::update_activity_table(w_cxt, courses)
    }
}
/// 以下文档由 AI 生成。
///
///  实现应用接口的签到主应用
impl<
    CaptchaSolver: CaptchaSolverTrait,
    CaptchaProtocol,
    SignProtocol,
    TypesProtocol,
    UserProtocol,
    Preprocessor,
    Context,
    T,
> AppTrait<Context>
    for SignMainApp<
        CaptchaSolver,
        CaptchaProtocol,
        SignProtocol,
        TypesProtocol,
        UserProtocol,
        Preprocessor,
        T,
    >
where
    Context: AsRef<DatabaseGuard>
        + AsRef<AppInfo>
        + AsRef<Preprocessor>
        + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>,
    T: CourseDataFilterAndSorterTrait<Context>,
    UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: Send + SignProtocolTrait + 'static,
    TypesProtocol: TypesProtocolTrait + 'static,
    Preprocessor: LocationPreprocessorTrait,
{
    type OwnedData = SignParser;
    /// 以下文档由 AI 生成。
    ///
    ///  运行签到主应用
    ///
    /// 1. 显示提示信息
    /// 2. 获取签到数据
    /// 3. 执行签到操作
    #[inline]
    fn run(&self, cxt: &Context, data: Self::OwnedData) {
        warn!("{}", SignParser::notice_content(cxt.as_ref()));
        Self::get_sign_and_do_sign(
            data,
            cxt,
            (
                DefaultLocationInfoGetter::from(cxt.as_ref()),
                AsRef::<Preprocessor>::as_ref(&cxt),
            ),
        )
        .unwrap_or_else(|e| error!("签到失败！错误信息：{e}."));
    }

    fn meta_app<MetaApp>(self) -> Self
    where
        Self: Sized,
        MetaApp:
            crate::ConstructFromTrait<Self, Context, ()> + crate::MetaAppTrait<Self, Context, ()>,
    {
        let meta_app = MetaApp::construct_from(&self);
        meta_app.register(self)
    }

    fn register_meta_app<MetaApp>(self, meta_app: MetaApp) -> Self
    where
        Self: Sized,
        MetaApp: crate::MetaAppTrait<Self, Context, ()>,
    {
        meta_app.register(self)
    }
}
/// 以下文档由 AI 生成。
///
///  实现命令行元应用接口
impl<
    CaptchaSolver,
    CaptchaProtocol,
    SignProtocol,
    TypesProtocol,
    UserProtocol,
    Preprocessor,
    Context,
    OwnedData,
    T,
> CmdMetaAppTrait<Context, OwnedData>
    for SignMainApp<
        CaptchaSolver,
        CaptchaProtocol,
        SignProtocol,
        TypesProtocol,
        UserProtocol,
        Preprocessor,
        T,
    >
where
    Context: AsRef<DatabaseGuard>
        + AsRef<AppInfo>
        + AsRef<Preprocessor>
        + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>
        + 'static,
    OwnedData: 'static,
    T: CourseDataFilterAndSorterTrait<Context> + 'static,
    UserProtocol: 'static + std::marker::Send + UserProtocolTrait,
    CaptchaProtocol: 'static + CaptchaProtocolTrait,
    SignProtocol: Send + SignProtocolTrait + 'static,
    TypesProtocol: TypesProtocolTrait + 'static,
    Preprocessor: LocationPreprocessorTrait + 'static,
    CaptchaSolver: CaptchaSolverTrait + 'static,
{
    /// 以下文档由 AI 生成。
    ///
    ///  从命令行参数解析数据
    ///
    /// 使用clap解析器将命令行参数转换为结构化数据
    #[inline]
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        SignParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
