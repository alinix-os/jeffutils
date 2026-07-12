# jsh

Shell interativo `jsh` do ecossistema JeffNix/Alinix.

## Recursos

- Prompt dinâmico com ícone por SO (🐧/🍎/🪟), sobrescrevível via `PROMPT_ICON`.
- Neofetch (`jeofetch`) integrado.
- Carregamento de `.jshrc` com aliases, exports, variáveis e funções.
- Sintaxe de shell: aspas simples/duplas, escapes (`\`), pipes (`|`),
  redirecionamentos (`>`, `>>`, `<`, `<<`, `<<<`, `2>`, `&>`), listas de
  comandos (`;`, `&&`, `||`), background (`&`).
- Variáveis de shell (`NAME=valor`), `export`/`unset`/`set`, variáveis
  especiais `$?`, `$$`, `$0`, `$PWD`, `$OLDPWD`.
- Substituição de comando `$(...)` / `` `...` ``.
- Globbing (`*`, `?`, `[...]`).
- Funções de shell simples: `nome() { corpo }`, com `$1`, `$2`, `$@`, `$#`
  e expansão de parâmetro `${N:+palavra}` / `${N:-palavra}`.
- Histórico estilo bash: `!!`, `!n`, `!prefixo`.
- Modo não interativo: `jsh script.jsh` ou `comando | jsh` executa um
  script sem precisar de TTY.
- `source`/`.` para rodar outro script no shell atual; scripts bash
  "de verdade" (funções complexas, `[[`, etc.) sourceados dessa forma são
  automaticamente repassados para `bash` quando um comando neles é chamado
  (ex.: `.jshrc` sourceando `nvm.sh` e depois usando `nvm`).

## Compilação

```bash
cargo build --release
```

## Fonte JSH Mono

`jsh` vem com uma fonte própria, **JSH Mono** (`assets/font/`) — um fork da
[JetBrains Mono](https://www.jetbrains.com/lp/mono/) onde os emojis 🐧
(`U+1F427`), 🍎 (`U+1F34E`) e 🪟 (`U+1FA9F`) foram substituídos por versões
minimalistas dos logos oficiais de Linux, macOS e Windows — os mesmos
usados no prompt (`os_logo()` em `src/shell/mod.rs`). Nenhum outro glifo é
tocado, incluindo a Private Use Area (reservada para uso futuro).

- `assets/font/*.ttf` — Regular, Bold, Italic, BoldItalic, prontos para uso.
- `assets/font/src-svg/` — os 3 SVGs originais dos logos.
- `assets/font/merge_font_colr.py` — script que funde os SVGs (compilados
  antes para COLRv1 via [nanoemoji](https://github.com/googlefonts/nanoemoji))
  em cada peso da JetBrains Mono. Só precisa rodar de novo se os logos
  forem redesenhados; veja o cabeçalho do script para o pipeline completo.

Para instalar manualmente (sem o pacote do monorepo — veja
`../packaging/`):

```bash
mkdir -p ~/.local/share/fonts
cp assets/font/*.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts
```

Depois, configure seu terminal para usar a fonte **JSH Mono**.
