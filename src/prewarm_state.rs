//! 启动预热状态记录。
//!
//! 用于在启动时异步预热 IID，并让 UI 能显示“预热中/完成”。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

static PREWARMING: AtomicBool = AtomicBool::new(false);
static PREWARM_ERROR: OnceLock<Mutex<Option<String>>> = OnceLock::new();

#[allow(dead_code)]
const IID_BLOCK_HINT: &str = "Для регистрации IID нужен доступ к https://log.snssdk.com/service/2/device_register/. Fanqie использует этот домен для регистрации устройства и рекламы, поэтому его часто блокируют корпоративные/кампусные сети, DNS-фильтры, правила прокси или блокировщики вроде AdGuard/uBlock. Добавьте log.snssdk.com в белый список или временно отключите блокировку и повторите попытку.";

pub fn mark_prewarm_start() {
    set_prewarm_error(None);
    PREWARMING.store(true, Ordering::SeqCst);
}

pub fn mark_prewarm_done() {
    PREWARMING.store(false, Ordering::SeqCst);
}

#[allow(dead_code)]
pub fn mark_prewarm_failed(err: impl Into<String>) {
    set_prewarm_error(Some(format_iid_register_failure(err.into())));
    PREWARMING.store(false, Ordering::SeqCst);
}

pub fn is_prewarm_in_progress() -> bool {
    PREWARMING.load(Ordering::SeqCst)
}

pub fn prewarm_error() -> Option<String> {
    PREWARM_ERROR
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

#[allow(dead_code)]
pub fn format_iid_register_failure(err: impl AsRef<str>) -> String {
    let err = err.as_ref().trim();
    if err.contains("log.snssdk.com") && err.contains("AdGuard") {
        return format!("Ошибка регистрации IID: {err}");
    }
    format!("Ошибка регистрации IID: {err}\n\n{IID_BLOCK_HINT}")
}

fn set_prewarm_error(err: Option<String>) {
    *PREWARM_ERROR
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = err;
}
