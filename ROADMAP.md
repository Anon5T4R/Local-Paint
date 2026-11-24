# Local Raster roadmap

**Stage: alpha** · os seis fluxos centrais do Photoshop (retocar, compor, recortar, desenhar,
pintar, trocar PSD) funcionam ponta a ponta e salvam/reencontram o arquivo; quatro são parciais
em profundidade, nenhum bloqueado.

Fork independente do photocraft (ArtCraft) na suíte Local do João. A régua é o Adobe Photoshop
2026, medida pelos artefatos gerados `docs/scorecard.md` (`cargo xtask scorecard`) e
`docs/parity-checklist.md` (`cargo xtask parity`); a arquitetura está em `docs/architecture.md`.
Histórico e datas: git e release notes.

## Números (medição de 2026-10-10, versão 0.6.0 do fork)

| | Valor |
|---|---|
| Amplitude de features | ~76% (menus 628/628, ferramentas 56/68, painéis 30/35, formatos 13+4 de 31) |
| Pronto para trabalho real | ~45% (40–50%) |
| Practitioner essencial | ~61% |
| Budgets de performance | 3 de 25 (`docs/scorecard.md`, medido) |

Beta pede ~75% pronto e nenhum buraco de PSD bloqueante.

## Fila (o que falta, em ordem)

1. **PSD que o Photoshop confie:** Save As sem corrupção e reabertura no Photoshop; oracle floor
   145 → 200+ de 258 (#1281, #2469).
2. **Performance:** os 22 cenários fora do budget e o crash de GPU com 150 camadas
   (`docs/scorecard.md`).
3. **Backlog de bugs:** 953 issues abertas (2026-10-13) — as do fork continuam valendo.
4. **Profundidade de pincel e transformação** (o "feel" das ferramentas).
5. **As 12 ferramentas que faltam** na caixa (56 de 68).
6. **Formatos pendentes:** JP2, DICOM, DPX, C2PA.
7. **Pro parity:** print, HDR e timeline; CMYK/Lab/Indexed/Bitmap/Duotone, soft proofing,
   Quick Mask, stack modes de smart object, artboards e layer comps são parciais.
8. **Caneta nativa no Linux Wayland** (tablet-v2; hoje via Xwayland, #79). Windows (WM_POINTER),
   macOS (AppKit) e X11 (XInput2) estão feitos.
9. **ML:** backend não começado (Select Subject aprendido, recursos generativos); Photomerge
   implementado, validação de qualidade no mundo real aberta.
10. **Interface RTL:** painéis, diálogos, abas e canvas seguem LTR por ora; só o chrome da janela
    espelha (crates/ui-egui/src/text/direction.rs).

## Idiomas

Interface: 25 catálogos em `crates/ui-egui/src/i18n/` (pt-br, zh-hans/zh-hant, es, fr, de, it, ja,
ko, ar, ru, uk, cs, pl, el, nl, da, fi, nb, sv, tr, hu, id, vi, sr…). Mensagens da engine ainda
em inglês.

## Suíte

Desde 2026-11 o app segue o padrão craft (guardas do kittest, diálogos na largura mínima,
contraste AA no accent, craft-palette/update/icons); o que muda na casca vem do
`C:\Dev\docs\planos\padrao-craft.md` (repo dev-notes).
