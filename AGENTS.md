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
- **Disco local:** nunca deixe `target/` para trás — `cargo clean` após uso local;
  builds pesados são do CI.

## Estrutura
- `crates/` — engine e módulos (`localpaint-*`), em camadas L0–L6 (`cargo xtask layers`)
- `apps/localpaint` — app desktop + control channel; `apps/localpaint-cli` — CLI/MCP;
  `apps/localpaint-web` e `apps/localpaint-web-boot` — wasm
- `libs/` — forks vendored: `craft-libs`, `egui`, `winit`, `winprint.rs`
- `packaging/` — Windows (WiX/MSI), Linux (AppImage/deb/rpm/tar/Flatpak), Arch (PKGBUILD), macOS, FreeBSD, web
- `book/` — o livro de documentação (mdBook); `docs/` — arquitetura, formatos, paridade, release
- `xtask/` — tarefas de build/CI (`cargo xtask ci`, `layers`, `wasm`, `parity`, `scorecard`)

## Build
```
cargo build --locked          # debug
cargo test --locked           # testes
packaging/windows/package.ps1 # MSI + portable (Windows)
packaging/linux/package.sh    # AppImage/deb/rpm/tar (Linux)
```

Regras técnicas do código (never-crash, camadas, tudo é comando, wasm, testes):
`docs/contributing.md`.


## Upstream e a pasta `C:\Dev\storytold`

Menções a `storytold/*` em código, comentários, docs e atribuições (NOTICE/ATTRIBUTION) são referência histórica e obrigação de licença. **Nada** é lido de `C:\Dev\storytold` em build, teste ou release: tudo vem do próprio repo (crates.io pinado pelo lock + `libs/`/`vendor/` vendored + CI que busca craft-fonts/craft-i18n do GitHub). Essa pasta existe somente como referência local para a skill `sync-upstream`.
