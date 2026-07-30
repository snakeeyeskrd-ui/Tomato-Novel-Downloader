//! Редактор конфигурации в режиме без UI.
//!
//! Интерактивное меню для изменения `config.yml`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow};

use crate::base_system::config::{ConfigSpec, write_with_comments};
use crate::base_system::context::{
    Config, output_format_choices, output_format_label, output_format_value_from_label,
};

#[derive(Debug, Clone, Copy)]
enum ConfigValueType {
    Bool,
    Int,
    Float,
    String,
    List,
    Selection,
}

#[derive(Debug, Clone, Copy)]
enum ConfigField {
    SavePath,
    NovelFormat,
    AutoClearDump,
    AllowOverwriteFiles,
    PreferredBookNameField,
    EnableAudiobook,
    AudiobookVoice,
    AudiobookRate,
    AudiobookVolume,
    AudiobookPitch,
    AudiobookConcurrency,
    AudiobookFormat,
    MaxWorkers,
    RequestTimeout,
    MaxRetries,
    MinWaitTime,
    MaxWaitTime,
    MinConnectTimeout,
    UseOfficialApi,
    ApiEndpoints,
    EnableSegmentComments,
    SegmentCommentsTopN,
    SegmentCommentsWorkers,
    DownloadCommentImages,
    DownloadCommentAvatars,
    MediaDownloadWorkers,
    BlockedMediaDomains,
    ForceConvertImagesToJpeg,
    JpegRetryConvert,
    JpegQuality,
    ConvertHeicToJpeg,
    KeepHeicOriginal,
    MediaLimitPerChapter,
    MediaMaxDimensionPx,
    FirstLineIndentEm,
    OldCli,
}

#[derive(Debug, Clone, Copy)]
struct ConfigOption {
    name: &'static str,
    field: ConfigField,
    ty: ConfigValueType,
}

