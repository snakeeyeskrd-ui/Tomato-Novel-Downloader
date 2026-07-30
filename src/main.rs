//! Tomato Novel Downloader (загрузчик новелл Tomato) — реализация на Rust.
//!
//! Этот crate отвечает за: загрузку конфигурации, интерфейс (TUI/CLI),
//! планирование загрузок, разбор контента и экспорт (txt/epub/аудиокниги и т.д.).
//!
//! Структура кода (точки входа для чтения):
//! - `base_system`: инфраструктура (конфиг / логи / повторы / пути)
//! - `download`: оркестрация загрузки (оглавление, контент, паузы / повторы)
//! - `book_parser`: разбор и экспорт (epub/txt/медиа/аудиокниги)
//! - `ui`: два режима взаимодействия — TUI и без UI (старый CLI)
//! - `prewarm_state`: состояние предпрогрева при запуске (совместно с UI)

use anyhow::{Result, anyhow};
use clap::Parser;
use std::thread;

mod base_system;
mod book_parser;
mod download;
mod network_parser;
mod prewarm_state;
mod third_party;
mod ui;

use base_system::config::{ConfigSpec, load_or_create, load_or_create_with_base};
use base_system::context::Config;
use base_system::logging::{LogOptions, LogSystem};
use tracing::info;
#[cfg(feature = "official-api")]
use tracing::warn;

#[cfg(all(feature = "official-api", feature = "no-official-api"))]
compile_error!(
    "features 'official-api' and 'no-official-api' are mutually exclusive; use exactly one"
);

#[cfg(feature = "official-api")]
use tomato_novel_official_api::prewarm_iid;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(name = "tomato-novel-downloader")]
#[command(about = "Tomato Novel Downloader (Rust TUI)")]
struct Cli {
    /// Включить отладочный вывод логов
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Включить режим сервера (Web UI)
    #[arg(long, default_value_t = false)]
    server: bool,

    /// Пароль Web UI (режим блокировки, чтобы посторонние не могли пользоваться)
    #[arg(long)]
    password: Option<String>,

    /// Добавить флаг Secure к cookie входа Web UI (рекомендуется при HTTPS / обратном прокси)
    #[arg(long, default_value_t = false)]
    cookie_secure: bool,

    /// Показать версию и выйти
    #[arg(long, default_value_t = false)]
    version: bool,

    /// Проверить и выполнить самообновление (скачать с GitHub Releases и заменить текущий исполняемый файл)
    #[arg(long, default_value_t = false)]
    self_update: bool,

    /// Автоподтверждение при самообновлении (эквивалент ввода Y)
    #[arg(long, default_value_t = false)]
    self_update_yes: bool,

    /// Путь к каталогу данных (для config.yml, logs и т.п.; удобно для монтирования в Docker)
    #[arg(long)]
    data_dir: Option<String>,

    /// Отключено: во избежание злоупотреблений CLI больше не поддерживает новые загрузки (параметр сохранён только для дружелюбной ошибки)
    #[arg(long, hide = true)]
    download: Option<String>,

    /// Обновить уже скачанную новеллу с указанным book_id (неинтерактивный режим)
    #[arg(long)]
    update: Option<String>,

    /// В неинтерактивном режиме один раз повторить главы с ошибками
    #[arg(long, default_value_t = false)]
    retry_failed: bool,
}

