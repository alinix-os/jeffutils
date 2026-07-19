# JeffUtils

Conjunto oficial de utilitários de linha de comando do **Alinix OS**, escrito em Rust. Inclui o shell **JSH**, o editor **texit**, o monitor **jtop**, além de reimplementações completas dos comandos essenciais do sistema — coreutils, rede, processos, disco, texto e mais.

Todos os utilitários compartilham um único workspace Cargo, otimizado para binários pequenos e rápidos (`lto`, `strip`, `panic = "abort"`).

## Destaques

- **jsh** — shell interativo com parser próprio, pipelines, background jobs, completions e builtins completos.
- **sh** — shell POSIX minimalista com builtins, pipes e expansão de glob.
- **texit** — editor de texto terminal com syntax highlighting, busca, undo/redo e suporte a UTF-8.
- **jeofetch** — informações do sistema com logo ASCII do Alinix OS.
- **jtop** — monitor de processos e recursos em tempo real.
- **cutils** — gerenciador de aliases para nomes padrão do Unix (`sort` -> `arrange`, `rm` -> `remove`, etc).

## Utilitários

### Shell & Terminal

| Comando | Descrição |
|---------|-----------|
| `jsh` | Shell interativo do Alinix |
| `sh` | Shell POSIX minimalista |
| `clear` | Limpa o terminal |
| `clock` | Exibe data/hora com formatação |
| `texit` | Editor de texto terminal |
| `reload` | Recarrega desktop shell via D-Bus |
| `termconfig` | Configura terminal (stty) |
| `terminal` | Verifica se stdin é terminal (tty) |

### Sistema & Informações

| Comando | Descrição |
|---------|-----------|
| `jeofetch` | Informações do sistema com logo ASCII |
| `jtop` | Monitor de processos |
| `sysinfo` | Resumo do sistema (OS, CPU, RAM) |
| `kinfo` | Informações do kernel |
| `uptime` | Tempo ligado e carga média |
| `identity` | UID/GID real e efetivo (id) |
| `online` | Quem está logado (who) |
| `cpucount` | Número de CPUs disponíveis (nproc) |
| `sessionuser` | Nome do usuário logado (logname) |
| `sessions` | Lista de usuários logados (users) |
| `usercheck` | Informações de usuário (pinky) |
| `machineid` | ID único da máquina (hostid) |
| `time` | Mede tempo de execução |
| `version` | Versão do jeffutils |
| `help` | Ajuda dos comandos |

### Arquivos & Diretórios

| Comando | Descrição |
|---------|-----------|
| `ls` | Lista diretórios |
| `cp` | Copia arquivos |
| `rename` | Move/renomeia arquivos (cross-device) |
| `remove` | Remove arquivos/diretórios (rm) |
| `create` | Cria arquivos/diretórios (touch/mkdir) |
| `touch` | Alias do create |
| `mkdir` | Alias do create |
| `link` | Cria hard/symlinks (ln) |
| `find` | Busca recursiva de arquivos |
| `search` | Busca texto em arquivos (grep) |
| `stat` | Metadados de arquivos |
| `perms` | Permissões, dono, ACL (chmod/chown) |
| `tree` | Árvore de diretórios |
| `leaf` | Strip de diretório (basename) |
| `stem` | Strip de componente (dirname) |
| `resolve` | Caminho canônico (realpath) |
| `dereference` | Valor de symlink (readlink) |
| `diskfree` | Espaço em disco por filesystem (df) |
| `spaceused` | Uso de disco recursivo (du) |

### Texto & Processamento

| Comando | Descrição |
|---------|-----------|
| `echo` | Imprime texto com escapes |
| `format` | Formata e imprime (printf) |
| `read` | Concatena arquivos no stdout (cat) |
| `head` | Primeiras N linhas |
| `tail` | Últimas N linhas, modo follow |
| `wc` | Conta linhas/palavras/bytes |
| `arrange` | Ordena linhas (sort) |
| `dedup` | Remove duplicatas adjacentes (uniq) |
| `slice` | Extrai campos (cut) |
| `convert` | Transfere caracteres (tr) |
| `flip` | Inverte ordem de linhas (tac) |
| `number` | Numera linhas (nl) |
| `stitch` | Junta arquivos lado a lado (paste) |
| `wrap` | Quebra linhas longas (fold) |
| `reflow` | Reformat parágrafos (fmt) |
| `untab` | Converte tabs para espaços (expand) |
| `retab` | Converte espaços para tabs (unexpand) |
| `mirror` | Tee — stdout + arquivo(s) |

### Números & Conversão

| Comando | Descrição |
|---------|-----------|
| `countup` | Sequência de números (seq) |
| `bytedump` | Dump em octal/hex/dec (od) |
| `calculate` | Avalia expressões (expr) |
| `unitformat` | Formata números com unidades (numfmt) |
| `primegen` | Fatorização em primos (factor) |

### Encoding & Checksums

| Comando | Descrição |
|---------|-----------|
| `encode64` | Base64 encode/decode |
| `encode32` | Base32 encode/decode |
| `hash` | SHA-256/SHA-512 de arquivos |
| `blake2` | BLAKE2b hash |
| `checksum` | CRC checksum (cksum) |
| `crcsum` | BSD 16-bit checksum (sum) |

### Processos & Sinais

| Comando | Descrição |
|---------|-----------|
| `ps` | Lista processos |
| `kill` | Envia sinais (SIGTERM, SIGKILL) |
| `jobs` | Jobs em background |
| `nice` | Ajusta prioridade |
| `persist` | Executa imune a hangups (nohup) |