pub(super) fn show_config_menu(config: &mut Config) -> Result<()> {
    // Порядок как в option_defs из old_main.py
    const OPTS: &[ConfigOption] = &[
        ConfigOption {
            name: "Путь сохранения",
            field: ConfigField::SavePath,
            ty: ConfigValueType::String,
        },
        ConfigOption {
            name: "Формат сохранения новеллы",
            field: ConfigField::NovelFormat,
            ty: ConfigValueType::Selection,
        },
        ConfigOption {
            name: "Автоочистка кэша",
            field: ConfigField::AutoClearDump,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Разрешить перезапись существующих файлов",
            field: ConfigField::AllowOverwriteFiles,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Предпочтительное поле названия",
            field: ConfigField::PreferredBookNameField,
            ty: ConfigValueType::Selection,
        },
        ConfigOption {
            name: "Создавать аудиокнигу",
            field: ConfigField::EnableAudiobook,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Голос аудиокниги",
            field: ConfigField::AudiobookVoice,
            ty: ConfigValueType::String,
        },
        ConfigOption {
            name: "Скорость речи аудиокниги (напр. +0%)",
            field: ConfigField::AudiobookRate,
            ty: ConfigValueType::String,
        },
        ConfigOption {
            name: "Громкость аудиокниги (напр. +0%)",
            field: ConfigField::AudiobookVolume,
            ty: ConfigValueType::String,
        },
        ConfigOption {
            name: "Высота тона аудиокниги (напр. +2Hz/-1st, можно оставить пустым)",
            field: ConfigField::AudiobookPitch,
            ty: ConfigValueType::String,
        },
        ConfigOption {
            name: "Параллелизм аудиокниги",
            field: ConfigField::AudiobookConcurrency,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Формат аудиокниги (mp3/wav)",
            field: ConfigField::AudiobookFormat,
            ty: ConfigValueType::String,
        },
        ConfigOption {
            name: "Макс. число потоков",
            field: ConfigField::MaxWorkers,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Таймаут запроса (сек)",
            field: ConfigField::RequestTimeout,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Макс. число повторов",
            field: ConfigField::MaxRetries,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Мин. время ожидания (мс)",
            field: ConfigField::MinWaitTime,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Макс. время ожидания (мс)",
            field: ConfigField::MaxWaitTime,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Мин. таймаут соединения",
            field: ConfigField::MinConnectTimeout,
            ty: ConfigValueType::Float,
        },
        ConfigOption {
            name: "Использовать официальное API",
            field: ConfigField::UseOfficialApi,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Список своих API (через запятую)",
            field: ConfigField::ApiEndpoints,
            ty: ConfigValueType::List,
        },
        ConfigOption {
            name: "Скачивать комментарии к абзацам",
            field: ConfigField::EnableSegmentComments,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Макс. комментариев на абзац",
            field: ConfigField::SegmentCommentsTopN,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Потоки для комментариев к абзацам",
            field: ConfigField::SegmentCommentsWorkers,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Скачивать изображения комментариев",
            field: ConfigField::DownloadCommentImages,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Скачивать аватары комментариев",
            field: ConfigField::DownloadCommentAvatars,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Потоки загрузки изображений комментариев",
            field: ConfigField::MediaDownloadWorkers,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Чёрный список доменов изображений (через запятую)",
            field: ConfigField::BlockedMediaDomains,
            ty: ConfigValueType::List,
        },
        ConfigOption {
            name: "Принудительно конвертировать все изображения в JPEG",
            field: ConfigField::ForceConvertImagesToJpeg,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Пытаться конвертировать не-JPEG в JPEG",
            field: ConfigField::JpegRetryConvert,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Качество JPEG (0-100)",
            field: ConfigField::JpegQuality,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Конвертировать HEIC в JPEG",
            field: ConfigField::ConvertHeicToJpeg,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Сохранять исходные HEIC-файлы",
            field: ConfigField::KeepHeicOriginal,
            ty: ConfigValueType::Bool,
        },
        ConfigOption {
            name: "Лимит медиа на главу (0 — без ограничений)",
            field: ConfigField::MediaLimitPerChapter,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Макс. длинная сторона изображения в пикселях (>0 включает)",
            field: ConfigField::MediaMaxDimensionPx,
            ty: ConfigValueType::Int,
        },
        ConfigOption {
            name: "Отступ первой строки EPUB (em)",
            field: ConfigField::FirstLineIndentEm,
            ty: ConfigValueType::Float,
        },
        ConfigOption {
            name: "Использовать старый CLI (нужен перезапуск)",
            field: ConfigField::OldCli,
            ty: ConfigValueType::Bool,
        },
    ];

    loop {
        println!("\n=== Параметры конфигурации ===");
        for (idx, opt) in OPTS.iter().enumerate() {
            let mut name = opt.name.to_string();
            if matches!(opt.field, ConfigField::EnableSegmentComments)
                && config.novel_format.eq_ignore_ascii_case("txt")
            {
                name.push_str(" (TXT не поддерживается)");
            }
            println!(
                "{}. {}: {}",
                idx + 1,
                name,
                config_value_display(config, opt.field)
            );
        }
        println!("0. Вернуться в главное меню");

        let choice = super::read_line("\nВыберите номер параметра для изменения: ")?;
        let choice = choice.trim();
        if choice == "0" {
            break;
        }
        let Ok(idx) = choice.parse::<usize>() else {
            println!("Введите числовой номер");
            continue;
        };
        if idx == 0 || idx > OPTS.len() {
            println!("Номер вне диапазона");
            continue;
        }
        let opt = OPTS[idx - 1];
        let cur = config_value_display(config, opt.field);

        let new_text = if matches!(opt.ty, ConfigValueType::Selection) {
            // Режим выбора: показать варианты и дать выбрать номер
            match show_selection_prompt(opt.field, &cur)? {
                Some(v) => v,
                None => {
                    println!("Изменение отменено");
                    continue;
                }
            }
        } else {
            let input = super::read_line(&format!(
                "Сейчас {} = {}\nВведите новое значение (пусто — отмена): ",
                opt.name, cur
            ))?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                println!("Изменение отменено");
                continue;
            }
            trimmed
        };

        apply_config_edit(config, opt, &new_text)?;

        // Сохранить в config.yml
        write_with_comments(config, Path::new(<Config as ConfigSpec>::FILE_NAME))
            .map_err(|e| anyhow!(e.to_string()))?;
        println!(
            "Обновлено {} = {}",
            opt.name,
            config_value_display(config, opt.field)
        );
    }

    Ok(())
}

