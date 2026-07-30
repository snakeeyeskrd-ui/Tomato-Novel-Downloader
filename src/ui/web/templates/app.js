/* ===== Tomato Novel Downloader – WebUI ===== */

let loginPromise = null;
let isDockerBuild = false;
let lastIidWarningMessage = null;

function fetchWithCreds(url, opts) {
  return fetch(url, { credentials: 'same-origin', ...(opts || {}) });
}

// ── Theme ──────────────────────────────────────────────────────────

const THEME_KEY = 'tnd.theme';

function getStoredTheme() {
  try { return localStorage.getItem(THEME_KEY); } catch { return null; }
}

function applyTheme(theme) {
  if (theme === 'light' || theme === 'dark') {
    document.documentElement.setAttribute('data-theme', theme);
  } else {
    document.documentElement.removeAttribute('data-theme');
  }
  updateThemeButton(theme);
}

function updateThemeButton(theme) {
  const icon = document.getElementById('themeIcon');
  const label = document.getElementById('themeLabel');
  if (!icon) return;

  const isDark = theme === 'dark' ||
    (!theme && window.matchMedia('(prefers-color-scheme: dark)').matches);

  if (isDark) {
    icon.innerHTML = '<circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>';
    if (label) label.textContent = 'Светлая тема';
  } else {
    icon.innerHTML = '<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>';
    if (label) label.textContent = 'Тёмная тема';
  }
}

function toggleTheme() {
  const current = document.documentElement.getAttribute('data-theme');
  let next;
  if (current === 'dark') {
    next = 'light';
  } else if (current === 'light') {
    next = 'dark';
  } else {
    // auto → opposite of system
    next = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'light' : 'dark';
  }
  try { localStorage.setItem(THEME_KEY, next); } catch {}
  applyTheme(next);
}

// Apply stored theme immediately
(function() {
  const stored = getStoredTheme();
  if (stored) applyTheme(stored);
})();

// ── Auth ───────────────────────────────────────────────────────────

function showLogin(show) {
  const modal = document.getElementById('loginModal');
  if (!modal) return;
  modal.classList.toggle('hidden', !show);
  document.body.style.overflow = show ? 'hidden' : '';
  if (show) {
    const inp = document.getElementById('loginPassword');
    if (inp) inp.focus();
  }
}

function showIidWarningModal(show, message = '') {
  const modal = document.getElementById('iidWarningModal');
  if (!modal) return;
  const body = document.getElementById('iidWarningBody');
  if (body && message) body.textContent = message;
  modal.classList.toggle('hidden', !show);
  document.body.style.overflow = show ? 'hidden' : '';
}

function maybeShowIidWarning(message) {
  const text = (message || '').toString().trim();
  if (!text || lastIidWarningMessage === text) return;
  lastIidWarningMessage = text;
  showIidWarningModal(true, text);
}

function maybeShowIidWarningFromError(message) {
  const text = (message || '').toString();
  const lower = text.toLowerCase();
  if (lower.includes('iid') || lower.includes('log.snssdk.com') || lower.includes('device_register')) {
    maybeShowIidWarning(text);
  }
}

async function requireLogin() {
  if (loginPromise) return loginPromise;

  showLogin(true);
  const msg = document.getElementById('loginMsg');
  if (msg) msg.textContent = '';

  loginPromise = new Promise((resolve, reject) => {
    const form = document.getElementById('loginForm');
    if (!form) { reject(new Error('login form missing')); return; }

    const handler = async (e) => {
      e.preventDefault();
      const pw = (document.getElementById('loginPassword')?.value || '').toString();
      try {
        const res = await fetchWithCreds('/api/login', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ password: pw })
        });
        if (!res.ok) { if (msg) msg.textContent = 'Неверный пароль'; return; }
        showLogin(false);
        form.removeEventListener('submit', handler);
        resolve(true);
      } catch (err) {
        if (msg) msg.textContent = String(err || 'login failed');
      }
    };
    form.addEventListener('submit', handler);
  }).finally(() => { loginPromise = null; });

  return loginPromise;
}

// ── HTTP Helper ────────────────────────────────────────────────────

async function j(url, opts) {
  const res = await fetchWithCreds(url, opts);
  if (res.status === 401) {
    await requireLogin();
    const res2 = await fetchWithCreds(url, opts);
    if (!res2.ok) {
      const text = await res2.text().catch(() => '');
      const message = `${res2.status} ${res2.statusText}${text ? `: ${text}` : ''}`;
      maybeShowIidWarningFromError(message);
      throw new Error(message);
    }
    const ct2 = res2.headers.get('content-type') || '';
    return ct2.includes('application/json') ? res2.json() : res2.text();
  }
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    const message = `${res.status} ${res.statusText}${text ? `: ${text}` : ''}`;
    maybeShowIidWarningFromError(message);
    throw new Error(message);
  }
  const ct = res.headers.get('content-type') || '';
  return ct.includes('application/json') ? res.json() : res.text();
}

// ── Utilities ──────────────────────────────────────────────────────

