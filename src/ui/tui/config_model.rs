//! TUI config model and edit logic.
//!
//! Maps `Config` to displayable/editable fields and writes back `config.yml`.

use std::path::Path;

use anyhow::{Result, anyhow};

use crate::base_system::config::{ConfigSpec, write_with_comments};
use crate::base_system::context::{
    Config, OUTPUT_FORMAT_BULK_TXT, OUTPUT_FORMAT_PDF, output_format_value_from_label,
};

use super::App;

#[derive(Debug, Clone, Copy)]
pub(in crate::ui) enum ConfigField {
    SavePath,
    NovelFormat,
    AutoClearDump,
    AutoOpenDownloadedFiles,
    AllowOverwriteFiles,
    PreferredBookNameField,
    OldCli,
    FirstLineIndentEm,
    EnableSegmentComments,
    UseOfficialApi,
    ApiEndpoints,
    MaxWorkers,
    RequestTimeout,
    MaxRetries,
    MinConnectTimeout,
    MinWait,
    MaxWait,
    EnableAudiobook,
    AudiobookVoice,
    AudiobookRate,
    AudiobookVolume,
    AudiobookPitch,
    AudiobookFormat,
    AudiobookConcurrency,
    AudiobookTtsProvider,
    AudiobookTtsApiUrl,
    AudiobookTtsApiToken,
    AudiobookTtsModel,
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
}

#[derive(Debug, Clone)]
pub(in crate::ui) struct ConfigEntry {
    pub(in crate::ui) title: &'static str,
    pub(in crate::ui) field: ConfigField,
}

#[derive(Debug, Clone)]
pub(in crate::ui) struct ConfigCategory {
    pub(in crate::ui) title: &'static str,
    pub(in crate::ui) entries: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::ui) struct VoicePreset {
    pub(in crate::ui) name: &'static str,
    pub(in crate::ui) label: &'static str,
}

pub(in crate::ui) const AUDIOBOOK_VOICE_PRESETS: &[VoicePreset] = &[
    VoicePreset {
        name: "zh-CN-XiaoxiaoNeural",
        label: "zh-CN-XiaoxiaoNeural (жен.)",
    },
    VoicePreset {
        name: "zh-CN-XiaoyiNeural",
        label: "zh-CN-XiaoyiNeural (жен.)",
    },
    VoicePreset {
        name: "zh-CN-YunjianNeural",
        label: "zh-CN-YunjianNeural (муж.)",
    },
    VoicePreset {
        name: "zh-CN-YunxiNeural",
        label: "zh-CN-YunxiNeural (муж.)",
    },
    VoicePreset {
        name: "zh-CN-YunxiaNeural",
        label: "zh-CN-YunxiaNeural (муж.)",
    },
    VoicePreset {
        name: "zh-CN-YunyangNeural",
        label: "zh-CN-YunyangNeural (муж.)",
    },
    VoicePreset {
        name: "zh-CN-liaoning-XiaobeiNeural",
        label: "zh-CN-liaoning-XiaobeiNeural (жен.)",
    },
    VoicePreset {
        name: "zh-CN-shaanxi-XiaoniNeural",
        label: "zh-CN-shaanxi-XiaoniNeural (жен.)",
    },
    VoicePreset {
        name: "zh-HK-HiuGaaiNeural",
        label: "zh-HK-HiuGaaiNeural (жен.)",
    },
    VoicePreset {
        name: "zh-HK-HiuMaanNeural",
        label: "zh-HK-HiuMaanNeural (жен.)",
    },
    VoicePreset {
        name: "zh-HK-WanLungNeural",
        label: "zh-HK-WanLungNeural (муж.)",
    },
    VoicePreset {
        name: "zh-TW-HsiaoChenNeural",
        label: "zh-TW-HsiaoChenNeural (жен.)",
    },
];

pub(in crate::ui) const BOOK_NAME_FIELD_PRESETS: &[VoicePreset] = &[
    VoicePreset {
        name: "book_name",
        label: "Название по умолчанию",
    },
    VoicePreset {
        name: "original_book_name",
        label: "Оригинальное название",
    },
    VoicePreset {
        name: "book_short_name",
        label: "Короткое название",
    },
    VoicePreset {
        name: "ask_after_download",
        label: "Спросить после загрузки",
    },
];

