#!/usr/bin/env bash
# 
# Имя файла: installer.sh
# Назначение:
#   1. Автоматически получает последнюю версию Tomato-Novel-Downloader через GitHub API
#   2. Спрашивает путь установки (по умолчанию — каталог запуска скрипта; в Termux — $HOME)
#   3. Поддерживает 2 способа загрузки:
#        (1) напрямую с GitHub
#        (2) через ускоряющее зеркало проекта (https://dl.zhongbai233.com/)
#   4. В среде Termux создаёт run.sh (по умолчанию --server)
#   5. На Linux / macOS (arm64 и Intel x86_64) скачивает бинарник нужной архитектуры и выдаёт права на выполнение
# 
# Использование:

#   chmod +x installer.sh
#   ./installer.sh

set -e

#####################################
# 0. Общие вспомогательные функции
#####################################

log_info()  { printf "\033[1;32m[INFO]\033[0m %s\n" "$*"; }
log_warn()  { printf "\033[1;33m[WARN]\033[0m %s\n" "$*"; }
log_error() { printf "\033[1;31m[ERR ]\033[0m %s\n" "$*" >&2; }

command_exists() { command -v "$1" >/dev/null 2>&1; }

IS_TERMUX=false
if [ -n "${PREFIX:-}" ]; then
    case "${PREFIX}" in
        *com.termux*|*bin.mt.plus*|*com.duoduo.mt*)
            IS_TERMUX=true
            ;;
    esac
fi

IS_MUSL=false
if command_exists ldd; then
    if ldd --version 2>&1 | grep -qi musl; then
        IS_MUSL=true
    fi
fi
# Запасной вариант: типичные пути загрузчика musl
if [ -e /lib/ld-musl-x86_64.so.1 ] || [ -e /lib/ld-musl-aarch64.so.1 ] || [ -e /lib/ld-musl-armhf.so.1 ]; then
    IS_MUSL=true
fi

DEFAULT_DIR="$(pwd)"
if $IS_TERMUX; then
    DEFAULT_DIR="${HOME}"
fi

echo ""
echo "Введите каталог установки (по умолчанию: ${DEFAULT_DIR}):"
read -r INPUT_DIR
if [ -z "${INPUT_DIR}" ]; then
    INSTALL_DIR="${DEFAULT_DIR}"
else
    INSTALL_DIR="${INPUT_DIR}"
fi

if $IS_TERMUX; then
    case "$INSTALL_DIR" in
        "$HOME"*|"$PREFIX"*)
            ;;
        *)
            echo ""
            log_warn "Обнаружен Termux: выбранный каталог установки может быть недоступен для выполнения (возможен Permission denied)."
            log_warn "Рекомендуется устанавливать внутрь каталогов Termux (HOME или PREFIX):"
            echo "  - ${HOME}"
            echo "  - ${PREFIX}"
            echo ""
            echo "Всё равно продолжить с этим каталогом? (y/N):"
            read -r CONFIRM_DIR
            case "$CONFIRM_DIR" in
                y|Y) ;;
                *)
                    INSTALL_DIR="${HOME}"
                    log_info "Каталог установки изменён на: ${INSTALL_DIR}"
                    ;;
            esac
            ;;
    esac
fi

if [ ! -d "$INSTALL_DIR" ]; then
    echo ""
    log_warn "Каталог не существует. Создать: ${INSTALL_DIR} ? (y/N):"
    read -r CREATE_DIR
    case "$CREATE_DIR" in
        y|Y)
            mkdir -p "$INSTALL_DIR"
            log_info "Каталог создан: ${INSTALL_DIR}"
            ;;
        *)
            log_warn "Каталог не создан, установка прервана."
            exit 1
            ;;
    esac
fi

