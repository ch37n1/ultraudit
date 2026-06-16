# План: prompt/practice pack v0.2

Этот документ фиксирует план отдельного прохода по переработке prompts/practices. Цель - уйти от генерации pack-а из Rust и сырой `.local/research`, сделать pack обычным версионированным Git-артефактом и собирать для каждого agent task полный self-contained prompt по конкретной линзе/оптике.

## Проблема текущей реализации

Сейчас `uat init` смешивает несколько ответственностей:

- создает проектный `.audit/`;
- создает пользовательский `~/.ultraudit`;
- генерирует default pack из Rust-кода;
- местами копирует материалы из `.local/research`.

В результате prompt/practice pack не является надежным source-of-truth в Git. Он получается как смесь fallback-строк в `src/pack.rs`, локальных research-файлов и установленной копии в home directory.

Ключевые текущие места:

- `src/pack.rs` - `seed_default_pack`, `pack_root`, генерация prompt/practice pack;
- `src/orchestrator.rs` - `run_lens_review`, `run_optic_review`, `run_cross_domain_review`, synthesis-проходы;
- `src/cli.rs` - есть `run` и `init`; отдельный `ultraudit install` добавлять не нужно, потому что утилита не должна устанавливать сама себя.

## Целевая модель

Research остается source material и не используется агентом напрямую. Агент должен получать curated prompt/practice assets, подготовленные из research заранее.

Source-of-truth для pack-а должен жить в репозитории:

```text
packs/
  0.2.0/
    pack.toml
    prompts/
      base-reviewer.md
      domain-discovery.md
      domain-lens-review.md
      project-optic-review.md
      cross-system-review.md
      domain-synthesis.md
      system-synthesis.md
      previous-runs-comparison.md
      final-editor.md
    lenses/
      architecture/
        prompt.md
        practices.md
        evidence.md
        false-positives.md
      code-quality/
      security/
      correctness/
      testing/
      reliability/
      performance/
      observability/
      operations/
      api-contracts/
      data-integrity/
      privacy-compliance/
      dependency-supply-chain/
      ux-product/
      ml-ai/
    optics/
      documentation-knowledge/
        prompt.md
        practices.md
        evidence.md
        false-positives.md
      nice-practices/
    integration/
      evidence-model.md
      severity-model.md
      confidence-model.md
      deduplication-rules.md
      final-editor-checklist.md
```

Installed copy в пользовательской системе:

```text
~/.ultraudit/
  config.toml
  packs/
    0.2.0/
      ...
```

Лишний уровень `packs/ultraudit-default/versions/0.1.0` нужно убрать для новой версии. Пока у нас один основной pack, `ultraudit-default` не дает полезной информации. Старый layout можно оставить только как backward compatibility.

## Правило сборки prompt-а

Каждая agent task получает полный self-contained prompt по своей задаче, линзе/оптике и домену.

Например задача:

```text
review domain users through code-quality
```

Получает:

```text
base reviewer contract
+ full code-quality prompt
+ full code-quality practices
+ code-quality evidence requirements
+ code-quality false-positive checks
+ severity/confidence model
+ task description
+ domain context
+ project/domain map
+ output schema and paths
```

Это не глобальная knowledge base, которую агент должен сам исследовать. Это и не короткий summary. Это полный task prompt, детерминированно собранный из версионированных блоков pack-а.

Сырые research-артефакты, source maps, coverage matrices и research gaps могут храниться рядом для сопровождения pack-а, но не должны автоматически попадать в runtime prompt, если они не превращены в agent-facing guidance.

## Разделение review flows

Нужно явно разделить типы проходов.

### 1. Domain lens review

Основной проход: один домен или субдомен плюс одна линза.

Пример:

```text
domain: users
lens: code-quality
```

Агент получает все практики и правила проверки по `code-quality`, карту проекта, описание домена и список файлов, которые нужно смотреть первыми.

Кандидаты для domain-level линз:

```text
architecture
code-quality
security
correctness
testing
reliability
performance
observability
operations
api-contracts
data-integrity
privacy-compliance
dependency-supply-chain
ux-product
ml-ai
```

### 2. Project optic review

Некоторые оптики не являются естественно доменными. Например `documentation-knowledge` лучше проверять на уровне проекта или системы, а не запускать отдельно по каждому домену.

Кандидаты:

```text
documentation-knowledge
nice-practices
```

Для этих оптик нужен отдельный prompt template вида `project-optic-review.md`.

### 3. Cross-system review

После domain-level проверок нужен проход второго порядка по всей системе. Здесь не нужны все линзы.

По умолчанию для cross-system review можно использовать:

```text
architecture
security
reliability
operations
api-contracts
data-integrity
privacy-compliance
ml-ai
```

По умолчанию исключить:

```text
code-quality
testing
documentation-knowledge
nice-practices
```

Причина: code-quality и testing обычно требуют просмотра конкретных файлов и локальных практик. Documentation является project-level optic, а не cross-system risk lens.

### 4. Synthesis и final editor

Synthesis-проходы не должны получать все lens practices. Им нужны:

```text
evidence model
severity/confidence model
deduplication rules
report contract
previous-run comparison rules
final editor checklist
```

Их задача - объединять, дедуплицировать, калибровать и редактировать findings, а не заново искать проблемы по линзам.

## Pack manifest

В `packs/0.2.0/pack.toml` нужно выразить flow policy.

Пример:

```toml
schema_version = "2"
version = "0.2.0"

[sets]
default = ["architecture", "code-quality", "security", "correctness", "testing"]
production = ["reliability", "performance", "observability", "operations"]
contracts-and-data = ["api-contracts", "data-integrity", "privacy-compliance", "dependency-supply-chain"]
product = ["ux-product", "ml-ai"]
full = ["architecture", "code-quality", "security", "correctness", "testing", "reliability", "performance", "observability", "operations", "api-contracts", "data-integrity", "privacy-compliance", "dependency-supply-chain", "ux-product", "ml-ai"]

[flows.domain_review]
lenses = ["architecture", "code-quality", "security", "correctness", "testing"]

[flows.project_review]
optics = ["documentation-knowledge", "nice-practices"]

[flows.cross_system_review]
lenses = ["architecture", "security", "reliability", "operations", "api-contracts", "data-integrity", "privacy-compliance", "ml-ai"]

[flows.synthesis]
integration = ["evidence-model", "severity-model", "confidence-model", "deduplication-rules"]
```

Позже можно добавить CLI flags вроде `--flow domain`, `--flow project`, `--flow cross-system`. Для первого прохода достаточно использовать manifest как внутреннюю policy.

## Installation model

Установка должна идти снаружи утилиты, через Makefile:

```bash
git clone <repo-url>
cd ultraudit
make install
```

`ultraudit` не должен иметь команду `install`: консольная утилита не должна устанавливать сама себя. Она должна только использовать уже установленный prompt/practice pack.

`make install` должен:

- собрать release binary;
- установить binary в user-level bin directory;
- создать `~/.ultraudit`;
- скопировать Git-tracked pack-и из репозитория;
- проверить, что `codex` доступен в `PATH`, и вывести понятную ошибку или warning, если он недоступен;
- не зависеть от `.local/research`.

Копирование pack-а:

```text
repo/packs/0.2.0
```

в:

```text
~/.ultraudit/packs/0.2.0
```

Путь установки binary нужно выбрать явно. Предпочтительный вариант:

```text
~/.local/bin/uat
```

Если `~/.local/bin` отсутствует в `PATH`, `make install` должен вывести post-install инструкцию.

`init` должен заниматься только проектной конфигурацией:

```text
.audit/config.toml
.audit/agents/codex.toml
.audit/agents/custom-shell.toml
```

Пример `.audit/config.toml` для новой версии:

```toml
[prompt_pack]
version = "0.2.0"
source = "~/.ultraudit/packs/0.2.0"

[run]
agent = "codex"
output_dir = ".audit-runs"
disabled_optics = []
```

`name = "ultraudit-default"` убрать из новой конфигурации или оставить как deprecated compatibility field.

## Implementation plan

1. Создать `packs/0.2.0` в репозитории.
2. Перенести туда curated prompt/practice assets из `.local/research`, но не raw research.
3. Добавить `Makefile` с целью `install`.
4. В `make install`:
   - выполнить `cargo build --release`;
   - установить `target/release/uat` в `~/.local/bin/uat` или другой выбранный user bin;
   - создать `~/.ultraudit/packs`;
   - скопировать `packs/0.2.0` в `~/.ultraudit/packs/0.2.0`;
   - проверить `command -v codex`;
   - вывести post-install summary.
5. Переписать `src/pack.rs`:
   - убрать генерацию основного pack из Rust;
   - изменить `pack_root` на `~/.ultraudit/packs/{version}`;
   - оставить старый layout только как fallback.
6. Обновить `init_project`:
   - не seed-ить pack;
   - писать `.audit/config.toml` на новую структуру.
7. Обновить `resolve_pack`:
   - сначала читать explicit `source`;
   - потом искать `~/.ultraudit/packs/{version}`;
   - если pack missing, возвращать понятную ошибку: run `make install` from the Ultraudit repository.
8. Обновить runtime prompt assembly:
   - `run_lens_review` собирает полный prompt из lens files;
   - `run_optic_review` использовать для project-level optics или разделить на новый flow;
   - `cross-domain`, `system-synthesis`, `final-editor` используют только integration prompts.
9. Обновить selection policy:
   - domain-level линзы отдельно;
   - project-level optics отдельно;
   - cross-system линзы отдельно.
10. Обновить snapshot behavior:
   - каждый run копирует resolved installed pack в `run_dir/prompt-pack`;
   - compiled prompts остаются в `raw/*/prompt.md`.
11. Обновить README:
   - `make install`
   - `uat init`
   - `uat run --pack default`

## Tests

Добавить или обновить тесты:

- `make install` собирает release binary;
- `make install` устанавливает binary в user-level bin directory;
- `make install` копирует `packs/0.2.0` в выбранный `ULTRAUDIT_PATH` или `~/.ultraudit`;
- `make install` проверяет доступность `codex`;
- installed files совпадают с checked-in pack assets;
- `uat init` не требует `.local/research`;
- `uat run --plan` или `run --dry-run` резолвит `~/.ultraudit/packs/0.2.0`;
- compiled prompt для domain-lens task содержит полный guide выбранной линзы;
- `documentation-knowledge` не запускается per-domain по умолчанию;
- cross-system review не запускает `code-quality` по умолчанию;
- fresh checkout без `.local/research` проходит install/init/plan.

## Acceptance criteria

Свежий checkout без `.local/research` должен поддерживать:

```bash
make install
uat init
uat run --plan
```

После `make install` команда `uat` должна быть доступна из shell, а `~/.ultraudit/packs/0.2.0` должен содержать копию Git-tracked pack-а.

Runtime prompt для domain-lens task должен быть self-contained и включать полный curated guide выбранной линзы/оптики, task context, domain/project map и output contract.

Installed pack должен быть копией Git-tracked `packs/0.2.0`, а не результатом генерации из Rust и не копией raw research.