pub(in crate::ui) const NOVEL_FORMAT_PRESETS: &[VoicePreset] = &[
    VoicePreset {
        name: "txt",
        label: "Формат txt",
    },
    VoicePreset {
        name: "epub",
        label: "Формат epub",
    },
    VoicePreset {
        name: OUTPUT_FORMAT_PDF,
        label: "Формат pdf",
    },
    VoicePreset {
        name: OUTPUT_FORMAT_BULK_TXT,
        label: "Отдельные файлы",
    },
    VoicePreset {
        name: "ask_after_download",
        label: "Спросить после загрузки",
    },
];

pub(in crate::ui) fn cfg_field_is_combo(field: ConfigField) -> bool {
    matches!(
        field,
        ConfigField::AudiobookVoice
            | ConfigField::PreferredBookNameField
            | ConfigField::NovelFormat
    )
}

pub(in crate::ui) fn cfg_combo_presets(field: ConfigField) -> Option<&'static [VoicePreset]> {
    match field {
        ConfigField::AudiobookVoice => Some(AUDIOBOOK_VOICE_PRESETS),
        ConfigField::PreferredBookNameField => Some(BOOK_NAME_FIELD_PRESETS),
        ConfigField::NovelFormat => Some(NOVEL_FORMAT_PRESETS),
        _ => None,
    }
}

pub(in crate::ui) fn build_config_categories() -> Vec<ConfigCategory> {
    vec![
        ConfigCategory {
            title: "Основное и формат",
            entries: vec![
                ConfigEntry {
                    title: "Путь сохранения",
                    field: ConfigField::SavePath,
                },
                ConfigEntry {
                    title: "Формат книги",
                    field: ConfigField::NovelFormat,
                },
                ConfigEntry {
                    title: "Отступ первой строки (em)",
                    field: ConfigField::FirstLineIndentEm,
                },
                ConfigEntry {
                    title: "Автоочистка кэша",
                    field: ConfigField::AutoClearDump,
                },
                ConfigEntry {
                    title: "Открывать после загрузки",
                    field: ConfigField::AutoOpenDownloadedFiles,
                },
                ConfigEntry {
                    title: "Разрешить перезапись файлов",
                    field: ConfigField::AllowOverwriteFiles,
                },
                ConfigEntry {
                    title: "Предпочтительное поле названия",
                    field: ConfigField::PreferredBookNameField,
                },
                ConfigEntry {
                    title: "Старый CLI UI",
                    field: ConfigField::OldCli,
                },
            ],
        },
        ConfigCategory {
            title: "Сеть и планирование",
            entries: vec![
                ConfigEntry {
                    title: "Макс. потоков",
                    field: ConfigField::MaxWorkers,
                },
                ConfigEntry {
                    title: "Таймаут запроса (с)",
                    field: ConfigField::RequestTimeout,
                },
                ConfigEntry {
                    title: "Макс. повторов",
                    field: ConfigField::MaxRetries,
                },
                ConfigEntry {
                    title: "Мин. таймаут соединения (с)",
                    field: ConfigField::MinConnectTimeout,
                },
                ConfigEntry {
                    title: "Мин. ожидание (мс)",
                    field: ConfigField::MinWait,
                },
                ConfigEntry {
                    title: "Макс. ожидание (мс)",
                    field: ConfigField::MaxWait,
                },
            ],
        },
        ConfigCategory {
            title: "API",
            entries: vec![
                ConfigEntry {
                    title: "Использовать официальный API",
                    field: ConfigField::UseOfficialApi,
                },
                ConfigEntry {
                    title: "Список API (через запятую)",
                    field: ConfigField::ApiEndpoints,
                },
            ],
        },
        ConfigCategory {
            title: "Комментарии к абзацам",
            entries: vec![
                ConfigEntry {
                    title: "Включить комментарии к абзацам",
                    field: ConfigField::EnableSegmentComments,
                },
                ConfigEntry {
                    title: "Лимит комментариев на абзац",
                    field: ConfigField::SegmentCommentsTopN,
                },
                ConfigEntry {
                    title: "Потоки для комментариев",
                    field: ConfigField::SegmentCommentsWorkers,
                },
            ],
        },
        ConfigCategory {
            title: "Медиа",
            entries: vec![
                ConfigEntry {
                    title: "Скачивать картинки комментариев",
                    field: ConfigField::DownloadCommentImages,
                },
                ConfigEntry {
                    title: "Скачивать аватары комментариев",
                    field: ConfigField::DownloadCommentAvatars,
                },
                ConfigEntry {
                    title: "Потоки загрузки медиа",
                    field: ConfigField::MediaDownloadWorkers,
                },
                ConfigEntry {
                    title: "Блокируемые домены картинок",
                    field: ConfigField::BlockedMediaDomains,
                },
                ConfigEntry {
                    title: "Принудительно в JPEG",
                    field: ConfigField::ForceConvertImagesToJpeg,
                },
                ConfigEntry {
                    title: "При ошибке снова в JPEG",
                    field: ConfigField::JpegRetryConvert,
                },
                ConfigEntry {
                    title: "Качество JPEG (0-100)",
                    field: ConfigField::JpegQuality,
                },
                ConfigEntry {
                    title: "HEIC в JPEG",
                    field: ConfigField::ConvertHeicToJpeg,
                },
                ConfigEntry {
                    title: "Сохранять оригинал HEIC",
                    field: ConfigField::KeepHeicOriginal,
                },
                ConfigEntry {
                    title: "Лимит медиа на главу",
                    field: ConfigField::MediaLimitPerChapter,
                },
                ConfigEntry {
                    title: "Макс. размер медиа (px)",
                    field: ConfigField::MediaMaxDimensionPx,
                },
            ],
        },
        ConfigCategory {
            title: "Аудиокнига",
            entries: vec![
                ConfigEntry {
                    title: "Включить аудиокнигу",
                    field: ConfigField::EnableAudiobook,
                },
                ConfigEntry {
                    title: "Голос",
                    field: ConfigField::AudiobookVoice,
                },
                ConfigEntry {
                    title: "Тип TTS (edge/third_party)",
                    field: ConfigField::AudiobookTtsProvider,
                },
                ConfigEntry {
                    title: "URL стороннего TTS API",
                    field: ConfigField::AudiobookTtsApiUrl,
                },
                ConfigEntry {
                    title: "Токен стороннего TTS",
                    field: ConfigField::AudiobookTtsApiToken,
                },
                ConfigEntry {
                    title: "Модель стороннего TTS",
                    field: ConfigField::AudiobookTtsModel,
                },
                ConfigEntry {
                    title: "Скорость речи",
                    field: ConfigField::AudiobookRate,
                },
                ConfigEntry {
                    title: "Громкость",
                    field: ConfigField::AudiobookVolume,
                },
                ConfigEntry {
                    title: "Высота тона",
                    field: ConfigField::AudiobookPitch,
                },
                ConfigEntry {
                    title: "Формат вывода (mp3/wav)",
                    field: ConfigField::AudiobookFormat,
                },
                ConfigEntry {
                    title: "Параллельных глав генерации",
                    field: ConfigField::AudiobookConcurrency,
                },
            ],
        },
    ]
}

