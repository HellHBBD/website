#!/usr/bin/env bash

set -euo pipefail

dx bundle --release --platform web --verbose --debug-symbols false

# 顏色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

PUBLIC_SOURCE="./target/dx/website/release/web/public"
SERVER_SOURCE="./target/x86_64-unknown-linux-gnu/server-release/website"
DEST_DIR="hellhbbd@home:~/website"
CADDYFILE="deploy/Caddyfile"
RUN_SCRIPT="scripts/run.sh"

# 進度顯示函數
show_progress() {
    echo -e "${CYAN}🔄 $1...${NC}"
}

show_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

show_error() {
    echo -e "${RED}❌ $1${NC}"
}

show_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

check_required_files() {
    local missing_files=()

    if [ ! -d "$PUBLIC_SOURCE" ]; then
        missing_files+=("public 目錄")
    fi

    if [ ! -f "${PUBLIC_SOURCE}/index.html" ]; then
        missing_files+=("index.html")
    fi

    if [ ! -d "${PUBLIC_SOURCE}/assets" ]; then
        missing_files+=("assets 目錄")
    fi

    if [ ! -x "$SERVER_SOURCE" ]; then
        missing_files+=("website 執行檔")
    fi

    if [ ! -f "$RUN_SCRIPT" ]; then
        missing_files+=("run.sh")
    fi

    if [ ! -f "$CADDYFILE" ]; then
        missing_files+=("Caddyfile")
    fi

    if [ ${#missing_files[@]} -ne 0 ]; then
        show_error "缺少必要檔案: ${missing_files[*]}"
        show_error "請先執行 build.sh 編譯專案"
        exit 1
    fi
}

show_progress "開始部署檢查"

# 檢查必要檔案
check_required_files

show_success "必要檔案檢查通過"

DEPLOY_DIR="$(mktemp -d)"
cleanup_tmp() {
    rm -rf "$DEPLOY_DIR"
}
trap cleanup_tmp EXIT

show_progress "準備部署內容"
mkdir -p "$DEPLOY_DIR/public"
rsync -a --delete "${PUBLIC_SOURCE}/" "$DEPLOY_DIR/public/"
cp -p "$RUN_SCRIPT" "$DEPLOY_DIR/run.sh"
cp -p "$CADDYFILE" "$DEPLOY_DIR/Caddyfile"
cp -p "$SERVER_SOURCE" "$DEPLOY_DIR/website"
show_success "部署內容準備完成"

# --- rsync command ---
show_progress "同步檔案到伺服器"
echo -e "${PURPLE}來源: ${WHITE}${DEPLOY_DIR}${NC}"
echo -e "${PURPLE}目標: ${WHITE}${DEST_DIR}${NC}"

rsync -avh --progress --delete \
	--exclude '.env' \
	--exclude 'tasks.json' \
	--exclude 'tasks.json.tmp' \
	--filter 'protect .env' \
	--filter 'protect tasks.json' \
	--filter 'protect tasks.json.tmp' \
	"${DEPLOY_DIR}/" "${DEST_DIR}/"

if [ $? -eq 0 ]; then
    show_success "同步完成"
    show_info "部署成功！服務已更新"
else
    show_error "同步失敗，請檢查網路連線或權限"
    exit 1
fi