echo ""
log_info "Получение сведений о последней версии через GitHub API..."
GITHUB_API_URL="https://api.github.com/repos/zhongbai2333/Tomato-Novel-Downloader/releases/latest"
if command_exists curl; then
    TAG_NAME=$(curl -s "${GITHUB_API_URL}" | grep -m1 '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
elif command_exists wget; then
    TAG_NAME=$(wget -qO- "${GITHUB_API_URL}" | grep -m1 '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
else
    log_error "В системе не найдены curl или wget. Установите один из них."
    exit 1
fi

if [ -z "${TAG_NAME}" ]; then
    log_error "Не удалось получить tag_name из GitHub API. Проверьте сеть или существование репозитория."
    exit 1
fi

VERSION="${TAG_NAME#v}"
log_info "Последняя версия: ${TAG_NAME} (VERSION=${VERSION})"

echo ""
echo "Выберите способ загрузки (введите номер, по умолчанию 1):"
echo "  1) Напрямую с GitHub"
echo "  2) Через ускоряющее зеркало проекта (https://dl.zhongbai233.com/)"
read -r ACCEL_CHOICE
ACCEL_CHOICE="${ACCEL_CHOICE:-1}"
case "$ACCEL_CHOICE" in
    1) ACCEL_METHOD="direct" ;;
    2) ACCEL_METHOD="accel" ;;
    *) log_warn "Неверный ввод, используется прямая загрузка по умолчанию."; ACCEL_METHOD="direct" ;;
esac
log_info "Выбранный способ загрузки: ${ACCEL_METHOD}"

PLATFORM="$(uname)"
ARCH="$(uname -m)"
BINARY_NAME=""
case "$PLATFORM" in
    Linux)
        if $IS_TERMUX; then
            # Определение архитектуры Termux: aarch64 → arm64, armv7l → arm32
            case "$ARCH" in
                aarch64|arm64)
                    ANDROID_ARCH="arm64"
                    ;;
                armv7l|arm)
                    ANDROID_ARCH="arm32"
                    ;;
                *)
                    log_error "Неподдерживаемая архитектура Android [${ARCH}]! Поддерживаются только aarch64/arm64 и armv7l/arm."
                    exit 1
                    ;;
            esac
            BINARY_NAME="TomatoNovelDownloader-Android_${ANDROID_ARCH}-v${VERSION}"
            log_info "Обнаружен Termux (архитектура: ${ANDROID_ARCH}), будет использована нативная Android-версия."
        else
            case "$ARCH" in
                x86_64|amd64)
                    if $IS_MUSL; then
                        BINARY_NAME="TomatoNovelDownloader-Linux_musl_amd64-v${VERSION}"
                    else
                        BINARY_NAME="TomatoNovelDownloader-Linux_amd64-v${VERSION}"
                    fi
                    ;;
                aarch64|arm64)
                    if $IS_MUSL; then
                        BINARY_NAME="TomatoNovelDownloader-Linux_musl_arm64-v${VERSION}"
                    else
                        BINARY_NAME="TomatoNovelDownloader-Linux_arm64-v${VERSION}"
                    fi
                    ;;
                *)
                    log_error "Неподдерживаемая архитектура Linux [${ARCH}]! Поддерживаются только x86_64/amd64 и aarch64/arm64."
                    exit 1
                    ;;
            esac
        fi
        ;;
    Darwin)
        case "$ARCH" in
            arm64)
                BINARY_NAME="TomatoNovelDownloader-macOS_arm64-v${VERSION}"
                ;;
            x86_64)
                BINARY_NAME="TomatoNovelDownloader-macOS_amd64-v${VERSION}"
                ;;
            *)
                log_error "Неподдерживаемая архитектура macOS [${ARCH}]! Поддерживаются только arm64 и x86_64."
                exit 1
                ;;
        esac
        ;;
    *)
        log_error "Неподдерживаемая платформа [${PLATFORM}]! Поддерживаются только Linux, macOS (Darwin) и Termux."
        exit 1
        ;;
esac

ORIGINAL_URL="https://github.com/zhongbai2333/Tomato-Novel-Downloader/releases/download/${TAG_NAME}/${BINARY_NAME}"
DOWNLOAD_URL="$ORIGINAL_URL"
case "$ACCEL_METHOD" in
    direct) ;;
    accel) DOWNLOAD_URL="https://dl.zhongbai233.com/release/${TAG_NAME}/${BINARY_NAME}" ;;