pub(in crate::ui) fn current_cfg_value(app: &App, field: ConfigField) -> String {
    match field {
        ConfigField::SavePath => app.config.save_path.clone(),
        ConfigField::NovelFormat => {
            output_format_label_ru(app.config.current_output_format_choice()).to_string()
        }
        ConfigField::FirstLineIndentEm => format!("{:.2}", app.config.first_line_indent_em),
        ConfigField::AutoClearDump => app.config.auto_clear_dump.to_string(),
        ConfigField::AutoOpenDownloadedFiles => app.config.auto_open_downloaded_files.to_string(),
        ConfigField::AllowOverwriteFiles => app.config.allow_overwrite_files.to_string(),
        ConfigField::PreferredBookNameField => {
            book_name_field_to_chinese(&app.config.preferred_book_name_field).to_string()
        }
        ConfigField::OldCli => app.config.old_cli.to_string(),
        ConfigField::EnableSegmentComments => app.config.enable_segment_comments.to_string(),
        ConfigField::UseOfficialApi => app.config.use_official_api.to_string(),
        ConfigField::ApiEndpoints => app.config.api_endpoints.join(","),
        ConfigField::MaxWorkers => app.config.max_workers.to_string(),
        ConfigField::RequestTimeout => app.config.request_timeout.to_string(),
        ConfigField::MaxRetries => app.config.max_retries.to_string(),
        ConfigField::MinConnectTimeout => format!("{:.2}", app.config.min_connect_timeout),
        ConfigField::MinWait => app.config.min_wait_time.to_string(),
        ConfigField::MaxWait => app.config.max_wait_time.to_string(),
        ConfigField::EnableAudiobook => app.config.enable_audiobook.to_string(),
        ConfigField::AudiobookVoice => app.config.audiobook_voice.clone(),
        ConfigField::AudiobookRate => app.config.audiobook_rate.clone(),
        ConfigField::AudiobookVolume => app.config.audiobook_volume.clone(),
        ConfigField::AudiobookPitch => app.config.audiobook_pitch.clone(),
        ConfigField::AudiobookFormat => app.config.audiobook_format.clone(),
        ConfigField::AudiobookConcurrency => app.config.audiobook_concurrency.to_string(),
        ConfigField::AudiobookTtsProvider => app.config.audiobook_tts_provider.clone(),
        ConfigField::AudiobookTtsApiUrl => app.config.audiobook_tts_api_url.clone(),
        ConfigField::AudiobookTtsApiToken => app.config.audiobook_tts_api_token.clone(),
        ConfigField::AudiobookTtsModel => app.config.audiobook_tts_model.clone(),
        ConfigField::SegmentCommentsTopN => app.config.segment_comments_top_n.to_string(),
        ConfigField::SegmentCommentsWorkers => app.config.segment_comments_workers.to_string(),
        ConfigField::DownloadCommentImages => app.config.download_comment_images.to_string(),
        ConfigField::DownloadCommentAvatars => app.config.download_comment_avatars.to_string(),
        ConfigField::MediaDownloadWorkers => app.config.media_download_workers.to_string(),
        ConfigField::BlockedMediaDomains => app.config.blocked_media_domains.join(","),
        ConfigField::ForceConvertImagesToJpeg => {
            app.config.force_convert_images_to_jpeg.to_string()
        }
        ConfigField::JpegRetryConvert => app.config.jpeg_retry_convert.to_string(),
        ConfigField::JpegQuality => app.config.jpeg_quality.to_string(),
        ConfigField::ConvertHeicToJpeg => app.config.convert_heic_to_jpeg.to_string(),
        ConfigField::KeepHeicOriginal => app.config.keep_heic_original.to_string(),
        ConfigField::MediaLimitPerChapter => app.config.media_limit_per_chapter.to_string(),
        ConfigField::MediaMaxDimensionPx => app.config.media_max_dimension_px.to_string(),
    }
}

