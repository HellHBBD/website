# website

## Production security setup

The schedule and API endpoints require Caddy Basic Auth. Before starting the
deployed service, create a non-versioned `.env` beside `run.sh`:

```sh
CADDY_BASIC_AUTH_USER=your-user-name
CADDY_BASIC_AUTH_HASH='$2a$14$replace-this-with-caddy-hash-password-output'
```

Generate the password hash on the deployment host. Do not store the password
or its hash in Git:

```sh
caddy hash-password --algorithm bcrypt --plaintext 'a-long-random-password'
```

Export these variables before invoking `run.sh`, for example:

```sh
set -a
. ./.env
set +a
./run.sh
```

After deploying an updated `Caddyfile`, load the same environment before
validating and reloading Caddy:

```sh
set -a
. ./.env
set +a
caddy validate --config Caddyfile
caddy reload --config Caddyfile
```

The public DNS record for `website.hellhbbd.pp.ua` must resolve to the
deployment host before Caddy can obtain a valid TLS certificate. Verify this
before deployment:

```sh
curl --fail --head https://website.hellhbbd.pp.ua/
```

`run.sh` binds the Dioxus server to `127.0.0.1:8080`; keep the host firewall
configured so port 8080 is not reachable from external networks.

## Cloudflare rules

Use `Full (strict)` SSL/TLS mode. Add cache-bypass rules for `/schedule*` and
`/api/*`; Caddy also sends `Cache-Control: no-store` for those responses.

Add a rate-limiting rule for `/schedule*` and `/api/*` that issues a Managed
Challenge or temporary block after repeated requests from one IP. Keep Browser
Insights disabled because its injected script is intentionally blocked by the
site CSP.
