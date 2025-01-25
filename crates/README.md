# 项目结构简述

## 0

这些 crate 仅依赖于外部库。

- ### [`cxlib_store`](./cxlib_store)

  用户信息持久化相关 Trait.

  例如通过手动设置或应用信息（如作者、应用名称等）确定数据目录的位置。见 [`Dir`@`lib.rs:90`](cxlib_store/src/lib.rs).
- ### [`cxlib_error`](./cxlib_error)

  cxsign 库中所使用的错误类型。见 [lib.rs](cxlib_error/src/lib.rs).
- ### [`cxlib_imageproc`](./cxlib_imageproc)

  与图像相关的函数。包括裁剪、下载、定位子图像等操作。见 [utils.rs](cxlib_imageproc/src/utils.rs).
- ### [`cxlib_obfuscate`](./cxlib_obfuscate)

  一些经过混淆过的代码，用于 `cxlib_captcha` 中。无意隐藏，仅在仓库中不可见；若使用 IDE 或熟悉 rust, 则易看到源码。
- ### [`cxlib_utils`](./cxlib_utils)

  一些函数，包括一些时间操作、命令行询问操作等。应当重新分类至其他 crate 中。见 [lib.rs](cxlib_utils/src/lib.rs).

## 1

- ### [`cxlib_protocol`](./cxlib_protocol)

  方便管理各种网络请求。见 [README](cxlib_protocol/README.md).

## 2

- ### [`cxlib_captcha`](./cxlib_captcha)

  仅供内部使用，无公开 api.
- ### [`cxlib_login`](./cxlib_login)

  登录的低级 api, crate 中包含 UA 标识、基本的登录协议描述、`LoginTrait` 以及对 `ureq::Agent` 的 `LoginTrait`
  实现；也包含一个简单的 des 加密函数。

  见 [lib.rs](cxlib_login/src/lib.rs) 及 [utils.rs](cxlib_login/src/utils.rs).

## 4

- ### [`cxlib_types`](./cxlib_types)

  一些数据类型，如网盘图片类型 `Photo`, 课程类型 `Course`, 地理位置类型 `Location` 等及相关操作。

  另见 [`cxlib_types_ext`](./cxlib_types_ext), 此后不再单独列出。

## 5

- ### [`cxlib_sign`](./cxlib_sign)

  `SignTrait` 及一些相关类型，如签到后状态 `SignState`, 预签到结果 `PreSignResult`, 签到结果`SignResult` 等。

## 6

- ### [`cxlib_default_impl`](./cxlib_default_impl)

  为各类抽象提供了可覆盖的默认实现。包括各类签到类型及对应的 Signner 实现、用户信息持久化的实现等。

## 7

- ### [`cxlib_internal`](./cxlib_internal)

  对其他 crate 的重新导出。见 [lib.rs](cxlib_internal/src/lib.rs).