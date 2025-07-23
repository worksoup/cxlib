use crate::{
    AccountTable, AppTrait, CmdMetaAppTrait, CourseDataFilterAndSorterTrait, CourseTable,
    CoursesCmdApp, DefaultCourseDataSorter, DefaultLocationInfoGetter, GlobalMultimap,
    error::Error,
};
use clap::{ArgMatches, FromArgMatches, Parser};
use cxlib_internal::{
    captcha::CaptchaSolver,
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
    sign::{SignResult, SignTrait, SignnerTrait},
    types::{
        Activity, Course, CourseWithInfo, LocationPreprocessorTrait, RawSign, Session,
        UntypedLoginSolver, ext::ActivityExt,
    },
};
use cxlib_store::AppInfo;
use log::{debug, error, info, warn};
use redb::Database;
use ref_wrapper::Unit;
use std::{collections::HashMap, marker::PhantomData, path::PathBuf, time::Duration};

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
}

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
    pub fn match_signs<CaptchaProtocol, SignProtocol, TypesProtocol, UserProtocol, T>(
        raw_sign: RawSign<SignProtocol>,
        location_getter: T,
        preprocessor: &impl LocationPreprocessorTrait,
        sessions: &[Session<UserProtocol>],
        captcha_solver: &'static CaptchaSolver,
        cli_args: &CliArgs,
    ) -> Result<(), Error>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: SignProtocolTrait + Send + 'static,
        TypesProtocol: TypesProtocolTrait + 'static,
        UserProtocol: UserProtocolTrait + Send + 'static,
        T: LocationInfoGetterTrait,
    {
        let sign_name = raw_sign.name().clone();
        let mut sign = if sessions.is_empty() {
            warn!("无法判断签到[{sign_name}]的签到类型。");
            Sign::<SignProtocol, TypesProtocol>::Unknown(raw_sign)
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
                sign_results = <DefaultPhotoSignner as SignnerTrait<_, CaptchaProtocol, _>>::sign(
                    &mut DefaultPhotoSignner::new(image),
                    ps,
                    sessions,
                    captcha_solver,
                )?;
            }
            Sign::Normal(ns) => {
                info!("签到[{sign_name}]为普通签到。");
                sign_results = <DefaultNormalOrRawSignner as SignnerTrait<
                    _,
                    CaptchaProtocol,
                    _,
                >>::sign(
                    &mut DefaultNormalOrRawSignner, ns, sessions, captcha_solver
                )?;
            }
            Sign::QrCode(qs) => {
                info!("签到[{sign_name}]为二维码签到。");
                sign_results = SignnerTrait::<_, CaptchaProtocol, _>::sign(
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
                    captcha_solver,
                )?;
            }
            Sign::GestureOrSigncode(goss) => {
                if goss.is_gesture() {
                    info!("签到[{sign_name}]为手势签到。");
                } else {
                    info!("签到[{sign_name}]为签到码签到。");
                }
                if let Some(signcode) = signcode {
                    sign_results = SignnerTrait::<_, CaptchaProtocol, _>::sign(
                        &mut DefaultGestureOrSigncodeSignner::new(signcode),
                        goss,
                        sessions,
                        captcha_solver,
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
                sign_results = SignnerTrait::<_, CaptchaProtocol, _>::sign(
                    &mut DefaultLocationSignner::new(location_getter, location_str, preprocessor),
                    ls,
                    sessions,
                    captcha_solver,
                )?;
            }
            Sign::Unknown(us) => {
                warn!("签到[{}]为无效签到类型！", us.name());
                sign_results = SignnerTrait::<_, CaptchaProtocol, _>::sign(
                    &mut DefaultNormalOrRawSignner,
                    us,
                    sessions,
                    captcha_solver,
                )?;
            }
        }
        if !sign_results.is_empty() {
            info!("签到活动[{}]签到结果：", sign.as_raw().name());
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
    pub fn do_sign<
        CaptchaProtocol,
        SignProtocol,
        TypesProtocol,
        UserProtocol,
        Cxt,
        LocationGetter,
        T: CourseDataFilterAndSorterTrait<Cxt>,
    >(
        self,
        cxt: &Cxt,
        location_getter: LocationGetter,
        preprocessor: &impl LocationPreprocessorTrait,
        captcha_solver: &'static CaptchaSolver,
    ) -> Result<(), Error>
    where
        CaptchaProtocol: CaptchaProtocolTrait,
        SignProtocol: std::marker::Send + SignProtocolTrait + 'static,
        TypesProtocol: TypesProtocolTrait + 'static,
        UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
        Cxt: AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>> + AsRef<Database>,
        LocationGetter: LocationInfoGetterTrait + Copy,
    {
        let db = cxt.as_ref();
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
        let login_solvers: &GlobalMultimap<_> = cxt.as_ref();
        let sessions = if let Some(uid_list_str) = &uid_list_str {
            AccountTable::get_sessions_by_uid_list_str(db, uid_list_str, login_solvers)?
        } else {
            AccountTable::get_all_sessions(db, login_solvers)?
        };
        let mut courses = CoursesCmdApp::update_course_table(db, login_solvers)?;
        let courses = if let Some(course) = course {
            Some((course.clone(), courses.remove(&course).unwrap()))
                .into_iter()
                .collect()
        } else {
            courses
        };
        let mut courses =
            CourseTable::courses_to_course_sessions_map_with_current_sessions(courses, sessions)
                .filter(|(course, (info, data, _))| T::filter((course, info, data)))
                .collect::<Vec<_>>();
        courses.sort_by(
            |(a_course, (a_info, a_data, _)), (b_course, (b_info, b_data, _))| {
                T::sorter((a_course, a_info, a_data), (b_course, b_info, b_data))
            },
        );
        {
            let debug_courses = courses.iter().map(|(a, _b)| a).collect::<Vec<_>>();
            debug!("{debug_courses:?}");
        }
        let iter = courses
            .into_iter()
            .map(|(course, (info, _, sessions))| (CourseWithInfo::new(course, info), sessions));
        let activities_receiver = if let Some(limit) = limit {
            Activity::<SignProtocol>::get_from_courses::<TypesProtocol, UserProtocol>(
                iter.take(limit),
            )
        } else {
            Activity::get_from_courses::<TypesProtocol, UserProtocol>(iter)
        };
        let signs = activities_receiver.into_iter().map(|(a, s)| {
            let max = a.iter().max_by_key(|a| a.start_time_mills()).unwrap();

            CourseTable::update_recently_used_time(db, max.course(), max.start_time_mills());
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
                        && active_id.is_none_or(|id| (*raw_sign.active_id()) == id.to_string())
                    {
                        have = true;
                        info!(
                            "即将处理签到：[{}], id 为 {}, 开始时间为 {}, 课程为 {} / {} / {}",
                            raw_sign.name(),
                            raw_sign.active_id(),
                            chrono::DateTime::<chrono::Local>::from(
                                std::time::UNIX_EPOCH
                                    + Duration::from_millis(*raw_sign.start_time_mills())
                            )
                            .format("%+"),
                            raw_sign.course().class_id(),
                            raw_sign.course().id(),
                            raw_sign.course().name()
                        );
                        let names = sessions.iter().map(|s| s.name()).collect::<Vec<_>>();
                        info!("签到者：{names:?}");
                        Self::match_signs::<CaptchaProtocol, _, TypesProtocol, _, _>(
                            raw_sign,
                            location_getter,
                            preprocessor,
                            &sessions,
                            captcha_solver,
                            &arg,
                        )
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
pub struct SignMainApp<
    CaptchaProtocol = cxlib_internal::protocol::collect::CaptchaProtocol,
    SignProtocol = cxlib_internal::protocol::collect::SignProtocol,
    TypesProtocol = cxlib_internal::protocol::collect::TypesProtocol,
    UserProtocol = cxlib_internal::protocol::collect::UserProtocol,
    Preprocessor = Unit,
    T = DefaultCourseDataSorter,
> {
    _p: PhantomData<(CaptchaProtocol, SignProtocol, TypesProtocol, UserProtocol)>,
    _lp: PhantomData<Preprocessor>,
    _t: PhantomData<T>,
}
impl<C, S, Ty, U, T> Default for SignMainApp<C, S, Ty, U, T> {
    #[inline]
    fn default() -> Self {
        SignMainApp {
            _p: PhantomData,
            _lp: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<CaptchaProtocol, SignProtocol, TypesProtocol, UserProtocol, Preprocessor, Context, T>
    AppTrait<Context>
    for SignMainApp<CaptchaProtocol, SignProtocol, TypesProtocol, UserProtocol, Preprocessor, T>
where
    Context: AsRef<Database>
        + AsRef<AppInfo>
        + AsRef<Preprocessor>
        + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>
        + AsRef<&'static CaptchaSolver>,
    T: CourseDataFilterAndSorterTrait<Context>,
    UserProtocol: UserProtocolTrait + std::marker::Send + 'static,
    CaptchaProtocol: CaptchaProtocolTrait,
    SignProtocol: Send + SignProtocolTrait + 'static,
    TypesProtocol: TypesProtocolTrait + 'static,
    Preprocessor: LocationPreprocessorTrait,
{
    type OwnedData = SignParser;

    fn run(&self, cxt: &Context, data: Self::OwnedData) {
        warn!("{}", SignParser::notice_content(cxt.as_ref()));
        data.do_sign::<CaptchaProtocol, SignProtocol, TypesProtocol, _, _, _, T>(
            cxt,
            DefaultLocationInfoGetter::from(cxt.as_ref()),
            AsRef::<Preprocessor>::as_ref(&cxt),
            AsRef::<&'static CaptchaSolver>::as_ref(&cxt),
        )
        .unwrap_or_else(|e| error!("签到失败！错误信息：{e}."));
    }
}

impl<
    CaptchaProtocol,
    SignProtocol,
    TypesProtocol,
    UserProtocol,
    Preprocessor,
    Context,
    OwnedData,
    T,
> CmdMetaAppTrait<Context, OwnedData>
    for SignMainApp<CaptchaProtocol, SignProtocol, TypesProtocol, UserProtocol, Preprocessor, T>
where
    Context: AsRef<Database>
        + AsRef<AppInfo>
        + AsRef<Preprocessor>
        + AsRef<GlobalMultimap<UntypedLoginSolver<UserProtocol>>>
        + AsRef<&'static CaptchaSolver>
        + 'static,
    OwnedData: 'static,
    T: CourseDataFilterAndSorterTrait<Context> + 'static,
    UserProtocol: 'static + std::marker::Send + UserProtocolTrait,
    CaptchaProtocol: 'static + CaptchaProtocolTrait,
    SignProtocol: Send + SignProtocolTrait + 'static,
    TypesProtocol: TypesProtocolTrait + 'static,
    Preprocessor: LocationPreprocessorTrait + 'static,
{
    fn read_owned_data(
        &self,
        _: &Context,
        matches: &[&ArgMatches],
    ) -> <Self as AppTrait<Context, ()>>::OwnedData {
        SignParser::from_arg_matches(matches.last().unwrap()).unwrap()
    }
}
