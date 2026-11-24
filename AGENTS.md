# AGENTS.md — Local Raster

Fork independente do photocraft (ArtCraft), rebrandado para a suíte Local do João.
Rust puro + egui. Sem conta, sem telemetria, sem dependência de servidores de terceiros.

## Regras
- **Dependências:** toda dep que entra/sai/atualiza exige `Cargo.lock` commitado +
  linha no inventário do repo `Anon5T4R/local-deps` (regra completa em
  `local-deps/WORKFLOW.md`). PR sem isso não mergea.
- **Sem fontes git:** nenhuma dependência pode apontar para `git+` (o CI valida).
  Tudo vem do crates.io (pinado pelo lock) ou de `libs/` (vendored).
- **Commits:** português, sem Co-Authored-By.
- **Release:** tag `vX.Y.Z` → o Actions builda Windows + Linux e publica. Mínimo local.
- **Disco:** no workspace do João vale a política de `target/` do dev-notes (`MEMORY.md` §4 — limpa o que
  está frio ou feito, mantém durante iteração ativa); fora dele, `cargo clean` ao terminar. Builds pesados são do CI.

## Estrutura
- `crates/` — engine e módulos (`localraster-*`), em camadas L0–L6 (`cargo xtask layers`)
- `apps/localraster` — app desktop + control channel; `apps/localraster-cli` — CLI/MCP
- `libs/` — forks vendored: `craft-libs`, `egui`, `winit`, `winprint.rs`
- `packaging/` — Windows (WiX/MSI), Linux (AppImage/deb/rpm/tar FHS com `install.sh`), Arch (PKGBUILD), macOS, FreeBSD
- `book/` — o livro de documentação (mdBook); `docs/` — arquitetura, formatos, release e notas de design (scorecard e checklist de menu são gerados pelo xtask)
- `scripts/check-mirror.sh` — confere as crates e os assets (`craft-fonts`) espelhados do `local-libs` contra a fonte (CI e `xtask ci`)
- `xtask/` — tarefas de build/CI (`cargo xtask ci`, `layers`, `parity`, `scorecard`)

## Renome (Local Paint → Local Raster)
O app se chamava Local Paint (marca colidia com o Microsoft Paint). A identidade nova (`localraster`) vale em
tudo; o nome velho só sobrevive onde quebraria instalação ou arquivo existente:
- **Pastas do usuário**: `apps/localraster/src/legacy_dirs.rs` move `LocalPaint`/`localpaint` (config, estado, recovery,
  `LocalPaintData` do portátil) para o nome novo na primeira abertura, uma vez, sem sobrescrever.
- **Variáveis de ambiente**: `LOCALPAINT_*` ainda vale quando a `LOCALRASTER_*` não está definida
  (`crates/engine/src/legacy_env.rs`; clap passa por `blank_env_is_unset`). Variáveis só de teste, bench e
  build de CI não têm fallback.
- **Atualizador**: `suite::LOCALRASTER` carrega os ids/nomes/repo antigos (`former_*`); `LOCALPAINT` é alias obsoleto.
- **Windows**: UpgradeCode e GUIDs de componente do MSI ficam; o MSI novo faz major upgrade do antigo.
- **Linux**: deb/rpm/Arch declaram `replaces`/`conflicts`/`provides` de `localpaint` (e `com.localpaint.app`).
- **Dados em disco que ficam com o nome velho** (formato/interoperabilidade, não marca): chave `localpaintDroplet`
  dos `.pcdroplet`, `localpaint-presets` e `localpaint-brush-group` dos presets, valor `localpaintManages` de
  `colorHandling` (o engine aceita também `localrasterManages`), as chaves `Id::unique("localpaint-…")` do egui
  (o estado de painéis/docking persistido em `ui.ron` é indexado por elas) e o URI MCP `localpaint://` (aceito como alias).

## Agent-native (MCP)
O eixo mais completo da suíte: `crates/automation` (MCP headless + bridge com o token do control
channel, workspace de capabilities), `localraster-cli mcp` (as roots `--automation-read-root` /
`--automation-write-root` **negam por padrão**: sem root, aquela direção de acesso não existe),
`serve` (JSON lines em stdio/TCP) e `run --script` (array JSON ou JSON Lines; passo é id,
`[id, params]` ou `{"command", "params"}`). A crate compartilhada **`craft-mcp`**
(`libs/craft-libs/crates/mcp`, fonte `local-libs`) entra no `mcp --connect` (alias de `--bridge`;
porta nua = loopback; host público recusado). Docs: `book/src/automation/{cli,mcp,control-protocol}.md`.

## Build
```
cargo build --locked          # debug
cargo test --locked           # testes (as fontes vendored entram sempre: .cargo/config.toml, igual ao CI)
cargo xtask ci                # o gate local: fmt, clippy, camadas, scorecard, testes, cargo-deny e espelhos do local-libs
packaging/windows/package.ps1 # MSI + portable (Windows)
packaging/linux/package.sh    # AppImage/deb/rpm/tar (Linux)
```

Regras técnicas do código (never-crash, camadas, tudo é comando, testes):
`docs/contributing.md`.


## Upstream e a pasta `C:\Dev\storytold`

Menções a `storytold/*` em código, comentários, docs e atribuições (NOTICE/ATTRIBUTION) são referência histórica e obrigação de licença. **Nada** é lido de `C:\Dev\storytold` em build, teste ou release: tudo vem do próprio repo (crates.io pinado pelo lock + `libs/`/`vendor/` vendored + CI que busca craft-fonts/craft-i18n do GitHub). Essa pasta existe somente como referência local para a skill `sync-upstream`.
