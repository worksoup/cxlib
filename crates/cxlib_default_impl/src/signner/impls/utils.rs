//! 写了 8 行导入语句、3 行辅助特型、8 行特型实现，只为复用 40 行的代码。
//! 好，还有 2 行调侃。
use crate::sign::{LocationSign, QrCodeSign};
use cxlib_captcha::CaptchaSolver;
use cxlib_protocol::collect::{CaptchaProtocolTrait, SignProtocolTrait};
use cxlib_sign::{PreSignResult, SignError, SignResult, SignTrait};
use cxlib_types::{Geoaddr, Session};
use log::warn;
use std::borrow::Borrow;

pub(crate) trait SignRetry<I, O: Borrow<<Self as SignTrait>::Data>, SignProtocol>:
    SignTrait
{
    fn guess_if_retry(msg: &str) -> bool {
        msg.contains("位置")
            || msg.contains("Location")
            || msg.contains("范围")
            || msg.contains("location")
    }
    fn data_helper(data: I) -> O;
}
impl<SignProtocol> SignRetry<Geoaddr, <Self as SignTrait>::Data, SignProtocol> for QrCodeSign {
    fn data_helper(data: Geoaddr) -> <Self as SignTrait>::Data {
        Some(data)
    }
}
impl<'a, SignProtocol> SignRetry<&'a Geoaddr, &'a <Self as SignTrait>::Data, SignProtocol>
    for LocationSign
{
    fn data_helper(data: &'a Geoaddr) -> &'a <Self as SignTrait>::Data {
        data
    }
}
/// 提供数据，不断进行签到，成功则返回。其通过失败时的 msg 判断是否需要重试，若无需重试，则签到失败。
pub(crate) fn sign_single_retry<
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
    captcha_solver: &CaptchaSolver,
) -> Result<SignResult, SignError>
where
    SignProtocol: SignProtocolTrait,
    Sign: SignTrait + SignRetry<InputData, Data, SignProtocol>,
    Data: Borrow<<Sign as SignTrait>::Data>,
    InputDataIter: IntoIterator<Item = InputData>,
    CaptchaProtocol: CaptchaProtocolTrait,
{
    let guess_result = sign.guess_sign_result_by_state::<SignProtocol, U>(session)?;
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
                match sign.sign::<CaptchaProtocol, SignProtocol, U>(
                    session,
                    url,
                    pre_sign_result_data,
                    pre_sign_data,
                    captcha_solver,
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
