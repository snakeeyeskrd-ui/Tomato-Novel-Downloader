//! Точка входа без UI (старый CLI).
//!
//! Взаимодействие через стандартный ввод/вывод; перед входом по возможности
//! восстанавливается режим терминала.

use std::io::{self, BufRead, Write};

use anyhow::Result;

use crossterm::event::DisableMouseCapture;
use crossterm::execute;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};

use crate::base_system::context::Config;
use crate::prewarm_state;

mod app_update;
mod config;
mod download;
mod history;
mod update;

fn show_config_menu(config: &mut Config) -> Result<()> {
    config::show_config_menu(config)
}

pub(crate) fn download_book(book_id: &str, config: &Config) -> Result<()> {
    download::download_book(book_id, config)
}

pub(crate) fn update_existing_book_non_interactive(
    book_id: &str,
    config: &Config,
    retry_failed: bool,
) -> Result<()> {
    download::update_existing_book_non_interactive(book_id, config, retry_failed)
}

pub fn run(config: &mut Config) -> Result<()> {
    // In case the previous run exited while in TUI raw mode (e.g., Ctrl+C),
    // best-effort restore the console so stdin line input works in PowerShell.
    let _ = disable_raw_mode();
    let mut out = io::stdout();
    let _ = execute!(out, DisableMouseCapture, LeaveAlternateScreen);

    println!(
        "Добро пожаловать в Tomato Novel Downloader! v{}\n\
Репозиторий: https://github.com/zhongbai2333/Tomato-Novel-Downloader \n\
Fork From: https://github.com/Dlmily/Tomato-Novel-Downloader-Lite \n\
Автор: zhongbai233 (https://github.com/zhongbai2333) \n\
Ранний код проекта: Dlmily (https://github.com/Dlmily) \n\
\n\
О проекте: это форк проекта Dlmily; код переработан и оптимизирован, добавлены возможности, в том числе: поддержка EPUB, улучшенная докачка, улучшенная обработка ошибок и др. \n\
Проект [полностью] основан на стороннем API и [не] использует официальное API; при необходимости см. проект Dlmily \n\
Проект предназначен только для изучения технологий веб-скрейпинга, обработки веб-данных и связанных исследований. Не используйте его для любых действий, нарушающих законы или права других лиц.",
        env!("CARGO_PKG_VERSION")
    );

    #[cfg(feature = "official-api")]
    println!(
        "\n【Бесплатно】Программа полностью бесплатна. Если вам предлагают платный доступ — это мошенничество!\n\
      Официальный репозиторий: https://github.com/zhongbai2333/Tomato-Novel-Downloader"
    );

    // При каждом запуске проверяем обновления программы (не влияет на дальнейший
    // ход работы; ошибки просто игнорируются).
    app_update::startup_check();

    let mut shown_iid_error: Option<String> = None;
    loop {
        if let Some(err) = prewarm_state::prewarm_error()
            && shown_iid_error.as_deref() != Some(err.as_str())
        {
            println!("\n⚠️  {}\n", err);
            shown_iid_error = Some(err);
        }

        let prompt = format!(
            "Старый CLI больше не поддерживает новые загрузки; введите команду (s — настройки / h — история загрузок / u — обновить новеллу / c — проверить обновления / U — самообновление / q — выход; сохранение по умолчанию: {}):",
            config.default_save_dir().display()
        );
        let input = read_line(&prompt)?;
        let text = input.trim();
        if text.is_empty() {
            continue;
        }
        if text.eq_ignore_ascii_case("q") {
            println!("Выход выполнен.");
            break;
        }
        if text.eq_ignore_ascii_case("s") {
            show_config_menu(config)?;
            continue;
        }
        if text.eq_ignore_ascii_case("h") {
            history::show_history_menu()?;
            continue;
        }
        if text.eq_ignore_ascii_case("u") {
            if let Some(book_id) = update::update_menu(config)? {
                println!("Выбрано обновление book_id={}\n", book_id);
                // Сразу переходим к загрузке этой книги
                match download_book(&book_id, config) {
                    Ok(()) => println!("Загрузка завершена\n"),
                    Err(err) => println!("Ошибка загрузки: {}\n", err),
                }
            }
            continue;
        }

        if text.eq_ignore_ascii_case("c") {
            app_update::check_update_menu()?;
            continue;
        }

        if text == "U" {
            let _ = crate::base_system::self_update::check_for_updates(
                env!("CARGO_PKG_VERSION"),
                false,
            );
            continue;
        }

        println!(
            "В режиме старого CLI загрузка новых новелл отключена.\nДля новых загрузок используйте TUI или Web UI; в старом CLI доступна только команда «u» для обновления уже скачанных новелл.\n"
        );
    }

    Ok(())
}

fn read_line(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line)
}