pub(in crate::ui) fn cfg_field_is_bool(field: ConfigField) -> bool {
    matches!(
        field,
        ConfigField::AutoClearDump
            | ConfigField::AutoOpenDownloadedFiles
            | ConfigField::AllowOverwriteFiles
            | ConfigField::OldCli
            | ConfigField::EnableSegmentComments
            | ConfigField::UseOfficialApi
            | ConfigField::EnableAudiobook
            | ConfigField::DownloadCommentImages
            | ConfigField::DownloadCommentAvatars
            | ConfigField::ForceConvertImagesToJpeg
            | ConfigField::JpegRetryConvert
            | ConfigField::ConvertHeicToJpeg
            | ConfigField::KeepHeicOriginal
    )
}

fn cfg_field_current_bool(app: &App, field: ConfigField) -> Option<bool> {
    let val = match field {
        ConfigField::AutoClearDump => app.config.auto_clear_dump,
        ConfigField::AutoOpenDownloadedFiles => app.config.auto_open_downloaded_files,
        ConfigField::AllowOverwriteFiles => app.config.allow_overwrite_files,
        ConfigField::OldCli => app.config.old_cli,
        ConfigField::EnableSegmentComments => app.config.enable_segment_comments,
        ConfigField::UseOfficialApi => app.config.use_official_api,
        ConfigField::EnableAudiobook => app.config.enable_audiobook,
        ConfigField::DownloadCommentImages => app.config.download_comment_images,
        ConfigField::DownloadCommentAvatars => app.config.download_comment_avatars,
        ConfigField::ForceConvertImagesToJpeg => app.config.force_convert_images_to_jpeg,
        ConfigField::JpegRetryConvert => app.config.jpeg_retry_convert,
        ConfigField::ConvertHeicToJpeg => app.config.convert_heic_to_jpeg,
        ConfigField::KeepHeicOriginal => app.config.keep_heic_original,
        _ => return None,
    };
    Some(val)
}

