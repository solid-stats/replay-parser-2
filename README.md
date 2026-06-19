# replay-parser-2

**Русский** · [English](README.en.md)

Rust-парсер OCAP-реплеев для **Solid Stats** — статистики игр сообщества
[Solid Games](https://sg.zone) (ArmA 3). Превращает OCAP JSON-реплеи в компактные,
детерминированные, версионируемые артефакты, которые `server-2` хранит, аудирует на
уровне вклада в статистику и использует для публичных показателей. Замена legacy-парсера
на&nbsp;Rust.

Главный принцип v1 — артефакт по&nbsp;умолчанию **сокращает** данные реплея: OCAP-файл
на&nbsp;10–15&nbsp;МБ не&nbsp;должен превращаться в&nbsp;ещё один JSON на&nbsp;10–15&nbsp;МБ
на&nbsp;обычном пути загрузки.

Часть многорепной платформы: поиск сырых реплеев — в&nbsp;`replays-fetcher`, хранение
бизнес-состояния, API и&nbsp;оркестрация задач — в&nbsp;`server-2`, веб-интерфейс —
в&nbsp;`web`, рантайм и&nbsp;операции — в&nbsp;`infrastructure`. replay-parser-2 владеет
только разбором и&nbsp;контрактом выходного артефакта; каноническая идентичность игроков
остаётся за&nbsp;`server-2`.

> Solid Stats от&nbsp;и&nbsp;до строят AI-агенты по&nbsp;процессу
> [GSD](https://github.com/open-gsd/gsd-core). Разработка вне&nbsp;GSD — вне&nbsp;процесса.

## Статус

Веха v1.0 завершена и&nbsp;заархивирована: контракт артефакта `3.0.0`, режим CLI и&nbsp;воркер
RabbitMQ/S3 поставлены, строгие гейты качества на&nbsp;месте. Текущий фокус — ожидание
определения следующей вехи.

## Быстрый старт

Сборка и&nbsp;разбор одного реплея в&nbsp;минимальный JSON:

```bash
cargo build --release
replay-parser-2 parse path/to/replay.json --output path/to/artifact.json
```

Запуск воркера (читает задачи RabbitMQ, сырые объекты S3, пишет артефакты и&nbsp;публикует
`parse.completed` / `parse.failed`):

```bash
replay-parser-2 worker
```

Гейт качества воркспейса:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Полный список команд, опций воркера, деплоя и&nbsp;гейтов покрытия —
в&nbsp;[docs/parser-reference.md](docs/parser-reference.md).

## Документация

- [docs/parser-reference.md](docs/parser-reference.md) — контракт артефакта, CLI и&nbsp;воркер,
  деплой, гейты качества, данные валидации, история приёмки v1.0.
- `.planning/` — продуктовый контекст, требования, роадмап и&nbsp;состояние (GSD).
- `gsd-briefs/` — брифы проектов `replays-fetcher`, `replay-parser-2`, `server-2`, `web`.

## Стек

Rust 2024 (1.95) · Cargo workspace · serde / serde_json · schemars · tokio · lapin (RabbitMQ)
· aws-sdk-s3 · tracing

## Лицензия — MIT