### Disco & Dispositivos

| Comando | Descrição |
|---------|-----------|
| `mount` | Monta filesystems |
| `umount` | Desmonta filesystems |
| `blockcopy` | Cópia em bloco (dd) |
| `destroy` | Sobrescreve arquivo seguramente (shred) |
| `resize` | Redimensiona arquivo (truncate) |
| `chunk` | Divide arquivo em pedaços (split) |
| `segment` | Divide por padrão (csplit) |
| `temppath` | Cria arquivo temporário (mktemp) |
| `deploy` | Copia com permissões (install) |
| `pipefile` | Cria pipe nomeado (mkfifo) |
| `devnode` | Cria dispositivo especial (mknod) |
| `flush` | Sincroniza buffers (sync) |
| `fscheck` | Verifica integridade do filesystem |
| `pathcheck` | Valida caminhos (pathchk) |

### Memória & Swap

| Comando | Descrição |
|---------|-----------|
| `memory` | Uso de RAM/swap (free) |
| `mem-test` | Teste de memória |
| `zram` | Gerencia ZRAM |
| `zram-test` | Teste de ZRAM |
| `clear-cache` | Limpa cache do sistema |

### Rede

| Comando | Descrição |
|---------|-----------|
| `net` | Interfaces de rede |
| `dns` | Configura DNS |
| `ping` | Testa conectividade |

### Usuários & Grupos

| Comando | Descrição |
|---------|-----------|
| `whoami` | Nome do usuário atual |
| `groups` | Grupos do usuário |
| `passwd` | Altera senha |
| `env` | Variáveis de ambiente |

### Energia

| Comando | Descrição |
|---------|-----------|
| `poweroff` | Desliga o sistema |
| `reboot` | Reinicia o sistema |

### Diversos

| Comando | Descrição |
|---------|-----------|
| `pwd` | Diretório atual |
| `which` | Localiza comandos no PATH |
| `uuid` | Gera UUIDs |
| `sleep` | Pausa por duração |
| `context` | Contexto SELinux (chcon) |
| `conrun` | Executa com contexto SELinux |
| `jail` | Executa em root modificado (chroot) |
| `permutext` | Índice permutado (ptx) |
| `paginate` | Pagina texto para impressão (pr) |
| `toposort` | Ordenação topológica (tsort) |
| `compare` | Compara arquivos ordenados (comm) |
| `meld` | Junta arquivos por campo (join) |

## Compatibilidade com Coreutils

O comando `cutils` cria symlinks para que comandos padrão do Unix funcionem:

```bash
# Instalar aliases (ex: sort -> arrange, rm -> remove, etc)
sudo cutils install

# Listar todos os mapeamentos
cutils list

# Verificar quais estão instalados
cutils status

# Remover todos os aliases
sudo cutils uninstall

# Instalar em diretório custom
cutils install -d ~/bin

# Consultar mapeamento
cutils which sort    # sort -> arrange
cutils which rm      # rm -> remove
```

### Mapeamentos Principais

| Alias | Comando JeffUtils | Alias | Comando JeffUtils |
|-------|-------------------|-------|-------------------|
| `cat` | `read` | `rm` | `remove` |
| `chmod` | `perms` | `mv` | `rename` |
| `chown` | `perms` | `ln` | `link` |
| `sort` | `arrange` | `uniq` | `dedup` |
| `cut` | `slice` | `tr` | `convert` |
| `tac` | `flip` | `nl` | `number` |
| `tee` | `mirror` | `dd` | `blockcopy` |
| `df` | `diskfree` | `du` | `spaceused` |
| `shred` | `destroy` | `truncate` | `resize` |
| `basename` | `leaf` | `dirname` | `stem` |
| `realpath` | `resolve` | `readlink` | `dereference` |
| `id` | `identity` | `who` | `online` |
| `nproc` | `cpucount` | `yes` | `repeat` |
| `seq` | `countup` | `od` | `bytedump` |
| `expr` | `calculate` | `printf` | `format` |
| `fold` | `wrap` | `fmt` | `reflow` |
| `mktemp` | `temppath` | `install` | `deploy` |
| `sync` | `flush` | `nohup` | `persist` |
| `tty` | `terminal` | `factor` | `primegen` |
| `tsort` | `toposort` | `comm` | `compare` |
| `join` | `meld` | `pr` | `paginate` |
| `expand` | `untab` | `unexpand` | `retab` |
| `numfmt` | `unitformat` | `csplit` | `segment` |
| `base64` | `encode64` | `base32` | `encode32` |
| `cksum` | `checksum` | `b2sum` | `blake2` |
| `mkfifo` | `pipefile` | `mknod` | `devnode` |
| `chroot` | `jail` | `stty` | `termconfig` |
| `dircolors` | `dirtheme` | `pathchk` | `pathcheck` |
| `hostid` | `machineid` | `logname` | `sessionuser` |
| `users` | `sessions` | `pinky` | `usercheck` |
| `chcon` | `context` | `runcon` | `conrun` |
| `grep` | `search` | `touch` | `touch` |
| `mkdir` | `mkdir` | `mount` | `mount` |
| `ps` | `ps` | `kill` | `kill` |
| `free` | `memory` | `uname` | `kinfo` |

## Compilação

Requer a toolchain Rust (`cargo`).

```bash
# Compilar todos os utilitários
cargo build --release --workspace

# Compilar um utilitário específico
cargo build --release -p jsh
cargo build --release -p arrange
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