function esc(s) {
  return (s ?? '').toString().replace(/[&<>"']/g, c =>
    ({ '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;' }[c]));
}

function fmtBytes(n) {
  const x = Number(n || 0);
  if (!isFinite(x) || x <= 0) return '0 B';
  const k = 1024;
  const sizes = ['B','KB','MB','GB','TB'];
  const i = Math.floor(Math.log(x) / Math.log(k));
  return (x / Math.pow(k, i)).toFixed(i === 0 ? 0 : 1) + ' ' + sizes[i];
}

function fmtTime(ms) {
  const x = Number(ms || 0);
  if (!isFinite(x) || x <= 0) return '';
  return new Date(x).toLocaleString();
}

function encodePathSegments(path) {
  return (path || '').toString().split('/').map(seg => encodeURIComponent(seg)).join('/');
}

function parseBookId(input) {
  const trimmed = (input ?? '').toString().trim();
  if (!trimmed) return '';
  if (/^[0-9]+$/.test(trimmed)) return trimmed;

  const urlMatch = trimmed.match(/https?:\/\/\S+/i);
  const target = urlMatch ? urlMatch[0] : trimmed;

  const qs = target.match(/(?:^|[?&#])(?:book_id|bookId)=([0-9]+)/i);
  if (qs && qs[1]) return qs[1];

  const page = target.match(/\/page\/([0-9]+)/i);
  if (page && page[1]) return page[1];

  // Short link (e.g. https://changdunovel.com/t/E_HDbOHpMJA/) – return the URL so
  // the server can follow the redirect and extract the book ID.
  // Restrict to known share-link hosts to prevent forwarding arbitrary URLs.
  const allowedShortLinkHosts = new Set(['changdunovel.com', 'www.changdunovel.com', 'fanqienovel.com', 'www.fanqienovel.com', 'fqnovel.com', 'www.fqnovel.com']);
  try {
    const parsed = new URL(target);
    if (
      (parsed.protocol === 'http:' || parsed.protocol === 'https:') &&
      allowedShortLinkHosts.has(parsed.hostname.toLowerCase()) &&
      /^\/t\/[A-Za-z0-9_-]+\/?$/.test(parsed.pathname)
    ) {
      return target;
    }
  } catch (_) {
    // Not a valid absolute URL; ignore and fall through.
  }

  return '';
}

function isLikelyHeicUrl(url) {
  const s = (url || '').toString().toLowerCase();
  if (!s) return false;
  return /[\/.](heic|heif)(?:$|[?#])/i.test(s) || s.includes('format=heic') || s.includes('mime=image/heic');
}

function buildCoverCandidates(preview) {
  const list = [];
  const add = (u) => {
    const v = (u || '').toString().trim();
    if (!v) return;
    if (!(v.startsWith('http://') || v.startsWith('https://') || v.startsWith('/'))) return;
    if (!list.includes(v)) list.push(v);
  };

  add(preview?.detail_cover_url);
  add(preview?.cover_url);

  const nonHeic = list.filter(u => !isLikelyHeicUrl(u));
  const heic = list.filter(isLikelyHeicUrl);
  return [...nonHeic, ...heic];
}

// ── App Update ─────────────────────────────────────────────────────

const DISMISS_KEY = 'tnd.dismissed_release_tag';
let selfUpdatePollTimer = null;
let selfUpdateWasRunning = false;
let selfUpdateRestartWaiting = false;

function getDismissedTag() {
  try { return (localStorage.getItem(DISMISS_KEY) || '').toString(); } catch { return ''; }
}
function setDismissedTag(tag) {
  try { localStorage.setItem(DISMISS_KEY, (tag || '').toString()); } catch {}
}

function showAppUpdateBanner(show) {
  const el = document.getElementById('appUpdateBanner');
  if (el) el.classList.toggle('hidden', !show);
}

function renderSelfUpdateStatus(status) {
  const wrap = document.getElementById('selfUpdateProgressWrap');
  const stage = document.getElementById('selfUpdateStage');
  const msg = document.getElementById('selfUpdateMessage');
  const bar = document.getElementById('selfUpdateProgressBar');
  const pct = document.getElementById('selfUpdatePercent');
  if (!wrap || !stage || !msg || !bar || !pct) return;

  const st = (status?.state || 'idle').toString();
  const percent = Number(status?.percent || 0);
  const show = st !== 'idle';
  wrap.classList.toggle('hidden', !show);
  if (!show) return;

  stage.textContent = (status?.stage || 'idle').toString();
  msg.textContent = (status?.message || '').toString();
  bar.style.width = `${Math.max(0, Math.min(100, percent))}%`;
  pct.textContent = `${Math.max(0, Math.min(100, percent))}%`;
}

async function pollSelfUpdateStatus() {
  try {
    const status = await j('/api/self_update');
    renderSelfUpdateStatus(status);

    const st = (status?.state || '').toString();
    const stage = (status?.stage || '').toString();
    if (st === 'running') {
      selfUpdateWasRunning = true;
      selfUpdateRestartWaiting = false;
      if (!selfUpdatePollTimer) {
        selfUpdatePollTimer = setInterval(() => {
          pollSelfUpdateStatus().catch(() => {});
        }, 1000);
      }
    } else {
      if (selfUpdatePollTimer) {
        clearInterval(selfUpdatePollTimer);
        selfUpdatePollTimer = null;
      }
      // Detect restart scenarios
      if (selfUpdateWasRunning && !selfUpdateRestartWaiting) {
        if (st === 'done' && stage === 'restart') {
          // finish_done("restart") was called before exit — wait for new process
          startWaitingForRestart();
        } else if (st === 'idle') {
          // New process started with fresh state — page needs reload
          window.location.reload();
        }
      }
    }
  } catch {
    // Network error — if update was in progress, server likely restarted
    if (selfUpdateWasRunning && !selfUpdateRestartWaiting) {
      startWaitingForRestart();
    }
  }
}

function startWaitingForRestart() {
  selfUpdateRestartWaiting = true;
  if (selfUpdatePollTimer) {
    clearInterval(selfUpdatePollTimer);
    selfUpdatePollTimer = null;
  }

  // Push progress bar to 100%
  const wrap = document.getElementById('selfUpdateProgressWrap');
  const bar = document.getElementById('selfUpdateProgressBar');
  const pct = document.getElementById('selfUpdatePercent');
  const stageEl = document.getElementById('selfUpdateStage');
  const msgEl = document.getElementById('selfUpdateMessage');
  if (wrap) wrap.classList.remove('hidden');
  if (bar) bar.style.width = '100%';
  if (pct) pct.textContent = '100%';
  if (stageEl) stageEl.textContent = 'restart';
  if (msgEl) msgEl.textContent = 'Сервис перезапускается, ожидание соединения…';

  const hint = document.getElementById('appUpdateHint');
  if (hint) hint.textContent = 'Обновление завершено, ожидание перезапуска…';

  // Poll /api/status every 2 s; reload once the new process responds
  const reconnTimer = setInterval(async () => {
    try {
      await fetchWithCreds('/api/status');
      clearInterval(reconnTimer);
      if (msgEl) msgEl.textContent = 'Сервис перезапущен, обновление страницы…';
      if (hint) hint.textContent = 'Обновление завершено, обновление страницы…';
      setTimeout(() => window.location.reload(), 600);
    } catch {
      // still offline, keep waiting
    }
  }, 2000);
}

function applyDockerUpdateUi() {
  if (!isDockerBuild) return;
  const hint = document.getElementById('appUpdateHint');
  if (hint) hint.textContent = 'В Docker-сборке самообновление отключено. Обновите, заново загрузив образ.';
  showAppUpdateBanner(false);
  const btn = document.getElementById('appUpdateCheck');
  if (btn) btn.disabled = true;
  const selfBtn = document.getElementById('appSelfUpdate');
  if (selfBtn) selfBtn.disabled = true;
  const dismissBtn = document.getElementById('appUpdateDismiss');
  if (dismissBtn) dismissBtn.disabled = true;
}

async function refreshAppUpdate(manual) {
  const hint = document.getElementById('appUpdateHint');
  const latestEl = document.getElementById('appUpdateLatest');
  const bodyEl = document.getElementById('appUpdateBody');
  const linkEl = document.getElementById('appUpdateLink');

  if (isDockerBuild) {
    applyDockerUpdateUi();
    if (latestEl) latestEl.textContent = '';
    if (bodyEl) bodyEl.textContent = 'В Docker-сборке самообновление отключено. Обновите, заново загрузив образ.';
    if (linkEl) linkEl.style.pointerEvents = 'none';
    return { latestTag: '', hasUpdate: false, dockerBuild: true };
  }

  if (hint) hint.textContent = manual ? 'Проверка…' : '';

  const data = await j('/api/app_update');
  const latestTag = (data.latest_tag || '').toString();
  const latestBody = (data.latest_body || '').toString();
  const latestUrl = (data.latest_url || '').toString();
  const hasUpdate = !!data.has_update;

  if (latestEl) latestEl.textContent = latestTag || '';
  if (bodyEl) bodyEl.textContent = latestBody || '';
  if (linkEl) {
    linkEl.href = latestUrl || '#';
    linkEl.style.pointerEvents = latestUrl ? '' : 'none';
  }

  const dismissed = getDismissedTag();
  const shouldShow = hasUpdate && latestTag && dismissed !== latestTag;

  if (shouldShow) {
    showAppUpdateBanner(true);
    if (hint) hint.textContent = 'Доступна новая версия';
  } else {
    showAppUpdateBanner(false);
    if (manual) {
      if (!hasUpdate) {
        if (hint) hint.textContent = 'Уже установлена последняя версия';
      } else if (dismissed === latestTag) {
        if (hint) hint.textContent = 'Напоминание об этой версии отключено';
      }
    }
  }
  return { latestTag, hasUpdate };
}

// ── Status ─────────────────────────────────────────────────────────

let libraryPath = '';
let libraryPollTimer = null;
let pendingBookNameJobId = null;
let pendingBookNameOptions = [];
let pendingFormatJobId = null;
let pendingFormatOptions = [];

async function refreshStatus() {
  const data = await j('/api/status');
  document.getElementById('version').textContent = data.version || '';
  const prewarmError = (data.prewarm_error || '').toString();
  document.getElementById('prewarm').textContent = prewarmError
    ? 'failed'
    : (data.prewarm_in_progress ? 'warming' : 'ready');
  document.getElementById('saveDir').textContent = data.save_dir || '';
  document.getElementById('bind').textContent = data.bind_addr || '';
  document.getElementById('locked').textContent = data.locked ? 'locked' : 'unlocked';
  const iidBanner = document.getElementById('iidWarningBanner');
  const iidBannerBody = document.getElementById('iidWarningBannerBody');
  if (iidBanner) iidBanner.classList.toggle('hidden', !prewarmError);
  if (iidBannerBody && prewarmError) iidBannerBody.textContent = prewarmError;
  if (prewarmError) maybeShowIidWarning(prewarmError);
  isDockerBuild = !!data.docker_build;
  applyDockerUpdateUi();
}

// ── Config ─────────────────────────────────────────────────────────

async function refreshConfig() {
  const data = await j('/api/config');
  const nf = document.getElementById('cfgNovelFormat');
  const ea = document.getElementById('cfgEnableAudiobook');
  const af = document.getElementById('cfgAudiobookFormat');
  if (nf) nf.value = (data.novel_format || 'txt').toString();
  if (ea) ea.checked = !!data.enable_audiobook;
  if (af) af.value = (data.audiobook_format || 'mp3').toString();
}

async function saveConfig() {
  const nf = document.getElementById('cfgNovelFormat')?.value;
  const ea = !!document.getElementById('cfgEnableAudiobook')?.checked;
  const af = document.getElementById('cfgAudiobookFormat')?.value;

  await j('/api/config', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      novel_format: nf,
      enable_audiobook: ea,
      audiobook_format: af
    })
  });
}

async function refreshRawConfig() {
  const data = await j('/api/config/raw');
  const ta = document.getElementById('cfgRaw');
  const msg = document.getElementById('cfgRawMsg');
  if (ta) ta.value = (data.yaml || '').toString();
  if (msg) msg.textContent = data.generated ? 'Создана конфигурация по умолчанию (файл не найден)' : '';
}

async function saveRawConfig() {
  const ta = document.getElementById('cfgRaw');
  const yaml = (ta?.value || '').toString();
  await j('/api/config/raw', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ yaml })
  });
}