esac

echo ""
log_info "Подготовка к загрузке: ${BINARY_NAME}"
echo "Ссылка для загрузки: ${DOWNLOAD_URL}"
echo "Каталог установки: ${INSTALL_DIR}"

TARGET_BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"
if [ -f "$TARGET_BINARY_PATH" ]; then
    log_warn "В целевом каталоге уже есть файл с таким именем, он будет перезаписан: ${TARGET_BINARY_PATH}"
    rm -f "$TARGET_BINARY_PATH"
fi

download_file() {
    if command_exists wget; then
        wget -4 -q --show-progress -O "${TARGET_BINARY_PATH}" "${DOWNLOAD_URL}"
    elif command_exists curl; then
        curl -4 -L -o "${TARGET_BINARY_PATH}" "${DOWNLOAD_URL}"
    else
        log_error "Не найдены wget или curl. Установите один из них."
        return 127
    fi
}

log_info "Начало загрузки..."
download_file || {
    log_error "Загрузка не удалась. Проверьте сеть, прокси или URL."
    exit 1
}

if [ ! -f "$TARGET_BINARY_PATH" ] || [ ! -s "$TARGET_BINARY_PATH" ]; then
    log_error "Загруженный файл отсутствует или пуст."
    exit 1
fi

chmod +x "$TARGET_BINARY_PATH"
log_info "Загрузка завершена, права на выполнение выданы: ${TARGET_BINARY_PATH}"

# Переименование в каноническое имя (без номера версии), как после автообновления программы
CANONICAL_NAME="${BINARY_NAME%-v*}"
if [ "$CANONICAL_NAME" != "$BINARY_NAME" ]; then
    CANONICAL_PATH="${INSTALL_DIR}/${CANONICAL_NAME}"
    mv "${TARGET_BINARY_PATH}" "${CANONICAL_PATH}"
    chmod +x "${CANONICAL_PATH}"
    TARGET_BINARY_PATH="${CANONICAL_PATH}"
    log_info "Переименовано в каноническое имя: ${CANONICAL_NAME}"
fi

if $IS_TERMUX; then
    echo ""
    log_info "Создание run.sh..."
    RUN_SH_PATH="${INSTALL_DIR}/run.sh"
    cat > "$RUN_SH_PATH" <<EOF
#!/usr/bin/env bash
# Среда Termux / MT Manager: запуск нативного Android TomatoNovelDownloader (по умолчанию режим Web UI-сервера)
# Адрес прослушивания и пароль можно задать переменными окружения:
#   TOMATO_WEB_ADDR=0.0.0.0:18423
#   TOMATO_WEB_PASSWORD=ваш_пароль
SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
termux-open-url "http://127.0.0.1:18423/" >/dev/null 2>&1 || true
exec "\${SCRIPT_DIR}/${CANONICAL_NAME}" --server "\$@"
EOF
    chmod +x "$RUN_SH_PATH"
    log_info "Создан: ${RUN_SH_PATH}"

    echo ""
    echo "Установка завершена. Выполните:"
    echo "    cd ${INSTALL_DIR}"
    echo "    ./run.sh"
    echo ""
    echo "Подсказка: если при запуске появляется Permission denied, разместите каталог установки внутри Termux (рекомендуется ${HOME})."
elif [ "$PLATFORM" = "Linux" ]; then
    echo ""
    log_info "Обнаружена среда Linux."
    echo "Установка завершена, файл находится здесь: ${TARGET_BINARY_PATH}"
    echo "Запуск:"
    echo "    cd ${INSTALL_DIR}"
    echo "    ./${CANONICAL_NAME}"
elif [ "$PLATFORM" = "Darwin" ]; then
    echo ""
    log_info "Обнаружена среда macOS."
    echo "Установка завершена, файл находится здесь: ${TARGET_BINARY_PATH}"
    echo "Запуск:"
    echo "    cd ${INSTALL_DIR}"
    echo "    ./${CANONICAL_NAME}"
fi

log_info "Всё готово."
exit 0
