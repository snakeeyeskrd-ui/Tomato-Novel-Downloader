pub(crate) const INDEX_HTML_RAW: &str = include_str!("templates/index.html");
pub(crate) const APP_JS: &str = include_str!("templates/app.js");
pub(crate) const APP_CSS: &str = include_str!("templates/app.css");
pub(crate) const APP_FAVICON_ICO: &[u8] = include_bytes!("../../../img/Tomato-downloader-ico.ico");

/// 仅当启用 official-api feature 时才注入免费声明。
/// 未启用时展开为空串，占位符从 HTML 中抹除。
#[cfg(feature = "official-api")]
pub(crate) const FREE_NOTICE_HTML: &str = concat!(
    r#"<div class="free-notice">"#,
    "Эта программа полностью бесплатна &middot; ",
    r#"<a href="https://github.com/zhongbai2333/Tomato-Novel-Downloader" "#,
    r#"target="_blank" rel="noopener">Репозиторий</a><br />"#,
    "Если вам предлагают платную версию — это мошенничество!",
    "</div>",
);

#[cfg(not(feature = "official-api"))]
pub(crate) const FREE_NOTICE_HTML: &str = "";

/// 移动端插入到状态页底部的免费声明（仅 official-api feature 启用时非空）。
#[cfg(feature = "official-api")]
pub(crate) const FREE_NOTICE_MOBILE_HTML: &str = concat!(
    r#"<div class="free-notice free-notice-mobile">"#,
    "Эта программа полностью бесплатна &middot; ",
    r#"<a href="https://github.com/zhongbai2333/Tomato-Novel-Downloader" "#,
    r#"target="_blank" rel="noopener">Репозиторий</a><br />"#,
    "Если вам предлагают платную версию — это мошенничество!",
    "</div>",
);

#[cfg(not(feature = "official-api"))]
pub(crate) const FREE_NOTICE_MOBILE_HTML: &str = "";