// ── Full Config ────────────────────────────────────────────────────

let currentFullConfig = null;

const FULL_CONFIG_SCHEMA = [
  {
    title: 'Основные и формат',
    fields: [
      { key: 'save_path', label: 'Путь сохранения', type: 'text' },
      { key: 'novel_format', label: 'Формат книги', type: 'select', options: [
        { value: 'txt', label: 'txt' },
        { value: 'epub', label: 'epub' },
        { value: 'pdf', label: 'pdf' },
        { value: 'bulk_txt', label: 'Отдельные файлы' },
        { value: 'ask_after_download', label: 'Выбрать после загрузки' }
      ] },
      { key: 'first_line_indent_em', label: 'Отступ первой строки (em)', type: 'number', parse: 'float', step: '0.1', min: '0' },
      { key: 'auto_clear_dump', label: 'Автоочистка кэша', type: 'bool' },
      { key: 'auto_open_downloaded_files', label: 'Открывать после загрузки', type: 'bool' },
      { key: 'allow_overwrite_files', label: 'Разрешить перезапись файлов', type: 'bool' },
      { key: 'preferred_book_name_field', label: 'Предпочтительное поле названия', type: 'select', options: [
        { value: 'book_name', label: 'Название по умолчанию' },
        { value: 'original_book_name', label: 'Оригинальное название' },
        { value: 'book_short_name', label: 'Короткое название' },
        { value: 'ask_after_download', label: 'Выбрать после загрузки' }
      ] },
      { key: 'old_cli', label: 'Старый CLI UI', type: 'bool' },
    ]
  },
  {
    title: 'Сеть и планирование',
    fields: [
      { key: 'max_workers', label: 'Макс. потоков', type: 'number', parse: 'int', min: '1' },
      { key: 'request_timeout', label: 'Таймаут запроса (с)', type: 'number', parse: 'int', min: '1' },
      { key: 'max_retries', label: 'Макс. повторов', type: 'number', parse: 'int', min: '0' },
      { key: 'min_connect_timeout', label: 'Мин. таймаут соединения (с)', type: 'number', parse: 'float', step: '0.1', min: '0' },
      { key: 'min_wait_time', label: 'Мин. ожидание (мс)', type: 'number', parse: 'int', min: '0' },
      { key: 'max_wait_time', label: 'Макс. ожидание (мс)', type: 'number', parse: 'int', min: '0' },
    ]
  },
  {
    title: 'API',
    fields: [
      { key: 'use_official_api', label: 'Использовать официальный API', type: 'bool' },
      { key: 'api_endpoints', label: 'Список API', type: 'list', placeholder: 'По одной в строке или через запятую' },
    ]
  },
  {
    title: 'Комментарии к абзацам',
    fields: [
      { key: 'enable_segment_comments', label: 'Включить комментарии к абзацам', type: 'bool' },
      { key: 'segment_comments_top_n', label: 'Лимит комментариев на абзац', type: 'number', parse: 'int', min: '1' },
      { key: 'segment_comments_workers', label: 'Потоки для комментариев', type: 'number', parse: 'int', min: '1' },
    ]
  },
  {
    title: 'Загрузка медиа',
    fields: [
      { key: 'download_comment_images', label: 'Скачивать изображения комментариев', type: 'bool' },
      { key: 'download_comment_avatars', label: 'Скачивать аватары комментариев', type: 'bool' },
      { key: 'media_download_workers', label: 'Потоки загрузки медиа', type: 'number', parse: 'int', min: '1' },
      { key: 'blocked_media_domains', label: 'Блокируемые домены изображений', type: 'list', placeholder: 'По одному домену в строке' },
      { key: 'force_convert_images_to_jpeg', label: 'Принудительно в JPEG', type: 'bool' },
      { key: 'jpeg_retry_convert', label: 'Повтор при ошибке конвертации в JPEG', type: 'bool' },
      { key: 'jpeg_quality', label: 'Качество JPEG (0–100)', type: 'number', parse: 'int', min: '0', max: '100' },
      { key: 'convert_heic_to_jpeg', label: 'HEIC в JPEG', type: 'bool' },
      { key: 'keep_heic_original', label: 'Сохранять оригинал HEIC', type: 'bool' },
      { key: 'media_limit_per_chapter', label: 'Лимит медиа на главу', type: 'number', parse: 'int', min: '0' },
      { key: 'media_max_dimension_px', label: 'Макс. размер медиа (px)', type: 'number', parse: 'int', min: '0' },
    ]
  },
  {
    title: 'Аудиокнига',
    fields: [
      { key: 'enable_audiobook', label: 'Включить аудиокнигу', type: 'bool' },
      { key: 'audiobook_voice', label: 'Голос', type: 'voice' },
      { key: 'audiobook_tts_provider', label: 'Тип TTS', type: 'select', options: [
        { value: 'edge', label: 'edge' }, { value: 'third_party', label: 'third_party' }
      ] },
      { key: 'audiobook_tts_api_url', label: 'URL стороннего TTS API', type: 'text' },
      { key: 'audiobook_tts_api_token', label: 'Токен стороннего TTS', type: 'text' },
      { key: 'audiobook_tts_model', label: 'Модель стороннего TTS', type: 'text' },
      { key: 'audiobook_rate', label: 'Скорость речи', type: 'text' },
      { key: 'audiobook_volume', label: 'Громкость', type: 'text' },
      { key: 'audiobook_pitch', label: 'Тон', type: 'text' },
      { key: 'audiobook_format', label: 'Формат вывода', type: 'select', options: [
        { value: 'mp3', label: 'mp3' }, { value: 'wav', label: 'wav' }
      ] },
      { key: 'audiobook_concurrency', label: 'Параллельная генерация глав', type: 'number', parse: 'int', min: '1' },
    ]
  },
];

const AUDIOBOOK_VOICE_PRESETS = [
  { value: 'zh-CN-XiaoxiaoNeural', label: 'zh-CN-XiaoxiaoNeural (жен.)' },
  { value: 'zh-CN-XiaoyiNeural', label: 'zh-CN-XiaoyiNeural (жен.)' },
  { value: 'zh-CN-YunjianNeural', label: 'zh-CN-YunjianNeural (муж.)' },
  { value: 'zh-CN-YunxiNeural', label: 'zh-CN-YunxiNeural (муж.)' },
  { value: 'zh-CN-YunxiaNeural', label: 'zh-CN-YunxiaNeural (муж.)' },
  { value: 'zh-CN-YunyangNeural', label: 'zh-CN-YunyangNeural (муж.)' },
  { value: 'zh-CN-liaoning-XiaobeiNeural', label: 'zh-CN-liaoning-XiaobeiNeural (жен.)' },
  { value: 'zh-CN-shaanxi-XiaoniNeural', label: 'zh-CN-shaanxi-XiaoniNeural (жен.)' },
  { value: 'zh-HK-HiuGaaiNeural', label: 'zh-HK-HiuGaaiNeural (жен.)' },
  { value: 'zh-HK-HiuMaanNeural', label: 'zh-HK-HiuMaanNeural (жен.)' },
  { value: 'zh-HK-WanLungNeural', label: 'zh-HK-WanLungNeural (муж.)' },
  { value: 'zh-TW-HsiaoChenNeural', label: 'zh-TW-HsiaoChenNeural (жен.)' },
];

