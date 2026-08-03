# 部署機設定與維運

本文件說明如何從全新 Arch Linux 主機建立目前網站的手動部署環境。
目前服務由 `hellhbbd` 在互動式終端執行 `./run.sh` 管理；systemd 管理
服務與任務資料輪替備份尚未導入。

## 架構與責任邊界

```text
Internet
  -> Cloudflare proxy
  -> router TCP 80/443 forwarding
  -> UFW: Cloudflare networks only
  -> Caddy TCP 80/443
  -> Dioxus server 127.0.0.1:8080

LAN 192.168.0.0/24
  -> SSH TCP 22
```

- 部署主機使用者：`hellhbbd`。
- 部署目錄：`/home/hellhbbd/website`。
- 網站：`https://website.hellhbbd.pp.ua`。
- 建置機可由使用者執行 `./scripts/deploy.sh`。該腳本使用 rsync over SSH
  寫入部署目錄。
- 本文件中的「部署機」命令必須在 `home` 的本機終端執行。協作者的 SSH
  操作只能用於唯讀檢查。
- 絕不可使用 `sudo ./run.sh`。Caddy 和網站程序必須以 `hellhbbd` 執行。
- `.env`、`tasks.json` 和其暫存檔不得提交到 Git，不得經由一般部署覆寫。

## 1. 建置機前置條件

建置機需要與部署主機相同目標架構的 Rust 工具鏈、Dioxus CLI、Caddy、
rsync 和 SSH 連線設定。部署主機不應在正式部署時現場編譯 Rust 專案。

建立可連線部署主機的 SSH alias，例如 `~/.ssh/config`：

```sshconfig
Host home
    HostName 192.168.0.114
    User hellhbbd
    IdentityFile ~/.ssh/id_ed25519
    IdentitiesOnly yes
```

確認建置機與專案的 Dioxus 版本相容後，執行：

```sh
./scripts/test.sh
./scripts/deploy.sh
```

`deploy.sh` 會建立 release bundle，再同步 `website`、`public/`、`run.sh`
和 `Caddyfile`。它排除並保護部署機既有的 `.env`、`tasks.json` 與
`tasks.json.tmp`。同步成功只表示檔案已傳送，並不代表新服務已啟用。

## 2. 部署主機基礎設定

以下在部署機本機以一般使用者登入後執行。安裝或調整系統套件時才使用
`sudo`：

```sh
sudo pacman -Syu caddy ufw fail2ban rsync lsof openssl
```

建立部署帳號與目錄。若帳號已存在，只檢查所有權和權限：

```sh
id hellhbbd
install -d -m 700 /home/hellhbbd/website
chown hellhbbd:hellhbbd /home/hellhbbd/website
```

部署目錄的預期權限：

```text
/home/hellhbbd/website       0700 hellhbbd:hellhbbd
/home/hellhbbd/website/.env  0600 hellhbbd:hellhbbd
/home/hellhbbd/website/tasks.json 0600 hellhbbd:hellhbbd
```

部署後檢查：

```sh
stat -c '%A %U:%G %n' \
  /home/hellhbbd/website \
  /home/hellhbbd/website/.env \
  /home/hellhbbd/website/tasks.json
```

## 3. 網路與路由器

1. 對部署機設定 DHCP reservation，讓 LAN 位址固定為 `192.168.0.114`。
2. 只將 router TCP 80 和 TCP 443 轉發至 `192.168.0.114`。
3. 移除所有公網 TCP 22 port forwarding。
4. 不轉發 TCP 8080、UDP 443 或 RustDesk 的 inbound port。
5. DNS 記錄 `website.hellhbbd.pp.ua` 指向 router 的公網位址，並在
   Cloudflare 開啟 proxy。

DNS 只提供名稱解析，不會保護 SSH。日後需要非 LAN SSH 時，使用
Tailscale/MagicDNS 或 Cloudflare Tunnel + Access；不要重新公開 TCP 22。

## 4. Cloudflare 設定

在 Cloudflare dashboard 設定：

1. `website.hellhbbd.pp.ua` 的 A/AAAA 記錄開啟橘雲 proxy。
2. SSL/TLS encryption mode 設為 `Full (strict)`。
3. 對 `/schedule*` 和 `/api/*` 建立 cache bypass 規則。Caddy 同時對這些
   路徑回傳 `Cache-Control: no-store`。