pub(in crate::ui) fn start_cfg_edit(app: &mut App) {
    let Some(cat_idx) = app.cfg_cat_state.selected() else {
        return;
    };
    let Some(entry_idx) = app.cfg_entry_state.selected() else {
        return;
    };
    let Some(category) = app.cfg_categories.get(cat_idx) else {
        return;
    };
    if entry_idx >= category.entries.len() {
        return;
    }
    let entry = &category.entries[entry_idx];
    app.cfg_editing = Some((cat_idx, entry_idx));
    app.cfg_edit_buffer = current_cfg_value(app, entry.field);
    if cfg_field_is_bool(entry.field) {
        let selected = match cfg_field_current_bool(app, entry.field) {
            Some(true) => Some(0),
            Some(false) => Some(1),
            None => Some(0),
        };
        app.cfg_bool_state.select(selected);
    }
    if cfg_field_is_combo(entry.field) {
        app.cfg_combo_focus = super::ConfigComboFocus::List;
        if let Some(presets) = cfg_combo_presets(entry.field) {
            let idx = presets
                .iter()
                .position(|p| p.name.eq_ignore_ascii_case(&app.cfg_edit_buffer))
                .or(Some(0));
            app.cfg_combo_state.select(idx);
        }
    }
    app.status = format!("Правка [{}]: {}", category.title, entry.title);
}