function renderFullConfigForm(cfg) {
  const body = document.getElementById('configFullBody');
  if (!body) return;
  body.innerHTML = '';

  for (const section of FULL_CONFIG_SCHEMA) {
    const sec = document.createElement('div');
    sec.className = 'configSection';
    sec.innerHTML = `<h4>${esc(section.title)}</h4>`;
    body.appendChild(sec);

    for (const field of section.fields) {
      const row = document.createElement('div');
      row.className = 'config-field';

      const label = document.createElement('span');
      label.className = 'field-label';
      label.textContent = field.label;
      row.appendChild(label);

      let input;
      if (field.type === 'bool') {
        input = document.createElement('input');
        input.type = 'checkbox';
        input.checked = !!cfg[field.key];
      } else if (field.type === 'voice') {
        input = document.createElement('div');
        input.className = 'voiceRow';
        const select = document.createElement('select');
        const emptyOpt = document.createElement('option');
        emptyOpt.value = '';
        emptyOpt.textContent = 'Свой…';
        select.appendChild(emptyOpt);
        for (const opt of AUDIOBOOK_VOICE_PRESETS) {
          const o = document.createElement('option');
          o.value = opt.value;
          o.textContent = opt.label;
          select.appendChild(o);
        }
        const text = document.createElement('input');
        text.type = 'text';
        text.value = (cfg[field.key] ?? '').toString();
        text.placeholder = 'Введите или выберите голос';
        text.dataset.key = field.key;
        text.dataset.type = 'text';
        text.dataset.voiceInput = '1';

        const current = (cfg[field.key] ?? '').toString();
        const preset = AUDIOBOOK_VOICE_PRESETS.find(p => p.value === current);
        select.value = preset ? preset.value : '';

        select.addEventListener('change', () => { if (select.value) text.value = select.value; });
        input.appendChild(select);
        input.appendChild(text);
      } else if (field.type === 'select') {
        input = document.createElement('select');
        for (const opt of field.options || []) {
          const o = document.createElement('option');
          o.value = opt.value;
          o.textContent = opt.label;
          input.appendChild(o);
        }
        if (field.key === 'novel_format' && cfg.ask_format_after_download) {
          input.value = 'ask_after_download';
        } else if (field.key === 'novel_format' && cfg.bulk_files) {
          input.value = 'bulk_txt';
        } else {
          input.value = (cfg[field.key] ?? '').toString();
        }
      } else if (field.type === 'list') {
        input = document.createElement('textarea');
        input.value = Array.isArray(cfg[field.key]) ? cfg[field.key].join('\n') : '';
        input.placeholder = field.placeholder || '';
        input.classList.add('cfgList');
      } else if (field.type === 'number') {
        input = document.createElement('input');
        input.type = 'number';
        if (field.step) input.step = field.step;
        if (field.min) input.min = field.min;
        if (field.max) input.max = field.max;
        input.value = (cfg[field.key] ?? '').toString();
      } else {
        input = document.createElement('input');
        input.type = 'text';
        input.value = (cfg[field.key] ?? '').toString();
        if (field.placeholder) input.placeholder = field.placeholder;
      }

      if (field.type !== 'voice') {
        input.dataset.key = field.key;
        input.dataset.type = field.type;
        if (field.parse) input.dataset.parse = field.parse;
      }

      row.appendChild(input);
      sec.appendChild(row);
    }
  }
}

async function loadFullConfigPanel() {
  const msg = document.getElementById('cfgFullMsg');
  if (msg) msg.textContent = 'Загрузка…';
  try {
    const cfg = await j('/api/config/full');
    currentFullConfig = cfg || {};
    renderFullConfigForm(currentFullConfig);
    if (msg) msg.textContent = '';
  } catch (err) {
    if (msg) msg.textContent = 'Ошибка загрузки';
  }
}

function collectFullConfig() {
  const out = { ...(currentFullConfig || {}) };
  const body = document.getElementById('configFullBody');
  if (!body) return out;
  const inputs = body.querySelectorAll('[data-key]');
  for (const el of inputs) {
    const key = el.dataset.key;
    const type = el.dataset.type;
    if (!key || !type) continue;
    if (type === 'bool') {
      out[key] = !!el.checked;
    } else if (type === 'list') {
      out[key] = (el.value || '').toString().split(/[\n,;]/).map(s => s.trim()).filter(s => s.length > 0);
    } else if (type === 'number') {
      const raw = (el.value || '').toString().trim();
      if (!raw) continue;
      const parse = el.dataset.parse || 'int';
      const val = parse === 'float' ? parseFloat(raw) : parseInt(raw, 10);
      if (!Number.isNaN(val)) out[key] = val;
    } else {
      out[key] = (el.value || '').toString();
    }
  }
  if (out.novel_format === 'ask_after_download') {
    out.ask_format_after_download = true;
    out.bulk_files = false;
    out.novel_format = (currentFullConfig?.novel_format || 'txt').toString();
  } else if (out.novel_format === 'bulk_txt') {
    out.ask_format_after_download = false;
    out.bulk_files = true;
    out.novel_format = 'txt';
  } else if (out.novel_format) {
    out.ask_format_after_download = false;
    out.bulk_files = false;
  }
  return out;
}

async function saveFullConfig() {
  const cfg = collectFullConfig();
  await j('/api/config/full', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(cfg)
  });
}

// ── Library ────────────────────────────────────────────────────────

function scheduleLibraryPoll() {
  if (libraryPollTimer) clearTimeout(libraryPollTimer);
  libraryPollTimer = setTimeout(() => {
    libraryPollTimer = null;
    refreshLibrary(false).catch(() => {});
  }, 700);
}

async function refreshLibrary(start = true) {
  if (start && libraryPollTimer) {
    clearTimeout(libraryPollTimer);
    libraryPollTimer = null;
  }

  const params = new URLSearchParams();
  if (libraryPath) params.set('path', libraryPath);
  params.set('start', start ? 'true' : 'false');
  const qs = params.toString() ? `?${params.toString()}` : '';
  const data = await j(`/api/library${qs}`);
  const items = data.items || [];
  libraryPath = (data.path || '').toString();
  const running = !!data.running;
  const scanned = Number(data.scanned || items.length || 0);

  const pathLabel = document.getElementById('libPath');
  const backBtn = document.getElementById('libBack');
  const hint = document.getElementById('libHint');
  if (pathLabel) pathLabel.textContent = libraryPath ? `/${libraryPath}` : '/';
  if (backBtn) backBtn.disabled = !libraryPath;
  if (hint) {
    if (data.error) {
      hint.textContent = `Ошибка чтения: ${data.error}`;
    } else if (running) {
      hint.textContent = `Пакетное чтение: найдено ${items.length} / проверено ${scanned}`;
    } else {
      hint.textContent = `Всего: ${items.length}`;
    }
  }

  const tbody = document.getElementById('libraryBody');
  tbody.innerHTML = '';
  for (const it of items) {
    const tr = document.createElement('tr');
    const kind = it.kind || 'file';
    const rel = it.rel_path || '';
    const name = it.name || rel;
    const encodedRel = encodePathSegments(rel);
    const hrefFile = `/download/${encodedRel}`;
    const hrefZip = `/download-zip/${encodedRel}`;
    const sizeText = kind === 'dir'
      ? (it.file_count == null ? 'Папка' : `${fmtBytes(it.size)} (${Number(it.file_count || 0)} файлов)`)
      : fmtBytes(it.size);
    const timeText = fmtTime(it.modified_ms);

    if (kind === 'dir') {
      tr.innerHTML = `
        <td><button class="openDir sm" data-path="${esc(rel)}">Открыть</button> ${esc(name)} <span class="badge">Папка</span></td>
        <td>${esc(sizeText)}</td>
        <td>${esc(timeText)}</td>
        <td><a href="${hrefZip}">Скачать архивом</a></td>
      `;
    } else {
      tr.innerHTML = `
        <td><a href="${hrefFile}">${esc(name)}</a> <span class="badge">${esc(it.ext || '')}</span></td>
        <td>${esc(sizeText)}</td>
        <td>${esc(timeText)}</td>
        <td><a href="${hrefFile}">Скачать</a></td>
      `;
    }
    tbody.appendChild(tr);
  }

  if (items.length === 0) {
    if (running) {
      tbody.innerHTML = '<tr class="empty-row"><td colspan="4">Пакетное чтение библиотеки, результаты появятся постепенно…</td></tr>';
    } else {
      tbody.innerHTML = '<tr class="empty-row"><td colspan="4">Пока нет файлов — сначала загрузите книгу</td></tr>';
    }
  }

  if (running) {
    scheduleLibraryPoll();
  }
}