4. 對 `/schedule*` 和 `/api/*` 建立 rate limit 或 Managed Challenge 規則。
5. 關閉 Browser Insights，因網站 CSP 不允許它注入的 script。

HTTP-01 TLS 憑證驗證需要外部 TCP 80 可到達 Caddy。完成 UFW 後，這只會
允許 Cloudflare 網段進入；Cloudflare proxy 必須保持開啟。

## 5. Caddy 和 Basic Auth

### 5.1 允許非 root 綁定低埠

目前手動服務以 `hellhbbd` 執行，因此 Caddy 需要綁定 TCP 80/443 的
capability：

```sh
sudo setcap cap_net_bind_service=+ep /usr/bin/caddy
getcap /usr/bin/caddy
```

預期輸出：

```text
/usr/bin/caddy cap_net_bind_service=ep
```

每次更新 Caddy 套件後再次執行 `getcap /usr/bin/caddy`。套件升級可能會
移除 capability，屆時重新執行 `setcap`。

### 5.2 建立 Basic Auth 環境檔

在部署機本機建立 hash。省略 `--plaintext` 時，Caddy 會從 TTY 讀取密碼，
不會顯示輸入內容，也不會將明文放到 shell history：

```sh
caddy hash-password --algorithm bcrypt
```

將輸出的完整 hash 寫入 `/home/hellhbbd/website/.env`：

```dotenv
CADDY_BASIC_AUTH_USER='replace-with-login-name'
CADDY_BASIC_AUTH_HASH='$2a$14$replace-with-caddy-output'
```

設定嚴格權限：

```sh
chmod 600 /home/hellhbbd/website/.env
chown hellhbbd:hellhbbd /home/hellhbbd/website/.env
```

不要將 `.env`、明文密碼或 hash 貼到 issue、commit、log 或聊天紀錄。

### 5.3 Caddyfile 注意事項

部署的 Caddyfile 位於 `/home/hellhbbd/website/Caddyfile`，並設定：

- `admin off`。
- TLS 1.2 和 1.3。
- `/schedule`、`/api/*` 的 Basic Auth 和 no-store。
- API 只接受 POST。
- cross-site 與不合法 Origin 的 API 請求回 `403`。
- `.env`、`.git`、`tasks.json*` 和常見掃描路徑回 `404`。
- Dioxus upstream 固定為 `127.0.0.1:8080`。

因為 `admin off`，不要執行 `caddy reload`。更新 Caddyfile 後，停止目前的
`run.sh` 並完整重啟服務。

啟動前可以驗證設定。此命令需要 `.env` 中的 Basic Auth 變數：

```sh
cd /home/hellhbbd/website
set -a
. ./.env
set +a
caddy validate --config Caddyfile
```

## 6. UFW Cloudflare-only

此步驟會更改網路連線。保留目前 LAN SSH 工作階段，並準備第二個 LAN 終端
驗證 TCP 22，才可繼續。

### 6.1 重設 Fail2ban 的永久封鎖歷史

舊設定使用 `maxretry = 1` 和 `bantime = -1`，曾導致 15,901 條永久封鎖在
重開機後逐一恢復到 legacy iptables，造成 CPU 使用率偏高和 xtables lock。
不要刪除 `/run/xtables.lock`，也不要強制 kill 任何 iptables 程序。

先停止 Fail2ban 並確認 lock 已釋放：

```sh
sudo systemctl stop fail2ban
systemctl is-active fail2ban
pgrep -a iptables
pgrep -a ip6tables
sudo lsof /run/xtables.lock
```

`fail2ban` 必須是 `inactive`，且兩個 `pgrep` 和 `lsof` 都沒有輸出。否則
停止此流程並先調查程序。

封存資料庫並調整 `/etc/fail2ban/jail.local`：

```sh
stamp=$(date +%Y%m%d-%H%M%S)
sudo mv /var/lib/fail2ban/fail2ban.sqlite3 \
  "/var/lib/fail2ban/fail2ban.sqlite3.before-reset-$stamp"
sudoedit /etc/fail2ban/jail.local
```

使用以下設定：

```ini
[sshd]
enabled = true
backend = systemd
maxretry = 5
findtime = 10m
bantime = 1h
```

驗證設定，但此時先不要啟動 Fail2ban：

```sh
sudo fail2ban-client -t
```

### 6.2 建立 UFW 規則

