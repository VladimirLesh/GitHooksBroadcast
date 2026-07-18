# GitHookBroadcast

Небольшой HTTP-сервер на Rust, который принимает вебхуки от **GitHub / GitLab / Gitea**
(события push и pull request / merge request), проверяет подпись **HMAC-SHA256**
в constant-time и рассылает форматированное уведомление в мессенджеры:

- Telegram
- Discord
- Element (Matrix)
- Mattermost
- Pachca

Один статически слинкованный бинарник, без рантайм-зависимостей. Собирается через CI
под шесть таргетов: `x86_64`, `aarch64`, `riscv64gc` × `musl` (полностью статик) и `gnu`.

## Запуск

```bash
cp config.example.toml config.local.toml
# отредактируйте секции [[routes]] и [sinks.*], задайте переменные окружения
export HOOK_SECRET_MAIN=$(openssl rand -hex 32)
export TG_BOT_TOKEN=...
cargo run --release -- config.local.toml
```

Формат подписи по источникам:

| Источник | Заголовок | Схема |
|----------|-----------|-------|
| GitHub | `X-Hub-Signature-256` | HMAC-SHA256 hex, префикс `sha256=` |
| Gitea  | `X-Gitea-Signature` | HMAC-SHA256 hex |
| GitLab | `X-Gitlab-Token` | plain shared secret (constant-time compare) |

## Эндпоинты

- `POST /hook/{route_id}` — приёмник вебхука. Успешно принятое событие → `202 Accepted`,
  нерелевантное (issues, ping и т.п.) → `204 No Content`.
- `GET /healthz` — `200 ok`.

## Сборка под целевые архитектуры

Локально (нужен [`cross`](https://github.com/cross-rs/cross) и Docker):

```bash
cross build --release --target riscv64gc-unknown-linux-musl
cross build --release --target aarch64-unknown-linux-musl
cross build --release --target x86_64-unknown-linux-musl
```

Через CI — см. `.github/workflows/ci.yml`. Для musl-таргетов CI проверяет
`file` вывод на `statically linked`.

## Тесты

```bash
cargo test
```

## Лицензия

MIT OR Apache-2.0