// ── Search ─────────────────────────────────────────────────────────

async function doSearch(q) {
  const out = document.getElementById('searchResults');
  out.innerHTML = '';
  if (!q) return;
  const data = await j(`/api/search?q=${encodeURIComponent(q)}`);
  const items = data.items || [];
  if (items.length === 0) {
    out.innerHTML = '<tr class="empty-row"><td colspan="4">Нет результатов</td></tr>';
    return;
  }
  for (const b of items) {
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td>${esc(b.title ?? '')}</td>
      <td>${esc(b.author ?? '')}</td>
      <td><code>${esc(b.book_id)}</code></td>
      <td><button data-bookid="${esc(b.book_id)}" class="startDownload sm primary">Скачать</button></td>
    `;
    out.appendChild(tr);
  }
}

// ── Preview ────────────────────────────────────────────────────────

let currentPreviewBookId = null;
let currentPreviewData = null;

function showPreviewModal(show) {
  const modal = document.getElementById('previewModal');
  if (!modal) return;
  modal.classList.toggle('hidden', !show);
  document.body.style.overflow = show ? 'hidden' : '';
  if (!show) {
    // When closing preview, clean up cover cache folders created on the server
    if (currentPreviewBookId) {
      fetchWithCreds(`/api/preview/${encodeURIComponent(currentPreviewBookId)}/cleanup`, {
        method: 'POST',
      }).catch(() => {}); // fire-and-forget
    }
    currentPreviewBookId = null;
    currentPreviewData = null;
  }
}

async function openPreview(bookId) {
  currentPreviewBookId = bookId;
  currentPreviewData = null;
  showPreviewModal(true);

  const loading = document.getElementById('previewLoading');
  const data = document.getElementById('previewData');
  const rangeInput = document.getElementById('previewRangeInput');
  const rangeHint = document.getElementById('previewRangeHint');

  if (loading) loading.classList.remove('hidden');
  if (data) data.classList.add('hidden');
  if (rangeInput) rangeInput.value = '';
  if (rangeHint) { rangeHint.textContent = ''; rangeHint.classList.remove('error'); }

  try {
    const preview = await j(`/api/preview/${encodeURIComponent(bookId)}`);
    currentPreviewData = preview;

    if (loading) loading.classList.add('hidden');
    if (data) data.classList.remove('hidden');

    const title = document.getElementById('previewTitle');
    const origTitle = document.getElementById('previewOrigTitle');
    const author = document.getElementById('previewAuthor');
    const stats = document.getElementById('previewStats');
    const desc = document.getElementById('previewDesc');
    const tags = document.getElementById('previewTags');
    const chapters = document.getElementById('previewChapters');
    const cover = document.getElementById('previewCover');

    if (title) title.textContent = preview.book_name || 'Без названия';

    if (origTitle) {
      if (preview.original_book_name && preview.original_book_name !== preview.book_name) {
        origTitle.textContent = `Оригинал: ${preview.original_book_name}`;
        origTitle.classList.remove('hidden');
      } else {
        origTitle.classList.add('hidden');
      }
    }

    if (author) author.textContent = preview.author ? `Автор: ${preview.author}` : 'Автор: неизвестен';

    if (stats) {
      const parts = [];
      if (preview.chapter_count) parts.push(`Глав: ${preview.chapter_count}`);
      if (preview.finished !== null && preview.finished !== undefined) {
        parts.push(`Статус: ${preview.finished ? 'завершена' : 'продолжается'}`);
      }
      if (preview.word_count) {
        const words = Number(preview.word_count);
        parts.push(`Слов: ${words >= 10000 ? (words / 10000).toFixed(1) + ' тыс.' : words}`);
      }
      if (preview.score != null) parts.push(`Оценка: ${preview.score.toFixed(1)}`);
      if (preview.read_count_text || preview.read_count) {
        parts.push(`Прочтений: ${preview.read_count_text || preview.read_count}`);
      }
      stats.innerHTML = '';
      parts.forEach(p => {
        const span = document.createElement('span');
        span.textContent = p;
        stats.appendChild(span);
      });
    }

    if (desc) desc.textContent = preview.description || 'Нет описания';

    if (tags) {
      if (preview.tags && preview.tags.length > 0) {
        tags.innerHTML = '';
        preview.tags.forEach(t => {
          const badge = document.createElement('span');
          badge.className = 'badge';
          badge.textContent = t;
          tags.appendChild(badge);
        });
        tags.classList.remove('hidden');
      } else {
        tags.classList.add('hidden');
      }
    }

    if (chapters) {
      const chapterInfo = [];
      if (preview.chapter_count) chapterInfo.push(`Всего глав: ${preview.chapter_count}`);
      if (preview.first_chapter_title) chapterInfo.push(`Первая глава: ${preview.first_chapter_title}`);
      if (preview.last_chapter_title) chapterInfo.push(`Последняя глава: ${preview.last_chapter_title}`);
      if (preview.category) chapterInfo.push(`Жанр: ${preview.category}`);
      chapters.innerHTML = '';
      chapterInfo.forEach(info => {
        const div = document.createElement('div');
        div.textContent = info;
        chapters.appendChild(div);
      });
    }

    if (cover) {
      const candidates = buildCoverCandidates(preview);
      let coverIdx = 0;

      const loadCandidate = () => {
        if (coverIdx >= candidates.length) {
          cover.removeAttribute('src');
          cover.classList.add('hidden');
          cover.onerror = null;
          return;
        }
        cover.src = candidates[coverIdx++];
        cover.classList.remove('hidden');
      };

      cover.onerror = () => loadCandidate();
      if (candidates.length > 0) {
        loadCandidate();
      } else {
        cover.removeAttribute('src');
        cover.classList.add('hidden');
        cover.onerror = null;
      }
    }

    if (rangeHint && preview.chapter_count) {
      rangeHint.textContent = `Например: 1-10 — главы 1–10, 1-${preview.chapter_count} — все главы`;
    }
  } catch (err) {
    if (loading) loading.textContent = `Ошибка загрузки: ${err}`;
    console.error('Preview load error:', err);
  }
}

async function confirmPreview() {
  if (!currentPreviewBookId || !currentPreviewData) { showPreviewModal(false); return; }

  const bookId = currentPreviewBookId;
  const rangeInput = document.getElementById('previewRangeInput');
  const rangeHint = document.getElementById('previewRangeHint');
  const rangeText = rangeInput ? rangeInput.value.trim() : '';

  let rangeStart = null;
  let rangeEnd = null;

  if (rangeText) {
    const total = currentPreviewData.chapter_count || 0;
    if (total === 0) {
      if (rangeHint) { rangeHint.textContent = 'Число глав неизвестно, диапазон недоступен'; rangeHint.classList.add('error'); }
      return;
    }
    const parts = rangeText.split('-').map(p => p.trim());
    if (parts.length === 2) {
      const start = parts[0] === '' ? 1 : parseInt(parts[0], 10);
      const end = parts[1] === '' ? total : parseInt(parts[1], 10);
      if (isNaN(start) || isNaN(end) || start < 1 || end < 1 || start > end || end > total) {
        if (rangeHint) { rangeHint.textContent = `Неверный диапазон (1–${total})`; rangeHint.classList.add('error'); }
        return;
      }
      rangeStart = start;
      rangeEnd = end;
    } else {
      if (rangeHint) { rangeHint.textContent = 'Формат: start-end, например 1-10'; rangeHint.classList.add('error'); }
      return;
    }
  }

  if (rangeHint) rangeHint.classList.remove('error');
  showPreviewModal(false);

  try {
    const payload = { book_id: bookId };
    if (rangeStart !== null && rangeEnd !== null) {
      payload.range_start = rangeStart;
      payload.range_end = rangeEnd;
    }
    await j('/api/jobs', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(payload)
    });
    await refreshJobs();
    window.location.hash = '#jobs';
    const hint = document.getElementById('searchHint');
    if (hint) {
      hint.textContent = rangeStart && rangeEnd
        ? `Создана задача загрузки: ${bookId} (главы ${rangeStart}–${rangeEnd})`
        : `Создана задача загрузки: ${bookId}`;
    }
  } catch (err) {
    alert(`Не удалось создать задачу: ${err}`);
  }
}

async function startDownload(bookId) {
  await openPreview(bookId);
  return null;
}

async function startDownloadDirect(bookId) {
  const job = await j('/api/jobs', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ book_id: bookId })
  });
  await refreshJobs();
  return job;
}

// ── Jobs ───────────────────────────────────────────────────────────

async function refreshJobs() {
  const data = await j('/api/jobs');
  const tbody = document.getElementById('jobsBody');
  tbody.innerHTML = '';
  for (const it of data.items || []) {
    const tr = document.createElement('tr');
    const saved = it.progress ? it.progress.saved_chapters : 0;
    const total = it.progress ? it.progress.chapter_total : 0;
    const pct = total > 0 ? Math.min(100, Math.round((saved / total) * 100)) : 0;
    const progressText = it.progress ? `${saved}/${total}` : '';
    const title = it.title || it.book_id || '';
    const hasBookNameOptions = (it.book_name_options || []).length > 0;
    const hasFormatOptions = (it.format_options || []).length > 0;
    const needsPostConfig = hasBookNameOptions || hasFormatOptions;

    // Determine effective visual state
    let vState = (it.state || '').toLowerCase();
    if (vState === 'done' && total > 0 && saved < total) vState = 'partial';
    if (vState === 'failed' && it.message) maybeShowIidWarningFromError(it.message);

    // Row class & CSS custom property for progress gradient
    tr.className = 'job-row state-' + vState;
    if (vState === 'running' || vState === 'queued') {
      tr.style.setProperty('--progress', pct + '%');
    }

    // State badge
    let stateHtml;
    if (needsPostConfig) {
      stateHtml = '<span class="badge warning">Ожидает настройки</span>';
    } else switch (vState) {
      case 'running': stateHtml = `<span class="badge info">${pct}%</span>`; break;
      case 'queued':  stateHtml = '<span class="badge">В очереди</span>'; break;
      case 'done':    stateHtml = '<span class="badge success">Готово</span>'; break;
      case 'failed':  stateHtml = '<span class="badge danger">Ошибка</span>'; break;
      case 'partial': stateHtml = '<span class="badge warning">Частичная ошибка</span>'; break;
      case 'canceled':stateHtml = '<span class="badge">Отменено</span>'; break;
      default:        stateHtml = esc(it.state || '');
    }

    // Action button
    let btnHtml;
    if (needsPostConfig) {
      const kind = hasBookNameOptions ? 'book_name' : 'format';
      btnHtml = `<button data-jobid="${esc(it.id)}" data-kind="${esc(kind)}" class="configJob sm warning">Настроить…</button>`;
    } else switch (vState) {
      case 'done':
        btnHtml = `<button data-jobid="${esc(it.id)}" data-title="${esc(title)}" class="goLibrary sm success">Готово</button>`;
        break;
      case 'failed':
      case 'partial':
        btnHtml = `<button data-jobid="${esc(it.id)}" data-bookid="${esc(it.book_id)}" class="retryJob sm warning">Повторить</button>`;
        break;
      case 'canceled':
        btnHtml = `<button data-jobid="${esc(it.id)}" data-bookid="${esc(it.book_id)}" class="retryJob sm">Повторить</button>`;
        break;
      default: // running / queued
        btnHtml = `<button data-jobid="${esc(it.id)}" class="cancelJob sm">Отменить</button>`;
    }

    tr.innerHTML = `
      <td><span class="badge">${esc(it.id)}</span></td>
      <td>${esc(title)}</td>
      <td>${stateHtml}</td>
      <td>${esc(progressText)}</td>
      <td>${btnHtml}</td>
    `;
    tbody.appendChild(tr);
  }
  if ((data.items || []).length === 0) {
    tbody.innerHTML = '<tr class="empty-row"><td colspan="5">Нет задач (завершённые старше 2 часов скрываются)</td></tr>';
  }
}

// ── History ───────────────────────────────────────────────────────

async function refreshHistory() {
  const hint = document.getElementById('historyHint');
  const body = document.getElementById('historyBody');
  const kw = (document.getElementById('historyKeyword')?.value || '').toString().trim();
  if (!body) return;

  if (hint) hint.textContent = 'Загрузка…';
  body.innerHTML = '<tr class="empty-row"><td colspan="6">Загрузка…</td></tr>';

  const qs = new URLSearchParams();
  qs.set('limit', '200');
  if (kw) qs.set('q', kw);

  const data = await j(`/api/history?${qs.toString()}`);
  const items = data.items || [];

  body.innerHTML = '';
  for (const it of items) {
    const tr = document.createElement('tr');
    const status = (it.status || '').toString().toLowerCase();
    const badge = status === 'success'
      ? '<span class="badge success">Успех</span>'
      : '<span class="badge danger">Ошибка</span>';
    tr.innerHTML = `
      <td>${esc(it.timestamp || '')}</td>
      <td>${esc(it.book_name || '')}</td>
      <td>${esc(it.author || '')}</td>
      <td><code>${esc(it.book_id || '')}</code></td>
      <td>${esc(it.progress || '')}</td>
      <td>${badge}</td>
    `;
    body.appendChild(tr);
  }

  if (items.length === 0) {
    body.innerHTML = '<tr class="empty-row"><td colspan="6">История пуста</td></tr>';
  }
  if (hint) hint.textContent = `Всего: ${items.length}`;
}

// ── Updates ────────────────────────────────────────────────────────

let updatesPollTimer = null;

function scheduleUpdatesPoll() {
  if (updatesPollTimer) clearTimeout(updatesPollTimer);
  updatesPollTimer = setTimeout(() => {
    updatesPollTimer = null;
    refreshUpdates(false).catch(() => {});
  }, 1000);
}

async function refreshUpdates(start = true) {
  const hint = document.getElementById('updatesHint');
  const tbody = document.getElementById('updatesBody');
  if (!tbody) return;

  if (start && updatesPollTimer) {
    clearTimeout(updatesPollTimer);
    updatesPollTimer = null;
  }
  if (start) {
    if (hint) hint.textContent = 'Сканирование…';
    tbody.innerHTML = '<tr class="empty-row"><td colspan="7">Запуск сканирования…</td></tr>';
  }

  const data = await j(start ? '/api/updates' : '/api/updates?start=false');
  const updates = data.updates || [];
  const noUpdates = data.no_updates || [];
  const total = updates.length + noUpdates.length;
  const scanned = Number(data.scanned || total || 0);
  const expectedTotal = Number(data.total || total || 0);
  const running = !!data.running;

  if (hint) {
    if (data.error) {
      hint.textContent = `Ошибка сканирования: ${data.error}`;
    } else if (running) {
      const progress = expectedTotal > 0 ? `${scanned}/${expectedTotal}` : `${scanned}`;
      hint.textContent = `Сканирование ${progress}: обновляемых ${updates.length} / без обновлений ${noUpdates.length}`;
    } else {
      hint.textContent = `Обновляемых: ${updates.length} / без обновлений: ${noUpdates.length} / всего: ${total}`;
    }
  }

  tbody.innerHTML = '';
  for (const it of updates) {
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td>${esc(it.book_name || '')}</td>
      <td><code>${esc(it.book_id || '')}</code></td>
      <td>${esc(Number(it.local_total || 0))}</td>
      <td>${esc(Number(it.remote_total || 0))}</td>
      <td>${esc(Number(it.new_count || 0))}</td>
      <td>${esc(Number(it.local_failed || 0))}</td>
      <td><button data-bookid="${esc(it.book_id || '')}" class="startDownload sm primary">Обновить</button></td>
    `;
    tbody.appendChild(tr);
  }
  if (updates.length === 0) {
    if (running) {
      const text = scanned > 0 ? `Проверено ${scanned}, пока нет обновлений, сканирование продолжается…` : 'Сканирование, результаты появятся автоматически…';
      tbody.innerHTML = `<tr class="empty-row"><td colspan="7">${esc(text)}</td></tr>`;
    } else {
      tbody.innerHTML = '<tr class="empty-row"><td colspan="7">Нет книг с обновлениями</td></tr>';
    }
  }

  if (running) {
    scheduleUpdatesPoll();
  }
}