每次套用前，從 Cloudflare 官方端點取得最新 IP ranges。不要將舊清單視為
永久有效：

```sh
curl -fsS https://www.cloudflare.com/ips-v4
curl -fsS https://www.cloudflare.com/ips-v6
```

先允許 LAN SSH：

```sh
sudo ufw allow proto tcp from 192.168.0.0/24 to any port 22 comment 'LAN SSH'
```

人工檢查官方輸出後，將 IPv4 和 IPv6 CIDR 逐一加入 TCP 80 和 TCP 443。
下列指令從官方清單讀取，每一條都新增 `Cloudflare web` 註解：

```sh
while IFS= read -r cidr; do
  sudo ufw allow proto tcp from "$cidr" to any port 80,443 comment 'Cloudflare web'
done < <(curl -fsS https://www.cloudflare.com/ips-v4)

while IFS= read -r cidr; do
  sudo ufw allow proto tcp from "$cidr" to any port 80,443 comment 'Cloudflare web'
done < <(curl -fsS https://www.cloudflare.com/ips-v6)
```

檢查既有規則：

```sh
sudo ufw status numbered
sudo ufw show added
```

移除任何舊的全域公開規則。以下是在目前已知舊規則仍存在時的刪除命令：

```sh
sudo ufw delete allow 80
sudo ufw delete allow 443
sudo ufw delete allow 80/tcp
sudo ufw delete allow 443/tcp
sudo ufw delete allow 22/tcp
```

如果命令指出 rule 不存在，這表示它已被移除，繼續檢查最終規則即可。若使用
rule number 刪除，必須每刪除一條後重新執行 `sudo ufw status numbered`，因為
UFW 會重新編號。

設定預設政策並啟用：

```sh
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw default deny routed
sudo ufw logging low
sudo ufw --force enable
sudo systemctl enable --now ufw.service
```

目標規則只有：

```text
192.168.0.0/24 -> TCP 22
Cloudflare IPv4/IPv6 -> TCP 80,443
所有其他 inbound -> deny
所有 outbound -> allow
```

不開放 UDP 443、TCP 8080 或 RustDesk inbound。重啟 Fail2ban：

```sh
sudo systemctl start fail2ban
sudo fail2ban-client status sshd
sudo fail2ban-client get sshd maxretry
sudo fail2ban-client get sshd findtime
sudo fail2ban-client get sshd bantime
```

預期初始結果為 `Currently banned: 0`、`5`、`600`、`3600`。

### 6.3 防火牆回復

如果網站變成 Cloudflare `521`，在部署機本機執行：

```sh
sudo ufw disable
```

確認 Cloudflare IP ranges 和 router forwarding 後再重新啟用。Fail2ban 再次
異常時，可單獨停止它；UFW 規則仍會保護網站：

```sh
sudo systemctl stop fail2ban
```

## 7. 首次部署與啟動

在建置機完成 `./scripts/deploy.sh` 後，回到部署機本機：

```sh
cd /home/hellhbbd/website
chmod 700 run.sh
chmod 755 website
./run.sh
```

`run.sh` 會：

1. 載入同目錄 `.env`。
2. 將 Dioxus 綁定在 `127.0.0.1:8080`。
3. 啟動 Caddy。
4. 前景執行網站程序；網站異常結束時每 5 秒重試。

該終端必須保持開啟。按 Ctrl+C 會停止 Caddy 與網站；重開機後也必須由
`hellhbbd` 手動重新執行 `./run.sh`。不要改用 `nohup`，先前這種做法曾讓
服務在連線結束後停止並使 Cloudflare 回傳 `521`。

確認程序與 listener：

```sh
ps -o uid=,user=,pid=,ppid=,args= -C caddy -C website
ss -lntup
```

預期 Caddy 和 `website` 都屬於 `hellhbbd`，網站只監聽 `127.0.0.1:8080`。

## 8. 更新與回復

### 正常更新

1. 在建置機執行測試與 `./scripts/deploy.sh`。
2. 在部署機的 `run.sh` 終端按 Ctrl+C，確認舊服務停止。
3. 在部署機本機驗證新設定：

```sh
cd /home/hellhbbd/website
set -a
. ./.env
set +a
caddy validate --config Caddyfile
```

4. 驗證通過後以 `hellhbbd` 執行 `./run.sh`。
5. 執行第 9 節的外部健康檢查。

