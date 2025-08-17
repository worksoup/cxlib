//! 写了 8 行导入语句、3 行辅助特型、8 行特型实现，只为复用 40 行的代码。
//! 好，还有 2 行调侃。
use crate::sign::{LocationSign, QrCodeSign};
use cxlib_captcha::CaptchaSolverTrait;
use cxlib_protocol::collect::{CaptchaProtocolTrait, PreSignResult, SignProtocolTrait};
use cxlib_sign::{SignError, SignResult, SignTrait};
use cxlib_types::{Geoaddr, Session};
use log::warn;
use std::borrow::Borrow;

pub(crate) trait SignRetry<I, O: Borrow<<Self as SignTrait>::Data>, SignProtocol>:
    SignTrait
{
    #[inline]
    fn guess_if_retry(msg: &str) -> bool {
        msg.contains("位置")
            || msg.contains("Location")
            || msg.contains("范围")
            || msg.contains("location")
    }
    fn data_helper(data: I) -> O;
}
impl<SignProtocol> SignRetry<Geoaddr, <Self as SignTrait>::Data, SignProtocol> for QrCodeSign {
    #[inline]
    fn data_helper(data: Geoaddr) -> <Self as SignTrait>::Data {
        Some(data)
    }
}
impl<'a, SignProtocol> SignRetry<&'a Geoaddr, &'a <Self as SignTrait>::Data, SignProtocol>
    for LocationSign
{
    #[inline]
    fn data_helper(data: &'a Geoaddr) -> &'a <Self as SignTrait>::Data {
        data
    }
}
/// 提供数据，不断进行签到，成功则返回。其通过失败时的 msg 判断是否需要重试，若无需重试，则签到失败。
pub(crate) fn sign_single_retry<
    CaptchaSolver: CaptchaSolverTrait,
    CaptchaProtocol,
    SignProtocol,
    U,
    Sign,
    InputData,
    Data,
    InputDataIter,
>(
    sign: &Sign,
    session: &Session<U>,
    (pre_sign_data, locations): (&<Sign as SignTrait>::PreSignData, InputDataIter),
) -> Result<SignResult, SignError>
where
    SignProtocol: SignProtocolTrait,
    Sign: SignTrait + SignRetry<InputData, Data, SignProtocol>,
    Data: Borrow<<Sign as SignTrait>::Data>,
    InputDataIter: IntoIterator<Item = InputData>,
    CaptchaProtocol: CaptchaProtocolTrait,
{
    let state = SignProtocol::get_sign_state(session, sign.as_inner().active_id())?;
    let guess_result = SignResult::guess_by_state(state, session.name(), sign.as_inner().name());
    if let Some(guess_result) = guess_result {
        return Ok(guess_result);
    }
    let r = sign.pre_sign::<CaptchaProtocol, SignProtocol, U>(session, pre_sign_data)?;
    match r {
        PreSignResult::Susses => Ok(SignResult::Success),
        PreSignResult::Data {
            ref url,
            data: ref pre_sign_result_data,
        } => {
            for location in locations {
                match sign.sign::<CaptchaSolver, CaptchaProtocol, SignProtocol, U>(
                    session,
                    url,
                    pre_sign_result_data,
                    pre_sign_data,
                    Sign::data_helper(location).borrow(),
                )? {
                    r @ (SignResult::Success | SignResult::PartialSuccess { .. }) => return Ok(r),
                    SignResult::Failure { msg } => {
                        if Sign::guess_if_retry(msg.as_str()) {
                            continue;
                        } else {
                            return Ok(SignResult::Failure { msg });
                        }
                    }
                }
            }
            warn!("BUG: 请保留现场联系开发者处理。");
            Ok(SignResult::Failure {
                msg: "所有位置均不可用。".to_string(),
            })
        }
    }
}
