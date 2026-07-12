# JeffUtils

Conjunto oficial de utilitários de linha de comando do **Alinix OS**, escrito em Rust. Inclui o shell **JSH**, o coletor de informações **jeofetch**, além de reimplementações dos comandos essenciais do sistema (coreutils, rede, processos, disco e mais).

Todos os utilitários compartilham um único workspace Cargo, otimizado para binários pequenos e rápidos (`lto`, `strip`, `panic = "abort"`).

## Destaques

- **jsh** — shell interativo do Alinix, com parser próprio, pipelines, completions e builtins.
- **jeofetch** — exibe informações do sistema com o logo ASCII da distribuição (incluindo o logo do Alinix).
- **jtop** — monitor de processos e recursos em tempo real.
- **sysinfo / kinfo / uptime** — informações de sistema e kernel.

## Utilitários

Organizados por categoria (cada um é um crate independente do workspace):

| Categoria | Comandos |
|-----------|----------|
| **Shell & terminal** | `jsh`, `sh`, `clear`, `clock`, `texit`, `reload` |
| **Sistema & info** | `jeofetch`, `jtop`, `sysinfo`, `kinfo`, `uptime`, `version`, `help`, `time` |
| **Arquivos** | `ls`, `cp`, `mv`, `create`, `remove`, `rename`, `link`, `tree`, `search`, `find`, `stat`, `perms` |
| **Texto** | `echo`, `head`, `tail`, `wc`, `read`, `hash` |
| **Processos** | `ps`, `kill`, `jobs`, `nice` |
| **Usuários** | `whoami`, `groups`, `passwd`, `env` |
| **Rede** | `net`, `dns`, `ping` |
| **Disco & memória** | `mount`, `umount`, `fscheck`, `memory`, `zram`, `clear-cache` |
| **Energia** | `poweroff`, `reboot` |
| **Diversos** | `pwd`, `which`, `uuid`, `sleep` |

## Compilação

Requer a toolchain Rust (`cargo`).

```bash
# Compilar todos os utilitários (workspace inteiro)
cargo build --release --workspace

# Ou compilar um utilitário específico
cargo build --release -p jsh
```

Os binários ficam em `target/release/`.

### Via Makefile

O `Makefile` na raiz detecta automaticamente os projetos e suporta cross-compile:

```bash
make build                 # compila tudo
make ARCH=arm build        # cross-compile para aarch64
make package-deb           # gera pacote .deb
make package-rpm           # gera pacote .rpm
make info                  # lista projetos detectados
make clean
```

## Instalação

O `install.sh` compila a partir do código-fonte e instala os binários em `/opt/jeffutils`:

```bash
sudo ./install.sh <stage-dir>
```

No Alinix OS, os utilitários já vêm instalados e integrados ao PATH do sistema.

## Licença

Consulte o arquivo [`LICENSE`](LICENSE) (disponível em PT-BR e EN).

## Créditos

- **Autor/Desenvolvedor Original**: Jefferson (Mantenedor do Alinix OS).
