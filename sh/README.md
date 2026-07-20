# sh — Shell POSIX Minimalista

`sh` é a implementação minimalista e rápida de shell POSIX do ecossistema **JeffUtils** / **Alinix OS**, desenvolvida em Rust.

## Recursos

- **Compatibilidade POSIX:** Execução de scripts shell com sintaxe padrão (`sh script.sh`).
- **Parsing AST Robusto:** Lexer e Parser descentes recursivos dedicados (`ast.rs`, `lexer.rs`, `parser.rs`).
- **Builtins Integrados:**
  - `cd` — alteração de diretórios com suporte a `-` e `~`.
  - `echo` — impressão com suporte a opções `-n` e `-e`.
  - `exit` — encerramento com código de retorno específico.
  - `export` / `unset` / `set` — gestão de variáveis de ambiente.
  - `pwd` — exibição do diretório de trabalho atual.
- **Pipelines e Redirecionamentos:** Suporte a pipes (`|`), redirecionamentos de entrada/saída (`>`, `>>`, `<`).
- **Execução Leve:** Projetado para inicializações rápidas e scripts de inicialização do sistema onde a sobrecarga de um shell interativo completo não é necessária.

## Compilação

```bash
cargo build --release -p sh
```

## Uso

```bash
# Executar um script shell
sh script.sh

# Modo interativo simples
sh
```