同步前由操作者保存上一版 `website`、`public/` 和 `Caddyfile`，才能手動
回復。不要回復或覆蓋 `.env`、`tasks.json`、`tasks.json.tmp`。

### 失敗回復

若新設定無法啟動：

1. 在 `run.sh` 終端按 Ctrl+C。
2. 將上一版 `website`、`public/` 和 `Caddyfile` 恢復到部署目錄。
3. 載入 `.env` 並執行 `caddy validate --config Caddyfile`。
4. 以 `./run.sh` 啟動上一版。
5. 驗證公開網址恢復，再調查建置或 Caddyfile 差異。

## 9. 驗證清單

從建置機或另一台 LAN 裝置執行。不要提供正式 Basic Auth 密碼給自動化或
協作者。

```sh
curl --head https://website.hellhbbd.pp.ua/
curl --head https://website.hellhbbd.pp.ua/schedule
curl --request POST --head https://website.hellhbbd.pp.ua/api/download
```

預期首頁是 `200`；`/schedule` 與匿名 API 是 `401` 且包含
`Cache-Control: no-store`。

檢查敏感路徑：

```sh
for path in /tasks.json /tasks.json.tmp /tasks.json.bak /.env /.git/config; do
  curl --path-as-is --output /dev/null --write-out "$path %{http_code}\n" \
    "https://website.hellhbbd.pp.ua$path"
done
```

每一條必須是 `404`。

確認 LAN SSH 和來源隔離：

```sh
ssh -o ConnectTimeout=5 home true
curl --connect-timeout 5 --resolve website.hellhbbd.pp.ua:443:192.168.0.114 \
  --head https://website.hellhbbd.pp.ua/
curl --connect-timeout 5 http://192.168.0.114:8080/
```

SSH 必須成功；最後兩個來源直接連線必須 timeout 或被拒絕。

部署機本機驗證 origin 健康與 TLS：

```sh
curl --resolve website.hellhbbd.pp.ua:443:127.0.0.1 \
  --head https://website.hellhbbd.pp.ua/
openssl s_client -connect 127.0.0.1:443 \
  -servername website.hellhbbd.pp.ua -verify_return_error -brief </dev/null
```

最後確認防火牆：

```sh
sudo ufw status verbose
systemctl is-enabled ufw fail2ban
systemctl is-active ufw fail2ban
sudo fail2ban-client status sshd
```

## 10. 故障排除與定期維護

| 症狀 | 檢查與處理 |
| --- | --- |
| Cloudflare `521` | 確認 `run.sh` 仍在執行、Caddy 監聽 80/443、router forwarding 和 UFW Cloudflare ranges；必要時在本機 `sudo ufw disable` 回復。 |
| Caddy 無法綁定 80/443 | 執行 `getcap /usr/bin/caddy`；缺少 capability 時重新執行第 5.1 節的 `setcap`。 |
| `caddy reload` 失敗 | 這是預期行為，設定 `admin off`；停止並重啟 `run.sh`。 |
| Basic Auth 變數錯誤 | 檢查 `.env` 存在、權限 0600、變數名稱正確，並以 `set -a; . ./.env; set +a` 驗證 Caddyfile。 |
| `dx` 與 crates 不相容 | 不部署產物；先將 Dioxus CLI與 `Cargo.toml` 的 Dioxus版本對齊並完成 release bundle測試。 |
| 8080 已被使用 | 使用 `lsof -Pi :8080 -sTCP:LISTEN` 找出舊網站程序，正常停止原本的 `run.sh`，不要以 root 啟動第二個實例。 |
| xtables lock | 停止 Fail2ban，等待 iptables 程序結束；不要刪除 `/run/xtables.lock`。 |
| 重開機後停站 | 目前預期行為，登入 `hellhbbd` 後執行 `cd ~/website && ./run.sh`；日後遷移到 systemd。 |

至少每月執行：

1. 在套用前重新取得並比較 Cloudflare IPv4/IPv6 ranges。
2. 更新 Caddy 後檢查 `getcap /usr/bin/caddy`。
3. 檢查 `sudo fail2ban-client status sshd` 和 Fail2ban CPU 使用量。
4. 執行 Rust dependency advisory scan，以及 `./scripts/test.sh`。
5. 從 LAN 驗證 SSH、公開網址與來源 IP直連封鎖。
6. 規劃 systemd 服務隔離、任務資料備份與非 LAN SSH overlay network。