async function cancelJob(id) {
  await j(`/api/jobs/${encodeURIComponent(id)}/cancel`, { method: 'POST' });
  await refreshJobs();
}

async function clearJob(id) {
  await j(`/api/jobs/${encodeURIComponent(id)}`, { method: 'DELETE' });
}

// ── Book Name Modal ────────────────────────────────────────────────

function isBookNameModalOpen() {
  const modal = document.getElementById('bookNameModal');
  return modal && !modal.classList.contains('hidden');
}

function hideBookNameModal() {
  pendingBookNameJobId = null;
  pendingBookNameOptions = [];
  const modal = document.getElementById('bookNameModal');
  if (modal) modal.classList.add('hidden');
  document.body.style.overflow = '';
}

function showBookNameModal(job) {
  pendingBookNameJobId = job.id;
  pendingBookNameOptions = job.book_name_options || [];
  const modal = document.getElementById('bookNameModal');
  const hint = document.getElementById('bookNameJobHint');
  const options = document.getElementById('bookNameOptions');
  if (!modal || !options) return;

  if (hint) {
    const title = job.title || job.book_id || '';
    hint.textContent = title ? `《${title}》` : '';
  }

  options.innerHTML = '';
  pendingBookNameOptions.forEach((opt, idx) => {
    const id = `bookNameOpt_${idx}`;
    const row = document.createElement('label');
    row.className = 'row';
    row.innerHTML = `
      <input type="radio" name="bookNameOpt" id="${id}" value="${esc(opt.value)}" ${idx === 0 ? 'checked' : ''} />
      <span>${esc(opt.label)}: ${esc(opt.value)}</span>
    `;
    options.appendChild(row);
  });
  document.body.style.overflow = 'hidden';
  modal.classList.remove('hidden');
}

