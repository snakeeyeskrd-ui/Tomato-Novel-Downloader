//! Просмотр истории загрузок в режиме без UI.

use anyhow::Result;

use crate::base_system::download_history::read_download_history;

pub(super) fn show_history_menu() -> Result<()> {
    let mut keyword: Option<String> = None;

    loop {
        let items = read_download_history(50, keyword.as_deref());
        println!("\n===== История загрузок (последние 50) =====");
        if let Some(k) = keyword.as_deref() {
            println!("Ключ фильтра: {}", k);
        }

        if items.is_empty() {
            println!("Записей нет");
        } else {
            for (i, it) in items.iter().enumerate() {
                println!(
                    "{:>2}. [{}] «{}» ({}) | автор: {} | {} | статус: {}",
                    i + 1,
                    it.timestamp,
                    it.book_name,
                    it.book_id,
                    if it.author.trim().is_empty() {
                        "неизвестен"
                    } else {
                        it.author.trim()
                    },
                    it.progress,
                    it.status
                );
            }
        }

        println!("\nДействия: Enter=обновить, f=задать ключ фильтра, c=сбросить фильтр, q=назад");
        let cmd = super::read_line("Выбор: ")?;
        let cmd = cmd.trim();
        if cmd.is_empty() {
            continue;
        }
        if cmd.eq_ignore_ascii_case("q") {
            break;
        }
        if cmd.eq_ignore_ascii_case("f") {
            let q = super::read_line("Введите ключ (название/автор/ID): ")?;
            let q = q.trim();
            if q.is_empty() {
                println!("Ключ пуст, текущий фильтр сохранён.\n");
            } else {
                keyword = Some(q.to_string());
            }
            continue;
        }
        if cmd.eq_ignore_ascii_case("c") {
            keyword = None;
            continue;
        }
    }

    Ok(())
}