fn config_value_display(config: &Config, field: ConfigField) -> String {
    match field {
        ConfigField::SavePath => config.save_path.clone(),
        ConfigField::NovelFormat => {
            output_format_label(config.current_output_format_choice()).to_string()
        }
        ConfigField::AutoClearDump => config.auto_clear_dump.to_string(),
        ConfigField::AllowOverwriteFiles => config.allow_overwrite_files.to_string(),
        ConfigField::PreferredBookNameField => {
            book_name_field_display(&config.preferred_book_name_field).to_string()
        }
        ConfigField::EnableAudiobook => config.enable_audiobook.to_string(),
        ConfigField::AudiobookVoice => config.audiobook_voice.clone(),
        ConfigField::AudiobookRate => config.audiobook_rate.clone(),
        ConfigField::AudiobookVolume => config.audiobook_volume.clone(),
        ConfigField::AudiobookPitch => config.audiobook_pitch.clone(),
        ConfigField::AudiobookConcurrency => config.audiobook_concurrency.to_string(),
        ConfigField::AudiobookFormat => config.audiobook_format.clone(),
        ConfigField::MaxWorkers => config.max_workers.to_string(),
        ConfigField::RequestTimeout => config.request_timeout.to_string(),
        ConfigField::MaxRetries => config.max_retries.to_string(),
        ConfigField::MinWaitTime => config.min_wait_time.to_string(),
        ConfigField::MaxWaitTime => config.max_wait_time.to_string(),
        ConfigField::MinConnectTimeout => config.min_connect_timeout.to_string(),
        ConfigField::UseOfficialApi => config.use_official_api.to_string(),
        ConfigField::ApiEndpoints => config.api_endpoints.join(","),
        ConfigField::EnableSegmentComments => config.enable_segment_comments.to_string(),
        ConfigField::SegmentCommentsTopN => config.segment_comments_top_n.to_string(),
        ConfigField::SegmentCommentsWorkers => config.segment_comments_workers.to_string(),
        ConfigField::DownloadCommentImages => config.download_comment_images.to_string(),
        ConfigField::DownloadCommentAvatars => config.download_comment_avatars.to_string(),
        ConfigField::MediaDownloadWorkers => config.media_download_workers.to_string(),
        ConfigField::BlockedMediaDomains => config.blocked_media_domains.join(","),
        ConfigField::ForceConvertImagesToJpeg => config.force_convert_images_to_jpeg.to_string(),
        ConfigField::JpegRetryConvert => config.jpeg_retry_convert.to_string(),
        ConfigField::JpegQuality => config.jpeg_quality.to_string(),
        ConfigField::ConvertHeicToJpeg => config.convert_heic_to_jpeg.to_string(),
        ConfigField::KeepHeicOriginal => config.keep_heic_original.to_string(),
        ConfigField::MediaLimitPerChapter => config.media_limit_per_chapter.to_string(),
        ConfigField::MediaMaxDimensionPx => config.media_max_dimension_px.to_string(),
        ConfigField::FirstLineIndentEm => config.first_line_indent_em.to_string(),
        ConfigField::OldCli => config.old_cli.to_string(),
    }
}