async function submitBookNameChoice(value) {
  if (!pendingBookNameJobId) return;
  await j(`/api/jobs/${encodeURIComponent(pendingBookNameJobId)}/book_name`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ value })
  });
  hideBookNameModal();
  await refreshJobs();
}

function isFormatModalOpen() {
  const modal = document.getElementById('formatModal');
  return modal && !modal.classList.contains('hidden');
}

function hideFormatModal() {
  pendingFormatJobId = null;
  pendingFormatOptions = [];
  const modal = document.getElementById('formatModal');
  if (modal) modal.classList.add('hidden');
  document.body.style.overflow = '';
}

function showFormatModal(job) {
  pendingFormatJobId = job.id;
  pendingFormatOptions = job.format_options || [];
  const modal = document.getElementById('formatModal');
  const hint = document.getElementById('formatJobHint');
  const options = document.getElementById('formatOptions');
  if (!modal || !options) return;

  if (hint) {
    const title = job.title || job.book_id || '';
    hint.textContent = title ? `《${title}》` : '';
  }

  options.innerHTML = '';
  pendingFormatOptions.forEach((opt, idx) => {
    const id = `formatOpt_${idx}`;
    const row = document.createElement('label');
    row.className = 'row';
    row.innerHTML = `
      <input type="radio" name="formatOpt" id="${id}" value="${esc(opt.value)}" ${idx === 0 ? 'checked' : ''} />
      <span>${esc(opt.label)}: ${esc(opt.value)}</span>
    `;
    options.appendChild(row);
  });
  document.body.style.overflow = 'hidden';
  modal.classList.remove('hidden');
}

async function submitFormatChoice(value) {
  if (!pendingFormatJobId) return;
  await j(`/api/jobs/${encodeURIComponent(pendingFormatJobId)}/format`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ value })
  });
  hideFormatModal();
  await refreshJobs();
}

async function openJobConfiguration(jobId, kindHint) {
  const data = await j(`/api/jobs?id=${encodeURIComponent(jobId)}`);
  const job = (data.items || [])[0];
  if (!job) {
    throw new Error('Задача не существует или уже удалена');
  }

  const hasBookNameOptions = (job.book_name_options || []).length > 0;
  const hasFormatOptions = (job.format_options || []).length > 0;

  if (hasBookNameOptions && (kindHint === 'book_name' || !hasFormatOptions)) {
    hideFormatModal();
    showBookNameModal(job);
    return;
  }

  if (hasFormatOptions) {
    hideBookNameModal();
    showFormatModal(job);
    return;
  }

  throw new Error('У текущей задачи нет параметров для настройки');
}

// ── Wire ───────────────────────────────────────────────────────────

