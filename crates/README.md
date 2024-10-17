# 项目结构简述

## 0

这些 crate 仅依赖于外部库。

- ### [`cxsign_error`](./cxsign_error)
  cxsign 库中所使用的错误类型。见 [lib.rs](./cxsign_error/src/lib.rs).
- ### [`cxsign_imageproc`](./cxsign_imageproc)

  与图像相关的函数。包括裁剪、下载、定位子图像等操作。见 [utils.rs](./cxsign_imageproc/src/utils.rs).

  注：二维码识别等相关操作在 `cxsign_qrcode_utils` crate 中。
- ### [`cxsign_login`](./cxsign_login)

  登录的低级 api, crate 中包含 UA 标识、基本的登录协议描述、`LoginTrait` 以及对 `ureq::Agent` 的 `LoginTrait`
  实现；也包含一个简单的 des 加密函数。见 [lib.rs](./cxsign_login/src/lib.rs) 及 [utils.rs](./cxsign_login/src/utils.rs).
- ### [`cxsign_pan`](./cxsign_pan)

  网盘相关的低级 api, crate 中包含了基本的登录协议描述和一个针对 ureq 的 multipart
  实现。见 [protocol.rs](./cxsign_pan/src/protocol.rs) 及 [multipart.rs](./cxsign_pan/src/multipart.rs).
- ### [`cxsign_unused`](./cxsign_unused)

  暂时用不到的代码。见 [protocol.rs](./cxsign_unused/src/protocol.rs).
- ### [`cxsign_utils`](./cxsign_utils)

  一些函数，包括一些时间操作、命令行询问操作等。应当重新分类至其他 crate 中。见 [lib.rs](./cxsign_utils/src/lib.rs).

## 1

- ### [`cxsign_dir`](./cxsign_dir)

  通过手动设置或应用信息（如作者、应用名称等）确定数据目录的位置。见 [`Dir`@`lib.rs:90`](./cxsign_dir/src/lib.rs).
- ### [`cxsign_obfuscate`](./cxsign_obfuscate)

  一些经过混淆过的代码，用于 `cxsign_captcha` 中。无意隐藏，仅在仓库中不可见；若使用 IDE 或熟悉 rust, 则易看到源码。
- ### [`cxsign_qrcode_utils`](./cxsign_qrcode_utils)

  二维码相关操作，包括识别图像中的二维码和基本的二维码 url 判断。在桌面平台上还提供截取屏幕并识别二维码的功能。
  见 [lib.rs](./cxsign_qrcode_utils/src/lib.rs).

## 2

- ### [`cxsign_captcha`](./cxsign_captcha)

  仅供内部使用，无公开 api.
- ### [`cxsign_user`](./cxsign_user)

  登录、Cookies 相关操作以及一个 `Session` 类型。

## 3

- ### [`cxsign_activity`](./cxsign_activity)

  获取活动列表。
- ### [`cxsign_store`](./cxsign_store)

  用户信息持久化相关 Trait.

## 4

- ### [`cxsign_sign`](./cxsign_sign)

  `SignTrait` 及一些相关类型，如签到后状态 `SignState`, 预签到结果 `PreSignResult`, 签到结果`SignResult` 等。
- ### [`cxsign_types`](./cxsign_types)

  一些数据类型，如网盘图片类型 `Photo`, 课程类型 `Course`, 地理位置类型 `Location` 等及相关操作。

## 5

- ### [`cxsign_signner`](./cxsign_signner)

  签到逻辑的抽象，即 `SignnerTrait`.

## 6

- ### [`cxsign_default_impl`](./cxsign_default_impl)

  为各类抽象提供了可覆盖的默认实现。包括各类签到类型及对应的 Signner 实现、用户信息持久化的实现等。

## 7

- ### [`cxsign_internal`](./cxsign_internal)

  对其他 crate 的重新导出。见 [lib.rs](./cxsign_internal/src/lib.rs).