fn apply_config_edit(config: &mut Config, opt: ConfigOption, text: &str) -> Result<()> {
    match opt.ty {
        ConfigValueType::Bool => {
            let v = matches!(
                text.to_ascii_lowercase().as_str(),
                "true" | "1" | "yes" | "y"
            );
            set_bool(config, opt.field, v)?;
        }
        ConfigValueType::Int => {
            let v: i64 = text
                .parse()
                .map_err(|_| anyhow!("Ошибка преобразования типа: нужно целое число"))?;
            set_int(config, opt.field, v)?;
        }
        ConfigValueType::Float => {
            let v: f64 = text
                .parse()
                .map_err(|_| anyhow!("Ошибка преобразования типа: нужно число с дробной частью"))?;
            set_float(config, opt.field, v)?;
        }
        ConfigValueType::String => {
            set_string(config, opt.field, text)?;
        }
        ConfigValueType::Selection => {
            set_string(config, opt.field, text)?;
        }
        ConfigValueType::List => {
            let parts: Vec<String> = text
                .split([',', '\n'])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            set_list(config, opt.field, parts)?;
        }
    }
    Ok(())
}

fn set_bool(config: &mut Config, field: ConfigField, v: bool) -> Result<()> {
    match field {
        ConfigField::AutoClearDump => config.auto_clear_dump = v,
        ConfigField::AllowOverwriteFiles => config.allow_overwrite_files = v,
        ConfigField::EnableAudiobook => config.enable_audiobook = v,
        ConfigField::UseOfficialApi => config.use_official_api = v,
        ConfigField::EnableSegmentComments => {
            if v && config.novel_format.eq_ignore_ascii_case("txt") {
                config.novel_format = "epub".to_string();
                println!(
                    "Формат сохранения автоматически переключён на EPUB, чтобы включить комментарии к абзацам."
                );
            }
            config.enable_segment_comments = v;
        }
        ConfigField::DownloadCommentImages => config.download_comment_images = v,
        ConfigField::DownloadCommentAvatars => config.download_comment_avatars = v,
        ConfigField::ForceConvertImagesToJpeg => config.force_convert_images_to_jpeg = v,
        ConfigField::JpegRetryConvert => config.jpeg_retry_convert = v,
        ConfigField::ConvertHeicToJpeg => config.convert_heic_to_jpeg = v,
        ConfigField::KeepHeicOriginal => config.keep_heic_original = v,
        ConfigField::OldCli => config.old_cli = v,
        _ => return Err(anyhow!("Это поле не является bool")),
    }
    Ok(())
}