pub(in crate::ui) fn apply_cfg_edit(app: &mut App, cat_idx: usize, entry_idx: usize) -> Result<()> {
    let Some(category) = app.cfg_categories.get(cat_idx) else {
        return Ok(());
    };
    if entry_idx >= category.entries.len() {
        return Ok(());
    }
    let field = category.entries[entry_idx].field;
    let entry_title = category.entries[entry_idx].title;
    let raw = app.cfg_edit_buffer.trim();

    let mut note: Option<String> = None;

    match field {
        ConfigField::SavePath => {
            app.config.save_path = raw.to_string();
        }
        ConfigField::NovelFormat => {
            let field_name = if let Some(english) = chinese_to_novel_format(raw) {
                english
            } else {
                let lower = raw.to_ascii_lowercase();
                if lower == "txt"
                    || lower == "epub"
                    || lower == "pdf"
                    || lower == OUTPUT_FORMAT_BULK_TXT
                    || lower == "ask_after_download"
                {
                    lower
                } else {
                    app.status = "Выберите: Формат txt, Формат epub, Формат pdf, Отдельные файлы или Спросить после загрузки"
                        .to_string();
                    return Ok(());
                }
            };
            app.config
                .apply_output_format_choice(&field_name)
                .map_err(anyhow::Error::msg)?;
            if app.config.novel_format == "txt" && app.config.enable_segment_comments {
                app.config.enable_segment_comments = false;
                note = Some("Комментарии к абзацам отключены для совместимости с txt".to_string());
            }
        }
        ConfigField::FirstLineIndentEm => {
            let val: f32 = raw.parse().map_err(|_| anyhow!("Введите число"))?;
            if val.is_sign_negative() {
                app.status = "Отступ не может быть отрицательным".to_string();
                return Ok(());
            }
            app.config.first_line_indent_em = val;
        }
        ConfigField::AutoClearDump => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.auto_clear_dump = val;
        }
        ConfigField::AutoOpenDownloadedFiles => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.auto_open_downloaded_files = val;
        }
        ConfigField::AllowOverwriteFiles => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.allow_overwrite_files = val;
        }
        ConfigField::PreferredBookNameField => {
            // Try converting from display label; fall back to English field names
            let field_name = if let Some(english) = chinese_to_book_name_field(raw) {
                english
            } else {
                // If not a known label, check for a valid English field name
                let lower = raw.to_ascii_lowercase();
                if lower == "book_name"
                    || lower == "original_book_name"
                    || lower == "book_short_name"
                    || lower == "ask_after_download"
                {
                    lower
                } else {
                    app.status = "Выберите: Название по умолчанию, Оригинальное название, Короткое название или Спросить после загрузки".to_string();
                    return Ok(());
                }
            };
            app.config.preferred_book_name_field = field_name;
        }
        ConfigField::OldCli => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.old_cli = val;
        }
        ConfigField::EnableSegmentComments => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            if val && !app.config.novel_format.eq_ignore_ascii_case("epub") {
                app.status = "Комментарии к абзацам только для epub — сначала смените формат на epub".to_string();
                return Ok(());
            }
            app.config.enable_segment_comments = val;
        }
        ConfigField::UseOfficialApi => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.use_official_api = val;
        }
        ConfigField::ApiEndpoints => {
            let list = parse_string_list(raw);
            app.config.api_endpoints = list;
        }
        ConfigField::MaxWorkers => {
            let val: usize = raw.parse().map_err(|_| anyhow!("Введите целое положительное число"))?;
            if val == 0 {
                app.status = "Макс. потоков должно быть больше 0".to_string();
                return Ok(());
            }
            app.config.max_workers = val;
        }
        ConfigField::RequestTimeout => {
            let val: u64 = raw.parse().map_err(|_| anyhow!("Введите число секунд"))?;
            if val == 0 {
                app.status = "Таймаут должен быть больше 0".to_string();
                return Ok(());
            }
            app.config.request_timeout = val;
        }
        ConfigField::MaxRetries => {
            let val: u32 = raw.parse().map_err(|_| anyhow!("Введите целое число"))?;
            app.config.max_retries = val;
        }
        ConfigField::MinConnectTimeout => {
            let val: f64 = raw.parse().map_err(|_| anyhow!("Введите число"))?;
            if val <= 0.0 {
                app.status = "Таймаут соединения должен быть больше 0".to_string();
                return Ok(());
            }
            app.config.min_connect_timeout = val;
        }
        ConfigField::MinWait => {
            let val: u64 = raw.parse().map_err(|_| anyhow!("Введите целое число миллисекунд"))?;
            if val > app.config.max_wait_time {
                app.status = "Мин. ожидание не может превышать макс. ожидание".to_string();
                return Ok(());
            }
            app.config.min_wait_time = val;
        }
        ConfigField::MaxWait => {
            let val: u64 = raw.parse().map_err(|_| anyhow!("Введите целое число миллисекунд"))?;
            if val < app.config.min_wait_time {
                app.status = "Макс. ожидание не может быть меньше мин. ожидания".to_string();
                return Ok(());
            }
            app.config.max_wait_time = val;
        }
        ConfigField::EnableAudiobook => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.enable_audiobook = val;
        }
        ConfigField::AudiobookVoice => {
            app.config.audiobook_voice = raw.to_string();
        }
        ConfigField::AudiobookRate => {
            app.config.audiobook_rate = raw.to_string();
        }
        ConfigField::AudiobookVolume => {
            app.config.audiobook_volume = raw.to_string();
        }
        ConfigField::AudiobookPitch => {
            app.config.audiobook_pitch = raw.to_string();
        }
        ConfigField::AudiobookFormat => {
            let lower = raw.to_ascii_lowercase();
            if lower != "mp3" && lower != "wav" {
                app.status = "Формат только mp3 или wav".to_string();
                return Ok(());
            }
            app.config.audiobook_format = lower;
        }
        ConfigField::AudiobookConcurrency => {
            let val: usize = raw.parse().map_err(|_| anyhow!("Введите целое положительное число"))?;
            if val == 0 {
                app.status = "Число параллельных глав должно быть больше 0".to_string();
                return Ok(());
            }
            app.config.audiobook_concurrency = val;
        }
        ConfigField::AudiobookTtsProvider => {
            app.config.audiobook_tts_provider = raw.to_string();
        }
        ConfigField::AudiobookTtsApiUrl => {
            app.config.audiobook_tts_api_url = raw.to_string();
        }
        ConfigField::AudiobookTtsApiToken => {
            app.config.audiobook_tts_api_token = raw.to_string();
        }
        ConfigField::AudiobookTtsModel => {
            app.config.audiobook_tts_model = raw.to_string();
        }
        ConfigField::SegmentCommentsTopN => {
            let val: usize = raw.parse().map_err(|_| anyhow!("Введите целое число"))?;
            if val == 0 {
                app.status = "Лимит комментариев должен быть больше 0".to_string();
                return Ok(());
            }
            app.config.segment_comments_top_n = val;
        }
        ConfigField::SegmentCommentsWorkers => {
            let val: usize = raw.parse().map_err(|_| anyhow!("Введите целое положительное число"))?;
            if val == 0 {
                app.status = "Число потоков комментариев должно быть больше 0".to_string();
                return Ok(());
            }
            app.config.segment_comments_workers = val;
        }
        ConfigField::DownloadCommentImages => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.download_comment_images = val;
        }
        ConfigField::DownloadCommentAvatars => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.download_comment_avatars = val;
        }
        ConfigField::MediaDownloadWorkers => {
            let val: usize = raw.parse().map_err(|_| anyhow!("Введите целое положительное число"))?;
            if val == 0 {
                app.status = "Число потоков медиа должно быть больше 0".to_string();
                return Ok(());
            }
            app.config.media_download_workers = val;
        }
        ConfigField::BlockedMediaDomains => {
            app.config.blocked_media_domains = parse_string_list(raw);
        }
        ConfigField::ForceConvertImagesToJpeg => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.force_convert_images_to_jpeg = val;
        }
        ConfigField::JpegRetryConvert => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.jpeg_retry_convert = val;
        }
        ConfigField::JpegQuality => {
            let val: u8 = raw
                .parse()
                .map_err(|_| anyhow!("Введите целое число от 0 до 100"))?;
            if val > 100 {
                app.status = "Качество JPEG должно быть от 0 до 100".to_string();
                return Ok(());
            }
            app.config.jpeg_quality = val;
        }
        ConfigField::ConvertHeicToJpeg => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.convert_heic_to_jpeg = val;
        }
        ConfigField::KeepHeicOriginal => {
            let val = parse_bool(raw).ok_or_else(|| anyhow!("Введите true/false"))?;
            app.config.keep_heic_original = val;
        }
        ConfigField::MediaLimitPerChapter => {
            let val: usize = raw.parse().map_err(|_| anyhow!("Введите целое число"))?;
            app.config.media_limit_per_chapter = val;
        }
        ConfigField::MediaMaxDimensionPx => {
            let val: u32 = raw.parse().map_err(|_| anyhow!("Введите целое число"))?;
            app.config.media_max_dimension_px = val;
        }
    }

    let path = Path::new(Config::FILE_NAME);
    write_with_comments(&app.config, path).map_err(|e| anyhow!(e.to_string()))?;
    match note {
        Some(extra) => app.status = format!("Сохранено: {} ({})", entry_title, extra),
        None => app.status = format!("Сохранено: {}", entry_title),
    }
    Ok(())
}

