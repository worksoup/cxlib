# 项目结构简述

## 0

这些 crate 仅依赖于外部库。

- ### [`cxlib_dir`](./cxlib_store)

  通过手动设置或应用信息（如作者、应用名称等）确定数据目录的位置。见 [`Dir`@`lib.rs:90`](cxlib_store/src/lib.rs).
- ### [`cxlib_error`](./cxlib_error)

  cxsign 库中所使用的错误类型。见 [lib.rs](cxlib_error/src/lib.rs).
- ### [`cxlib_imageproc`](./cxlib_imageproc)

  与图像相关的函数。包括裁剪、下载、定位子图像等操作。见 [utils.rs](cxlib_imageproc/src/utils.rs).

  注：二维码识别等相关操作在 [`cxlib_qrcode_utils`](./cxlib_qrcode_utils) crate 中。
- ### [`cxlib_store`](./cxlib_store)

  用户信息持久化相关 Trait.
- ### [`cxlib_unused`](./cxlib_unused)

  暂时用不到的代码。见 [protocol.rs](cxlib_unused/src/protocol.rs).
- ### [`cxlib_utils`](./cxlib_utils)

  一些函数，包括一些时间操作、命令行询问操作等。应当重新分类至其他 crate 中。见 [lib.rs](cxlib_utils/src/lib.rs).

## 1

- ### [`cxlib_obfuscate`](./cxlib_obfuscate)

  一些经过混淆过的代码，用于 `cxlib_captcha` 中。无意隐藏，仅在仓库中不可见；若使用 IDE 或熟悉 rust, 则易看到源码。
- ### [`cxlib_protocol`](./cxlib_protocol)

  方便管理各种网络请求。见 [README](cxlib_protocol/README.md).

## 2

- ### [`cxlib_captcha`](./cxlib_captcha)

  仅供内部使用，无公开 api.
- ### [`cxlib_login`](./cxlib_login)

  登录的低级 api, crate 中包含 UA 标识、基本的登录协议描述、`LoginTrait` 以及对 `ureq::Agent` 的 `LoginTrait`
  实现；也包含一个简单的 des 加密函数。
  见 [lib.rs](cxlib_login/src/lib.rs) 及 [utils.rs](cxlib_login/src/utils.rs).
- ### [`cxlib_pan`](./cxlib_pan)

  网盘相关的低级 api, crate 中包含了基本的登录协议描述和一个针对 ureq 的 multipart
  实现。
  见 [protocol.rs](cxlib_pan/src/protocol.rs) 及 [multipart.rs](cxlib_pan/src/multipart.rs).
- ### [`cxlib_qrcode_utils`](./cxlib_qrcode_utils)

  二维码相关操作，包括识别图像中的二维码和基本的二维码 url 判断。在桌面平台上还提供截取屏幕并识别二维码的功能。
  见 [lib.rs](cxlib_qrcode_utils/src/lib.rs).

## 3

- ### [`cxlib_user`](./cxlib_user)

  登录、Cookies 相关操作以及一个 `Session` 类型。

## 4

- ### [`cxlib_activity`](./cxlib_activity)

  获取活动列表。
- ### [`cxlib_types`](./cxlib_types)

  一些数据类型，如网盘图片类型 `Photo`, 课程类型 `Course`, 地理位置类型 `Location` 等及相关操作。

## 5

- ### [`cxlib_sign`](./cxlib_sign)

  `SignTrait` 及一些相关类型，如签到后状态 `SignState`, 预签到结果 `PreSignResult`, 签到结果`SignResult` 等。

## 6

- ### [`cxlib_signner`](./cxlib_signner)

  签到逻辑的抽象，即 `SignnerTrait`.

## 7

- ### [`cxlib_default_impl`](./cxlib_default_impl)

  为各类抽象提供了可覆盖的默认实现。包括各类签到类型及对应的 Signner 实现、用户信息持久化的实现等。

## 8

- ### [`cxlib_internal`](./cxlib_internal)

  对其他 crate 的重新导出。见 [lib.rs](cxlib_internal/src/lib.rs).