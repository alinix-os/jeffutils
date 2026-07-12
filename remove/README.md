# remove

Utilitário `remove` para o ecossistema JeffNix.

Este é um utilitário escrito em Rust para o sistema operacional **JeffNix**.

## Uso

```bash
./remove <destino> [-f|-d] [-r] [--force]
```

### Opções

| Opção | Descrição |
| --- | --- |
| `--file`, `-f` | Remove um arquivo (padrão) |
| `--dir`, `-d` | Remove um diretório |
| `--recursive`, `-r` | Remove diretórios e seu conteúdo recursivamente |
| `--force` | Pula a confirmação simples (não pula as proteções críticas) |
| `--help`, `-h` | Mostra a ajuda |

## Segurança

O `remove` foi projetado para evitar destruições acidentais:

- **Caminhos protegidos** — alvos como `/`, `/etc`, `/home`, `/usr`, `/var`
  (e outros diretórios de sistema) são **recusados** sob qualquer flag.
- **Globs não expandidos** — se um alvo chegar contendo `*`, `?` ou `[`
  (por exemplo `rm * -rf` com aspas), a operação é **recusada**.
- **Resolução de caminho** — sequências como `../../..` são normalizadas
  antes da verificação, então não é possível "escalar" até a raiz.
- **Confirmação dupla** — alvos extremamente perigosos (o diretório home do
  usuário, ou caminhos rasos logo abaixo da raiz) exigem que você digite o
  caminho completo exatamente, **mesmo com `--force`**.
- Alvos comuns pedem apenas a confirmação simples `[y/N]` (a menos que
  `--force` seja usado).

## Compilação

Para compilar este utilitário individualmente:

```bash
cargo build --release
```