fn set_int(config: &mut Config, field: ConfigField, v: i64) -> Result<()> {
    match field {
        ConfigField::MaxWorkers => {
            if v <= 0 {
                return Err(anyhow!("Макс. число потоков должно быть больше 0"));
            }
            config.max_workers = v as usize;
        }
        ConfigField::RequestTimeout => {
            if v <= 0 {
                return Err(anyhow!("Таймаут запроса должен быть больше 0"));
            }
            config.request_timeout = v as u64;
        }
        ConfigField::MaxRetries => {
            if v < 0 {
                return Err(anyhow!("Макс. число повторов не может быть отрицательным"));
            }
            config.max_retries = v as u32;
        }
        ConfigField::MinWaitTime => {
            if v < 0 {
                return Err(anyhow!("Мин. время ожидания не может быть отрицательным"));
            }
            let v = v as u64;
            if v > config.max_wait_time {
                return Err(anyhow!(
                    "Мин. время ожидания не может превышать макс. время ожидания"
                ));
            }
            config.min_wait_time = v;
        }
        ConfigField::MaxWaitTime => {
            if v < 0 {
                return Err(anyhow!("Макс. время ожидания не может быть отрицательным"));
            }
            let v = v as u64;
            if v < config.min_wait_time {
                return Err(anyhow!(
                    "Макс. время ожидания не может быть меньше мин. времени ожидания"
                ));
            }
            config.max_wait_time = v;
        }
        ConfigField::AudiobookConcurrency => {
            if v <= 0 {
                return Err(anyhow!("Параллелизм аудиокниги должен быть больше 0"));
            }
            config.audiobook_concurrency = v as usize;
        }
        ConfigField::SegmentCommentsTopN => {
            if v <= 0 {
                return Err(anyhow!(
                    "Лимит комментариев к абзацам должен быть больше 0"
                ));
            }
            config.segment_comments_top_n = v as usize;
        }
        ConfigField::SegmentCommentsWorkers => {
            if v <= 0 {
                return Err(anyhow!(
                    "Число потоков для комментариев к абзацам должно быть больше 0"
                ));
            }
            config.segment_comments_workers = v as usize;
        }
        ConfigField::MediaDownloadWorkers => {
            if v <= 0 {
                return Err(anyhow!("Число медиа-потоков должно быть больше 0"));
            }
            config.media_download_workers = v as usize;
        }
        ConfigField::JpegQuality => {
            if !(0..=100).contains(&v) {
                return Err(anyhow!("Качество JPEG должно быть в диапазоне 0–100"));
            }
            config.jpeg_quality = v as u8;
        }
        ConfigField::MediaLimitPerChapter => {
            if v < 0 {
                return Err(anyhow!(
                    "Лимит медиа на главу не может быть отрицательным"
                ));
            }
            config.media_limit_per_chapter = v as usize;
        }
        ConfigField::MediaMaxDimensionPx => {
            if v < 0 {
                return Err(anyhow!(
                    "Макс. длинная сторона изображения не может быть отрицательной"
                ));
            }
            config.media_max_dimension_px = v as u32;
        }
        _ => return Err(anyhow!("Это поле не является int")),
    }
    Ok(())
}

fn set_float(config: &mut Config, field: ConfigField, v: f64) -> Result<()> {
    match field {
        ConfigField::MinConnectTimeout => {
            if v <= 0.0 {
                return Err(anyhow!("Мин. таймаут соединения должен быть больше 0"));
            }
            config.min_connect_timeout = v;
        }
        ConfigField::FirstLineIndentEm => {
            if v < 0.0 {
                return Err(anyhow!("Отступ не может быть отрицательным"));
            }
            config.first_line_indent_em = v as f32;
        }
        _ => return Err(anyhow!("Это поле не является float")),
    }
    Ok(())
}

fn set_string(config: &mut Config, field: ConfigField, v: &str) -> Result<()> {
    match field {
        ConfigField::SavePath => {
            let p = v.trim();
            if p.is_empty() {
                return Err(anyhow!("Путь сохранения не может быть пустым"));
            }
            fs::create_dir_all(p).with_context(|| format!("Не удалось создать каталог: {}", p))?;
            config.save_path = p.to_string();
        }
        ConfigField::NovelFormat => {
            let choice = output_format_value_from_label(v.trim()).unwrap_or(v.trim());
            config
                .apply_output_format_choice(choice)
                .map_err(anyhow::Error::msg)?;
            if config.novel_format == "txt" && config.enable_segment_comments {
                config.enable_segment_comments = false;
                println!(
                    "Комментарии к абзацам автоматически отключены для совместимости с форматом TXT."
                );
            }
        }
        ConfigField::AudiobookVoice => config.audiobook_voice = v.to_string(),
        ConfigField::AudiobookRate => config.audiobook_rate = v.to_string(),
        ConfigField::AudiobookVolume => config.audiobook_volume = v.to_string(),
        ConfigField::AudiobookPitch => config.audiobook_pitch = v.to_string(),
        ConfigField::AudiobookFormat => {
            let lower = v.trim().to_ascii_lowercase();
            if lower != "mp3" && lower != "wav" {
                return Err(anyhow!("Формат аудиокниги поддерживает только mp3/wav"));
            }
            config.audiobook_format = lower;
        }
        ConfigField::PreferredBookNameField => {
            // Сначала пробуем отображаемое имя, иначе — английское имя поля
            let field_name = if let Some(english) = display_to_book_name_field(v.trim()) {
                english
            } else {
                let lower = v.trim().to_ascii_lowercase();
                if lower == "book_name"
                    || lower == "original_book_name"
                    || lower == "book_short_name"
                    || lower == "ask_after_download"
                {
                    lower
                } else {
                    return Err(anyhow!(
                        "Предпочтительное поле названия поддерживает только: Название по умолчанию, Исходное название, Короткое название, Выбрать после загрузки"
                    ));
                }
            };
            config.preferred_book_name_field = field_name;
        }
        _ => return Err(anyhow!("Это поле не является string")),
    }
    Ok(())
}

