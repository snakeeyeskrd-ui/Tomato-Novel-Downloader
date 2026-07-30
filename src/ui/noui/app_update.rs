//! Проверка обновлений программы и подсказки в режиме без UI (старый CLI).

use anyhow::Result;

use crate::base_system::app_update;

pub(super) fn startup_check() {
    let report = match app_update::check_update_report_blocking_with_timeout(
        env!("CARGO_PKG_VERSION"),
        std::time::Duration::from_secs(3),
    ) {
        Ok(r) => r,
        Err(_) => return,
    };

    if !app_update::should_notify_startup(&report) {
        return;
    }

    println!(
        "\nПодсказка: обнаружена новая версия {} (текущая {}). Введите c, чтобы посмотреть журнал изменений; введите U для самообновления (если доступно).\n",
        report.latest.tag_name, report.current_tag
    );

    if let Some(body) = report.latest.body.as_deref() {
        let preview = preview_notes(body, 8, 800);
        if !preview.trim().is_empty() {
            println!("Журнал изменений (фрагмент):\n{}\n", preview);
        }
    }
}

pub(super) fn check_update_menu() -> Result<()> {
    let report = app_update::check_update_report_blocking(env!("CARGO_PKG_VERSION"))?;

    println!("\n===== Проверка обновлений программы =====");
    println!("Текущая версия: {}", report.current_tag);
    println!("Последняя версия: {}", report.latest.tag_name);

    if report.is_new_version {
        println!("Статус: есть новая версия");
    } else {
        println!("Статус: уже последняя версия");
    }

    if let Some(url) = report.latest.html_url.as_deref()
        && !url.trim().is_empty()
    {
        println!("Release: {}", url);
    }

    if let Some(body) = report.latest.body.as_deref() {
        let text = body.trim();
        if !text.is_empty() {
            println!("\n----- Журнал изменений -----\n{}\n--------------------", text);
        }
    }

    if report.is_new_version {
        let dismissed = app_update::dismissed_release_tag();
        if dismissed.as_deref() == Some(&report.latest.tag_name) {
            println!("Подсказка: вы уже отключили напоминание об этой версии (ручная проверка по-прежнему доступна).");
        }

        let ans = super::read_line("Отключить напоминания об этой версии? [y/N]: ")?;
        let ans = ans.trim().to_ascii_lowercase();
        if ans == "y" || ans == "yes" {
            app_update::dismiss_release_tag(&report.latest.tag_name)?;
            println!("Готово: больше не напоминать о {}\n", report.latest.tag_name);
        }
    }

    Ok(())
}

fn preview_notes(body: &str, max_lines: usize, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, line) in body.lines().enumerate() {
        if i >= max_lines {
            out.push('…');
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
        if out.len() >= max_chars {
            // Безопасно обрезаем по границе символа, чтобы многобайтовые
            // символы не вызывали panic при truncate.
            let mut end = max_chars;
            while !out.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            out.truncate(end);
            out.push('…');
            break;
        }
    }
    out
}
