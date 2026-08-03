#!/bin/bash

# 顏色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# 設置你的命令
WEB_SERVER_CMD="./website"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

if ! cd "$SCRIPT_DIR"; then
    echo "Unable to enter the deployment directory" >&2
    exit 1
fi

# 進度顯示函數
show_progress() {
    local message=$1
    echo -e "${CYAN}🔄 ${message}...${NC}"
}

show_success() {
    local message=$1
    echo -e "${GREEN}✅ ${message}${NC}"
}

show_warning() {
    local message=$1
    echo -e "${YELLOW}⚠️  ${message}${NC}"
}

show_error() {
    local message=$1
    echo -e "${RED}❌ ${message}${NC}"
}

show_info() {
    local message=$1
    echo -e "${BLUE}ℹ️  ${message}${NC}"
}

# 錯誤處理函數
cleanup() {
    show_warning "停止服務..."

    # 檢查進程是否仍在運行並停止它們
    if [ ! -z "$WEB_PID" ] && kill -0 $WEB_PID 2>/dev/null; then
        show_info "停止 Web Server (PID: $WEB_PID)"
        kill $WEB_PID
    fi

    if [ ! -z "$CADDY_PID" ] && kill -0 $CADDY_PID 2>/dev/null; then
        show_info "停止 Caddy (PID: $CADDY_PID)"
        kill $CADDY_PID
    fi

    # 等待進程結束
    wait 2>/dev/null
    show_success "所有服務已停止"
    exit 0
}

# 設置信號捕獲
trap cleanup INT TERM EXIT

show_progress "啟動服務"

# Do not expose detailed panic information in production logs.
export RUST_BACKTRACE=0
export RUST_LOG=info
# Caddy is the only public entry point and enforces authentication.
export IP=127.0.0.1
export PORT=8080

# 檢查 Web Server 執行檔是否存在
if [ ! -x "$WEB_SERVER_CMD" ]; then
    show_error "Web Server 執行檔不存在或沒有執行權限: $WEB_SERVER_CMD"
    show_progress "正在編譯專案..."
    dx bundle --release --platform web --verbose

    if [ ! -x "$WEB_SERVER_CMD" ]; then
        show_error "編譯失敗，請檢查錯誤訊息"
        exit 1
    fi
    show_success "編譯完成"
fi

# 檢查 Caddyfile 是否存在
if [ ! -f "Caddyfile" ]; then
    show_error "Caddyfile 不存在"
    exit 1
fi

# Load the deployment-only Caddy credentials regardless of the caller's directory.
if [ ! -r "$SCRIPT_DIR/.env" ]; then
    show_error ".env 不存在或無法讀取"
    exit 1
fi

set -a
if ! . "$SCRIPT_DIR/.env"; then
    set +a
    show_error ".env 載入失敗"
    exit 1
fi
set +a
: "${CADDY_BASIC_AUTH_USER:?CADDY_BASIC_AUTH_USER must be set}"
: "${CADDY_BASIC_AUTH_HASH:?CADDY_BASIC_AUTH_HASH must be set}"

# 啟動 Caddy
show_progress "啟動 Caddy"
caddy run --config Caddyfile &
CADDY_PID=$!

# 檢查 Caddy 是否成功啟動
sleep 1
if ! kill -0 $CADDY_PID 2>/dev/null; then
    show_error "Caddy 啟動失敗"
    cleanup
    exit 1
fi

show_success "Caddy PID: ${WHITE}$CADDY_PID${NC}"
show_success "所有服務已成功啟動"
show_info "按 Ctrl+C 停止所有服務"

# 啟動 Web Server 並在失敗時自動重啟
while true; do
    # 檢查 8080 端口是否被佔用
    if lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null; then
        show_warning "端口 8080 已被佔用，可能 Web Server 未完全關閉。將在 5 秒後重試..."
        sleep 5
        continue
    fi

    show_progress "啟動 Web Server (端口 8080)"
    $WEB_SERVER_CMD
    exit_code=$?
    show_error "Web Server 意外終止 (Exit Code: $exit_code)，將在 5 秒後重啟..."
    sleep 5
done