fn main() -> Result<()> {
    // Windows console often starts in a legacy code page; UTF-8 is needed for Chinese titles.
    #[cfg(windows)]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(["/C", "chcp 65001 >NUL"]).status();
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn SetConsoleOutputCP(w_code_page_id: u32) -> i32;
            fn SetConsoleCP(w_code_page_id: u32) -> i32;
        }
        unsafe {
            SetConsoleOutputCP(65001);
            SetConsoleCP(65001);
        }
    }

    let cli = Cli::parse();

    if cli.version {
        println!("Tomato Novel Downloader v{}", VERSION);
        return Ok(());
    }

    let data_dir = cli.data_dir.as_ref().map(std::path::Path::new);
    let _log = init_logging(cli.debug, data_dir)?;

    if cli.self_update {
        let _ = base_system::self_update::check_for_updates(VERSION, cli.self_update_yes);
        return Ok(());
    }

    if cli.download.is_some() && cli.update.is_some() {
        return Err(anyhow!("--download и --update нельзя использовать одновременно"));
    }

    // Принудительный hotfix при запуске (только если SHA256 отличается при том же tag).
    // Исключение: пропускается при cargo run / в режиме разработки.
    let _ = base_system::self_update::check_hotfix_and_apply(VERSION);

    prewarm_state::mark_prewarm_start();
    thread::spawn(|| {
        #[cfg(feature = "official-api")]
        {
            // Здесь только «предпрогрев / обеспечение доступности» — не менять IID
            // принудительно при каждом запуске. `prewarm_iid()` сначала использует
            // локальный файловый кэш и регистрирует новый IID только при отсутствии
            // или истечении кэша.
            match prewarm_iid() {
                Ok(_) => info!(target: "startup", "Предпрогрев IID завершён"),
                Err(err) => {
                    prewarm_state::mark_prewarm_failed(err.to_string());
                    if let Some(message) = prewarm_state::prewarm_error() {
                        warn!(target: "startup", "{message}");
                    }
                    return;
                }
            }
        }

        #[cfg(not(feature = "official-api"))]
        {
            info!(target: "startup", "Сборка no-official-api: предпрогрев IID пропущен");
        }
        prewarm_state::mark_prewarm_done();
    });

    let mut config = load_config_from_data_dir(data_dir)?;

    // Handle command-line download/update modes
    if cli.download.is_some() || cli.update.is_some() {
        info!(target: "startup", "Текущая версия: v{}", VERSION);

        if cli.download.is_some() {
            return Err(anyhow!(
                "Во избежание злоупотреблений новые загрузки в CLI отключены; сначала скачайте книгу через Web UI / TUI, далее обновляйте локальные новеллы только через --update."
            ));
        }

        if let Some(book_id) = cli.update.as_deref() {
            println!("Обновление указанной книги book_id={}", book_id);
            return ui::noui::update_existing_book_non_interactive(
                book_id,
                &config,
                cli.retry_failed,
            );
        }
    }

    if cli.server {
        let password = cli
            .password
            .or_else(|| std::env::var("TOMATO_WEB_PASSWORD").ok());
        let cookie_secure = cli.cookie_secure
            || parse_bool_env("TOMATO_WEB_COOKIE_SECURE")
            || parse_bool_env("TOMATO_COOKIE_SECURE");
        return ui::web::run(
            &mut config,
            password,
            config_path_from_data_dir(data_dir),
            cookie_secure,
        );
    }

    loop {
        if config.old_cli {
            info!(target: "startup", "Текущая версия: v{}", VERSION);
            return ui::noui::run(&mut config);
        }

        match ui::tui::run(config)? {
            ui::tui::TuiExit::Quit => return Ok(()),
            ui::tui::TuiExit::SwitchToOldCli => {
                // Имитация «перезапуска»: снова загрузить конфиг с диска и войти в noui
                config = load_config_from_data_dir(data_dir)?;
                config.old_cli = true;
            }
            ui::tui::TuiExit::SelfUpdate { auto_yes } => {
                let _ = base_system::self_update::check_for_updates(VERSION, auto_yes);
                return Ok(());
            }
        }
    }
}

fn load_config_from_data_dir(data_dir: Option<&std::path::Path>) -> Result<Config> {
    if let Some(dir) = data_dir {
        load_or_create_with_base::<Config>(None, Some(dir)).map_err(|e| anyhow!(e.to_string()))
    } else {
        load_or_create::<Config>(None).map_err(|e| anyhow!(e.to_string()))
    }
}

fn config_path_from_data_dir(data_dir: Option<&std::path::Path>) -> std::path::PathBuf {
    if let Some(dir) = data_dir {
        dir.join(<Config as ConfigSpec>::FILE_NAME)
    } else {
        std::path::PathBuf::from(<Config as ConfigSpec>::FILE_NAME)
    }
}

fn parse_bool_env(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn init_logging(debug: bool, base_dir: Option<&std::path::Path>) -> Result<LogSystem> {
    let opts = LogOptions {
        debug,
        use_color: true,
        archive_on_exit: true,
        console: false,
        broadcast_to_ui: true,
    };
    if let Some(base_dir) = base_dir {
        LogSystem::init_with_base(opts, Some(base_dir)).map_err(|e| anyhow!(e))
    } else {
        LogSystem::init(opts).map_err(|e| anyhow!(e))
    }
}