function wire() {
  // -- Navigation --
  const navLinks = document.querySelectorAll('.nav a');
  const sections = document.querySelectorAll('.section');

  function switchSection(hash) {
    if (!hash) hash = '#status';
    navLinks.forEach(link => {
      link.classList.toggle('active', link.getAttribute('href') === hash);
    });
    sections.forEach(sec => {
      sec.classList.toggle('active', '#' + sec.id === hash);
    });
  }

  window.addEventListener('hashchange', () => switchSection(window.location.hash));
  switchSection(window.location.hash);

  // -- Theme Toggle --
  const themeBtn = document.getElementById('themeToggle');
  if (themeBtn) themeBtn.addEventListener('click', toggleTheme);
  updateThemeButton(getStoredTheme());

  // -- Config Tabs --
  const configTabs = document.querySelectorAll('.config-tab');
  const configPanels = {
    quick: document.getElementById('configPanelQuick'),
    full: document.getElementById('configPanelFull'),
    yaml: document.getElementById('configPanelYaml'),
  };
  let fullConfigLoaded = false;

  configTabs.forEach(tab => {
    tab.addEventListener('click', async () => {
      const target = tab.dataset.tab;
      configTabs.forEach(t => t.classList.toggle('active', t === tab));
      Object.entries(configPanels).forEach(([k, panel]) => {
        if (panel) panel.classList.toggle('active', k === target);
      });

      // Lazy-load full config on first switch
      if (target === 'full' && !fullConfigLoaded) {
        fullConfigLoaded = true;
        await loadFullConfigPanel();
      }
    });
  });

  // -- Library Back --
  const backBtn = document.getElementById('libBack');
  if (backBtn) {
    backBtn.addEventListener('click', async () => {
      const parts = (libraryPath || '').split('/').filter(Boolean);
      parts.pop();
      libraryPath = parts.join('/');
      try { await refreshLibrary(); } catch (err) { alert(err); }
    });
  }

  // -- Search --
  const searchForm = document.getElementById('searchForm');
  if (searchForm) {
    searchForm.addEventListener('submit', async (e) => {
      e.preventDefault();
      const q = document.getElementById('q').value.trim();
      const hint = document.getElementById('searchHint');
      if (hint) hint.textContent = '';

      const bookId = parseBookId(q);
      if (bookId) {
        try {
          await startDownload(bookId);
          if (hint) hint.textContent = `Создана задача загрузки: ${bookId}`;
          const out = document.getElementById('searchResults');
          if (out) out.innerHTML = '<tr class="empty-row"><td colspan="4">Добавлено в очередь — прогресс на вкладке «Задачи»</td></tr>';
        } catch (err) {
          if (hint) hint.textContent = 'Не удалось создать задачу';
          alert(err);
        }
        return;
      }
      try { await doSearch(q); } catch (err) { alert(err); }
    });
  }

  // -- Updates --
  const updBtn = document.getElementById('updatesRefresh');
  if (updBtn) updBtn.addEventListener('click', async () => {
    try { await refreshUpdates(); } catch (err) { alert(err); }
  });

  // -- App Update --
  const appUpdBtn = document.getElementById('appUpdateCheck');
  if (appUpdBtn) appUpdBtn.addEventListener('click', async () => {
    try { await refreshAppUpdate(true); } catch (err) { alert(err); }
  });

  const dismissBtn = document.getElementById('appUpdateDismiss');
  if (dismissBtn) dismissBtn.addEventListener('click', async () => {
    try {
      const { latestTag } = await refreshAppUpdate(false);
      if (latestTag) {
        setDismissedTag(latestTag);
        showAppUpdateBanner(false);
        const hint = document.getElementById('appUpdateHint');
        if (hint) hint.textContent = 'Напоминания отключены';
      }
    } catch (err) { alert(err); }
  });

  const selfUpdBtn = document.getElementById('appSelfUpdate');
  if (selfUpdBtn) selfUpdBtn.addEventListener('click', async () => {
    const hint = document.getElementById('appUpdateHint');
    if (hint) hint.textContent = 'Запуск самообновления…';
    try {
      await j('/api/self_update', { method: 'POST' });
      if (hint) hint.textContent = 'Самообновление запущено';
      await pollSelfUpdateStatus();
    } catch (err) {
      if (hint) hint.textContent = 'Не удалось запустить самообновление';
      alert(err);
    }
  });

  const historyRefresh = document.getElementById('historyRefresh');
  if (historyRefresh) historyRefresh.addEventListener('click', async () => {
    try { await refreshHistory(); } catch (err) { alert(err); }
  });

  const historyKeyword = document.getElementById('historyKeyword');
  if (historyKeyword) historyKeyword.addEventListener('keydown', async (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      try { await refreshHistory(); } catch (err) { alert(err); }
    }
  });

  // -- Quick Config Save --
  const cfgForm = document.getElementById('configForm');
  if (cfgForm) cfgForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const msg = document.getElementById('configMsg');
    if (msg) msg.textContent = 'Сохранение…';
    try {
      await saveConfig();
      if (msg) msg.textContent = 'Сохранено';
    } catch (err) {
      if (msg) msg.textContent = 'Ошибка сохранения';
      alert(err);
    }
  });

  // -- Full Config Save --
  const cfgFullSave = document.getElementById('cfgFullSave');
  if (cfgFullSave) cfgFullSave.addEventListener('click', async () => {
    const msg = document.getElementById('cfgFullMsg');
    if (msg) msg.textContent = 'Сохранение…';
    try {
      await saveFullConfig();
      await refreshConfig();
      await refreshRawConfig();
      if (msg) msg.textContent = 'Сохранено';
    } catch (err) {
      if (msg) msg.textContent = 'Ошибка сохранения';
      alert(err);
    }
  });

  // -- YAML Config --
  const cfgRawReload = document.getElementById('cfgRawReload');
  if (cfgRawReload) cfgRawReload.addEventListener('click', async () => {
    const msg = document.getElementById('cfgRawMsg');
    if (msg) msg.textContent = 'Загрузка…';
    try {
      await refreshRawConfig();
      if (msg) msg.textContent = 'Загружено';
    } catch (err) {
      if (msg) msg.textContent = 'Ошибка загрузки';
      alert(err);
    }
  });

  const cfgRawSave = document.getElementById('cfgRawSave');
  if (cfgRawSave) cfgRawSave.addEventListener('click', async () => {
    const msg = document.getElementById('cfgRawMsg');
    if (msg) msg.textContent = 'Сохранение…';
    try {
      await saveRawConfig();
      await refreshConfig();
      await refreshRawConfig();
      if (msg) msg.textContent = 'Сохранено';
    } catch (err) {
      if (msg) msg.textContent = 'Ошибка сохранения';
      alert(err);
    }
  });

  // -- Delegated Click Handlers --
  document.addEventListener('click', async (e) => {
    const t = e.target;
    if (!t || !t.classList) return;

    if (t.classList.contains('startDownload')) {
      const bookId = t.getAttribute('data-bookid');
      try { await startDownload(bookId); } catch (err) { alert(err); }
    }
    if (t.classList.contains('cancelJob')) {
      const id = t.getAttribute('data-jobid');
      if (!confirm('Отменить задачу и удалить её из списка?')) return;
      try { await cancelJob(id); } catch (err) { alert(err); }
    }
    if (t.classList.contains('retryJob')) {
      const bookId = t.getAttribute('data-bookid');
      const jobId = t.getAttribute('data-jobid');
      try {
        await startDownloadDirect(bookId);
        if (jobId) {
          await clearJob(jobId).catch(() => {});
        }
        await refreshJobs();
      } catch (err) { alert(err); }
    }
    if (t.classList.contains('configJob')) {
      const jobId = t.getAttribute('data-jobid');
      const kind = t.getAttribute('data-kind') || '';
      try { await openJobConfiguration(jobId, kind); } catch (err) { alert(err); }
    }
    if (t.classList.contains('goLibrary')) {
      const title = t.getAttribute('data-title') || '';
      const jobId = t.getAttribute('data-jobid');
      if (jobId) {
        await clearJob(jobId).catch(() => {});
        await refreshJobs().catch(() => {});
      }
      libraryPath = '';
      window.location.hash = '#library';
      await refreshLibrary();
      highlightLibraryItem(title);
    }
    if (t.classList.contains('openDir')) {
      const p = (t.getAttribute('data-path') || '').toString();
      libraryPath = p;
      try { await refreshLibrary(); } catch (err) { alert(err); }
    }
  });

  // -- Escape Key for Modals --
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      const previewModal = document.getElementById('previewModal');
      if (previewModal && !previewModal.classList.contains('hidden')) {
        showPreviewModal(false);
        return;
      }
      const bookNameModal = document.getElementById('bookNameModal');
      if (bookNameModal && !bookNameModal.classList.contains('hidden')) {
        hideBookNameModal();
        return;
      }
      const formatModal = document.getElementById('formatModal');
      if (formatModal && !formatModal.classList.contains('hidden')) {
        hideFormatModal();
        return;
      }
      const iidWarningModal = document.getElementById('iidWarningModal');
      if (iidWarningModal && !iidWarningModal.classList.contains('hidden')) {
        showIidWarningModal(false);
        return;
      }
      const loginModal = document.getElementById('loginModal');
      if (loginModal && !loginModal.classList.contains('hidden')) {
        showLogin(false);
      }
    }
  });

  // -- Preview Modal Buttons --
  const previewConfirm = document.getElementById('previewConfirm');
  if (previewConfirm) previewConfirm.addEventListener('click', async () => {
    try { await confirmPreview(); } catch (err) { alert(err); }
  });

  const previewCancel = document.getElementById('previewCancel');
  if (previewCancel) previewCancel.addEventListener('click', () => showPreviewModal(false));

  const previewClose = document.getElementById('previewClose');
  if (previewClose) previewClose.addEventListener('click', () => showPreviewModal(false));

  // -- Book Name Modal --
  const bookNameConfirm = document.getElementById('bookNameConfirm');
  if (bookNameConfirm) bookNameConfirm.addEventListener('click', async () => {
    const selected = document.querySelector('input[name="bookNameOpt"]:checked');
    if (!selected) { alert('Выберите название'); return; }
    await submitBookNameChoice(selected.value);
  });

  const bookNameClose = document.getElementById('bookNameClose');
  if (bookNameClose) bookNameClose.addEventListener('click', () => hideBookNameModal());

  const formatConfirm = document.getElementById('formatConfirm');
  if (formatConfirm) formatConfirm.addEventListener('click', async () => {
    const selected = document.querySelector('input[name="formatOpt"]:checked');
    if (!selected) { alert('Выберите формат вывода'); return; }
    await submitFormatChoice(selected.value);
  });

  const formatClose = document.getElementById('formatClose');
  if (formatClose) formatClose.addEventListener('click', () => hideFormatModal());

  const iidWarningClose = document.getElementById('iidWarningClose');
  if (iidWarningClose) iidWarningClose.addEventListener('click', () => showIidWarningModal(false));
  const iidWarningOk = document.getElementById('iidWarningOk');
  if (iidWarningOk) iidWarningOk.addEventListener('click', () => showIidWarningModal(false));
}

function highlightLibraryItem(title) {
  if (!title) return;
  const rows = document.querySelectorAll('#libraryBody tr');
  for (const row of rows) {
    const firstTd = row.querySelector('td');
    if (firstTd && firstTd.textContent.includes(title)) {
      row.classList.add('lib-highlight');
      row.scrollIntoView({ behavior: 'smooth', block: 'center' });
      setTimeout(() => row.classList.remove('lib-highlight'), 3000);
      break;
    }
  }
}

// ── Boot ───────────────────────────────────────────────────────────

async function boot() {
  wire();
  await refreshStatus();
  await refreshConfig();
  await refreshRawConfig();
  await Promise.allSettled([
    refreshJobs(),
    refreshHistory(),
    refreshLibrary(false),
  ]);
  refreshUpdates().catch(() => {});
  if (!isDockerBuild) refreshAppUpdate(false).catch(() => {});
  pollSelfUpdateStatus().catch(() => {});
  setInterval(() => refreshJobs().catch(() => {}), 1500);
  setInterval(() => refreshStatus().catch(() => {}), 5000);
  if (!isDockerBuild) {
    setInterval(() => refreshAppUpdate(false).catch(() => {}), 6 * 60 * 60 * 1000);
  }
}

boot().catch(err => console.error(err));