fn set_list(config: &mut Config, field: ConfigField, v: Vec<String>) -> Result<()> {
    match field {
        ConfigField::ApiEndpoints => config.api_endpoints = v,
        ConfigField::BlockedMediaDomains => config.blocked_media_domains = v,
        _ => return Err(anyhow!("Это поле не является list")),
    }
    Ok(())
}

/// Преобразует английское имя поля названия в отображаемое имя.
fn book_name_field_display(field: &str) -> &'static str {
    match field {
        "book_name" => "Название по умолчанию",
        "original_book_name" => "Исходное название",
        "book_short_name" => "Короткое название",
        "ask_after_download" => "Выбрать после загрузки",
        _ => "Название по умолчанию",
    }
}

/// Преобразует отображаемое имя поля названия в английское имя.
fn display_to_book_name_field(display: &str) -> Option<String> {
    match display {
        "Название по умолчанию" => Some("book_name".to_string()),
        "Исходное название" => Some("original_book_name".to_string()),
        "Короткое название" => Some("book_short_name".to_string()),
        "Выбрать после загрузки" => Some("ask_after_download".to_string()),
        _ => None,
    }
}

/// Режим выбора: показать варианты и дать выбрать номер.
fn show_selection_prompt(field: ConfigField, current: &str) -> Result<Option<String>> {
    match field {
        ConfigField::NovelFormat => {
            println!("\nСейчас: {}", current);
            for (idx, (_, label)) in output_format_choices().iter().enumerate() {
                println!("  {}. {}", idx + 1, label);
            }
            println!("  0. Отмена");
            let choice = super::read_line("Выберите: ")?;
            let choice = choice.trim();
            if choice == "0" || choice.is_empty() {
                return Ok(None);
            }
            let Ok(idx) = choice.parse::<usize>() else {
                println!("Введите числовой номер");
                return Ok(None);
            };
            if idx == 0 || idx > output_format_choices().len() {
                println!("Номер вне диапазона");
                return Ok(None);
            }
            Ok(Some(output_format_choices()[idx - 1].0.to_string()))
        }
        ConfigField::PreferredBookNameField => {
            const OPTIONS: &[(&str, &str)] = &[
                ("Название по умолчанию", "book_name"),
                ("Исходное название", "original_book_name"),
                ("Короткое название", "book_short_name"),
                ("Выбрать после загрузки", "ask_after_download"),
            ];
            println!("\nСейчас: {}", current);
            for (idx, (label, _)) in OPTIONS.iter().enumerate() {
                println!("  {}. {}", idx + 1, label);
            }
            println!("  0. Отмена");
            let choice = super::read_line("Выберите: ")?;
            let choice = choice.trim();
            if choice == "0" || choice.is_empty() {
                return Ok(None);
            }
            let Ok(idx) = choice.parse::<usize>() else {
                println!("Введите числовой номер");
                return Ok(None);
            };
            if idx == 0 || idx > OPTIONS.len() {
                println!("Номер вне диапазона");
                return Ok(None);
            }
            Ok(Some(OPTIONS[idx - 1].0.to_string()))
        }
        _ => Ok(None),
    }
}
