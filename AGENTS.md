# AGENTS.md — Local Paint

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
- `crates/` — engine e módulos (`localpaint-*`), em camadas L0–L6 (`cargo xtask layers`)
- `apps/localpaint` — app desktop + control channel; `apps/localpaint-cli` — CLI/MCP;
  `apps/localpaint-web` e `apps/localpaint-web-boot` — wasm
- `libs/` — forks vendored: `craft-libs`, `egui`, `winit`, `winprint.rs`
- `packaging/` — Windows (WiX/MSI), Linux (AppImage/deb/rpm/tar FHS com `install.sh`), Arch (PKGBUILD), macOS, FreeBSD, web
- `book/` — o livro de documentação (mdBook); `docs/` — arquitetura, formatos, paridade, release
- `scripts/check-mirror.sh` — confere as crates e os assets (`craft-fonts`) espelhados do `local-libs` contra a fonte (CI e `xtask ci`)
- `xtask/` — tarefas de build/CI (`cargo xtask ci`, `layers`, `wasm`, `parity`, `scorecard`)

## Agent-native (MCP)
O eixo mais completo da suíte: `crates/automation` (MCP headless + bridge com o token do control
channel, workspace de capabilities), `localpaint-cli mcp` (as roots `--automation-read-root` /
`--automation-write-root` **negam por padrão**: sem root, aquela direção de acesso não existe),
`serve` (JSON lines em stdio/TCP) e `run --script` (array JSON ou JSON Lines; passo é id,
`[id, params]` ou `{"command", "params"}`). A crate compartilhada **`craft-mcp`**
(`libs/craft-libs/crates/mcp`, fonte `local-libs`) entra no `mcp --connect` (alias de `--bridge`;
porta nua = loopback; host público recusado). Docs: `book/src/automation/{cli,mcp,control-protocol}.md`.

## Build
```
cargo build --locked          # debug
cargo test --locked           # testes (as fontes vendored entram sempre: .cargo/config.toml, igual ao CI)
cargo xtask ci                # o gate local: fmt, clippy, camadas, scorecard, testes, wasm, cargo-deny e espelhos do local-libs
packaging/windows/package.ps1 # MSI + portable (Windows)
packaging/linux/package.sh    # AppImage/deb/rpm/tar (Linux)
```

Regras técnicas do código (never-crash, camadas, tudo é comando, wasm, testes):
`docs/contributing.md`.


## Upstream e a pasta `C:\Dev\storytold`

Menções a `storytold/*` em código, comentários, docs e atribuições (NOTICE/ATTRIBUTION) são referência histórica e obrigação de licença. **Nada** é lido de `C:\Dev\storytold` em build, teste ou release: tudo vem do próprio repo (crates.io pinado pelo lock + `libs/`/`vendor/` vendored + CI que busca craft-fonts/craft-i18n do GitHub). Essa pasta existe somente como referência local para a skill `sync-upstream`.
