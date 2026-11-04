<!--
Obrigado por contribuir com o Local Paint.

Baseie o PR em `main`, a única branch de integração. Apague as seções que não se aplicam.
-->

## O que muda

<!-- Qual comportamento é diferente depois, e por quê. Se fecha uma issue, escreva `Fixes #123`. -->

## Como você verificou

<!-- Os comandos que rodou e o que disseram. Por crate é suficiente e preferível:
     `cargo test -p <crate>`. O CI roda o workspace inteiro em todo PR. -->

- [ ] `cargo fmt --check`
- [ ] clippy com `-D warnings` nos crates que mudei
- [ ] testes passam nos crates que mudei
- [ ] `cargo xtask layers` e `cargo xtask wasm` (se mexi em fronteira de crate ou em L0–L6)
- [ ] `cargo xtask parity` e `docs/parity-checklist.md` regerado (se adicionei ou mudei comandos)

**Testes.** Comportamento novo precisa de teste na escala certa — unitário, de integração do crate
ou ponta a ponta. Para correção de bug, **confirme que o teste falha antes da correção**.

## Never-crash

Todo arquivo, comando, chamada MCP, arquivo de preferências e tecla é entrada não confiável
(`AGENTS.md`, `docs/contributing.md`).

- [ ] Nenhum `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!` novo fora de testes
- [ ] Nenhuma indexação ou slice com posição vinda da entrada (`get`, e strings só em fronteira de
      caractere); aritmética checada ou saturante; alocação dimensionada pela entrada é limitada;
      recursão tem limite
- [ ] Nenhum `unsafe` novo
- [ ] Todo crash corrigido tem um teste sintético de regressão

## Proveniência

- [ ] Nenhum asset copiado de produto proprietário (Photoshop e afins); só specs públicas e observação
- [ ] Nenhum arquivo de fonte commitado aqui (eles vivem em
      [storytold/craft-fonts](https://github.com/storytold/craft-fonts))
- [ ] Todo asset novo tem linha em `ATTRIBUTION.md`
- [ ] Nenhuma dependência `git+` (o CI valida)

## Documentação

- [ ] Docs atualizados se o comportamento documentado mudou (`docs/`, `book/`, README do crate)

## O que o revisor deve olhar primeiro

<!-- A decisão que você tem menos certeza, uma restrição deliberada que quer fixada, ou uma
     mudança de comportamento que quem já usa vai notar. -->