fn parse_bool(input: &str) -> Option<bool> {
    match input.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

fn parse_string_list(input: &str) -> Vec<String> {
    input
        .split([',', ';', '\n'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}


fn output_format_label_ru(choice: &str) -> &'static str {
    match choice.trim().to_ascii_lowercase().as_str() {
        "txt" => "Формат txt",
        "epub" => "Формат epub",
        "pdf" => "Формат pdf",
        "bulk_txt" => "Отдельные файлы",
        "ask_after_download" => "Спросить после загрузки",
        _ => "Формат txt",
    }
}

/// Map English book-name field to display label
fn book_name_field_to_chinese(field: &str) -> &'static str {
    match field {
        "book_name" => "Название по умолчанию",
        "original_book_name" => "Оригинальное название",
        "book_short_name" => "Короткое название",
        "ask_after_download" => "Спросить после загрузки",
        _ => "Название по умолчанию",
    }
}

/// Map display label to English book-name field
fn chinese_to_book_name_field(chinese: &str) -> Option<String> {
    match chinese {
        "Название по умолчанию" => Some("book_name".to_string()),
        "Оригинальное название" => Some("original_book_name".to_string()),
        "Короткое название" => Some("book_short_name".to_string()),
        "Спросить после загрузки" => Some("ask_after_download".to_string()),
        _ => None,
    }
}

/// Map display label to novel format value
fn chinese_to_novel_format(chinese: &str) -> Option<String> {
    match chinese {
        "Формат txt" => Some("txt".to_string()),
        "Формат epub" => Some("epub".to_string()),
        "Формат pdf" => Some("pdf".to_string()),
        "Отдельные файлы" => Some(OUTPUT_FORMAT_BULK_TXT.to_string()),
        "Спросить после загрузки" => Some("ask_after_download".to_string()),
        _ => output_format_value_from_label(chinese).map(ToString::to_string),
    }
}
