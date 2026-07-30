//! Проверка обновлений новелл и подсказки в режиме без UI.

use std::io::{self, Write};
use std::path::Path;

use crate::base_system::novel_updates;
use anyhow::Result;

use crate::base_system::context::Config;

#[derive(Debug, Clone)]
pub(super) struct UpdateEntry {
    pub(super) book_id: String,
    pub(super) label: String,
}

pub(super) fn update_menu(config: &Config) -> Result<Option<String>> {
    let save_dir = config.default_save_dir();
    if !save_dir.exists() {
        println!(
            "Нет новелл для обновления (каталог сохранения не существует): {}\n",
            save_dir.display()
        );
        return Ok(None);
    }

    let (updates, no_updates) = scan_updates(config, &save_dir)?;
    if updates.is_empty() && no_updates.is_empty() {
        println!("Нет новелл для обновления\n");
        return Ok(None);
    }

    loop {
        println!("\n===== Список новелл для обновления =====");
        for (idx, u) in updates.iter().enumerate() {
            println!("{}. {}", idx + 1, u.label);
        }
        let opt_no_update = if no_updates.is_empty() {
            None
        } else {
            let n = updates.len() + 1;
            println!("{}. Без обновлений ({})", n, no_updates.len());
            Some(n)
        };
        println!("q. Выход\n");

        let sel = super::read_line("Введите номер: ")?;
        let sel = sel.trim().to_ascii_lowercase();
        if sel == "q" {
            println!("Обновление отменено\n");
            return Ok(None);
        }
        let Ok(n) = sel.parse::<usize>() else {
            println!("Ошибка: введите числовой номер или q для выхода.\n");
            continue;
        };

        if n >= 1 && n <= updates.len() {
            return Ok(Some(updates[n - 1].book_id.clone()));
        }

        if let Some(no_idx) = opt_no_update
            && n == no_idx
            && let Some(book_id) = select_from_list(&no_updates, "Книги без обновлений")?
        {
            return Ok(Some(book_id));
        }
        if let Some(no_idx) = opt_no_update
            && n == no_idx
        {
            continue;
        }

        let max = opt_no_update.unwrap_or(updates.len());
        println!(
            "Ошибка: введите число от 1 до {} или q для выхода.\n",
            max
        );
    }
}

fn select_from_list(list: &[UpdateEntry], title: &str) -> Result<Option<String>> {
    loop {
        println!("\n===== {} =====", title);
        for (idx, u) in list.iter().enumerate() {
            println!("{}. {}", idx + 1, u.label);
        }
        println!("q. Отмена и возврат в предыдущее меню\n");

        let sel = super::read_line("Введите номер: ")?;
        let sel = sel.trim().to_ascii_lowercase();
        if sel == "q" {
            return Ok(None);
        }
        let Ok(n) = sel.parse::<usize>() else {
            println!("Ошибка: введите числовой номер или q для возврата.\n");
            continue;
        };
        if n >= 1 && n <= list.len() {
            return Ok(Some(list[n - 1].book_id.clone()));
        }
        println!(
            "Ошибка: введите число от 1 до {} или q для возврата.\n",
            list.len()
        );
    }
}

fn scan_updates(_config: &Config, save_dir: &Path) -> Result<(Vec<UpdateEntry>, Vec<UpdateEntry>)> {
    println!("Начинаю сканирование обновлений (результаты показываются по ходу)…");
    let scan = novel_updates::scan_novel_updates_with_progress(save_dir, |progress| {
        let row = progress.row;
        print!(
            "\rПроверено {}/{}, сейчас: «{}» ({})      ",
            progress.scanned, progress.total, row.book_name, row.book_id
        );
        let _ = io::stdout().flush();
        if row.has_update && !row.is_ignored {
            println!("\nНайдено обновление: {}", update_label(&row));
        }
    })?;
    println!(
        "\rСканирование завершено: с обновлениями {} шт., без обновлений {} шт.      ",
        scan.updates.len(),
        scan.no_updates.len()
    );

    let to_entry = |it: novel_updates::NovelUpdateRow| UpdateEntry {
        book_id: it.book_id.clone(),
        label: update_label(&it),
    };

    Ok((
        scan.updates.into_iter().map(to_entry).collect(),
        scan.no_updates.into_iter().map(to_entry).collect(),
    ))
}

fn update_label(it: &novel_updates::NovelUpdateRow) -> String {
    let ignore_marker = if it.is_ignored {
        "[игнорируется] "
    } else {
        ""
    };
    if it.new_count > 0 && it.local_failed > 0 {
        format!(
            "{}«{}» ({}) — новых глав: {} | неудачных глав: {}",
            ignore_marker, it.book_name, it.book_id, it.new_count, it.local_failed
        )
    } else if it.new_count > 0 {
        format!(
            "{}«{}» ({}) — новых глав: {}",
            ignore_marker, it.book_name, it.book_id, it.new_count
        )
    } else if it.local_failed > 0 {
        format!(
            "{}«{}» ({}) — неудачных глав: {}",
            ignore_marker, it.book_name, it.book_id, it.local_failed
        )
    } else {
        format!(
            "{}«{}» ({}) — новых глав: 0",
            ignore_marker, it.book_name, it.book_id
        )
    }
}
