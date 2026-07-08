# JeffUtils

> Versão: 1.0 (Draft - Atualizado com Status de Implementação)

---

# Introdução

O **JeffUtils** é o conjunto oficial de utilitários de linha de comando do JeffNix.

Essas ferramentas constituem a interface padrão entre o usuário, o sistema operacional e o kernel, oferecendo comandos consistentes, previsíveis e integrados.

Todo sistema JeffNix deverá incluir o JeffUtils como sua suíte oficial de ferramentas.

---

# Objetivo

O JeffUtils busca oferecer uma experiência moderna para administração do sistema, substituindo a necessidade de diversas ferramentas independentes.

Entre seus objetivos estão:

- Interface consistente;
- Sintaxe padronizada;
- Alto desempenho;
- Integração total com o kernel;
- Facilidade de aprendizado;
- APIs reutilizáveis para outras aplicações.

---

# Ideia

Em vez de reunir utilitários criados em épocas e projetos diferentes, o JeffUtils foi concebido como uma coleção unificada.

Todos os comandos seguem as mesmas convenções de nomenclatura, parâmetros, mensagens de erro e formatos de saída.

Essa padronização reduz a curva de aprendizado, facilita a automação e melhora a manutenção do sistema.

---

# Filosofia

O JeffUtils é baseado em cinco princípios.

## Consistência

Todos os comandos devem apresentar comportamento previsível.

## Simplicidade

A sintaxe deve ser intuitiva e evitar parâmetros desnecessários.

## Integração

Os utilitários devem utilizar as APIs oficiais do kernel sempre que possível (comunicação direta por APIs/Syscalls, sem envelopar subcomandos Linux).

## Segurança

Operações potencialmente destrutivas devem exigir confirmação ou permissões apropriadas.

## Eficiência e Portabilidade

Os comandos devem consumir o mínimo possível de recursos do sistema e suportar multiplataforma de forma nativa.

# Comandos

O **JeffUtils** reúne os utilitários oficiais do JeffNix.

Todos os comandos seguem uma sintaxe consistente:

```bash
<comando> [subcomando] [argumentos] [opções]
```

## Filosofia

- Um comando, uma responsabilidade.
- Sintaxe previsível.
- Nomes intuitivos.
- Mensagens de erro padronizadas.
- Suporte a `--help` e `--version` em todos os programas.

---

# Sistema

| Comando      | Descrição                     | Status / Nota |
| ------------ | ----------------------------- | ------------- |
| c            | Limpa o terminal (alias)      | ⚠️ Alias de `clear` |
| clear        | Limpa o terminal              | ✅ Implementado (Multiplataforma) |
| help         | Lista comandos ou exibe ajuda | ✅ Implementado (Multiplataforma) |
| clock        | Exibe data e hora             | ✅ Implementado (Multiplataforma) |
| sysinfo      | Informações do sistema        | ✅ Implementado (Multiplataforma) |
| uptime       | Tempo ligado                  | ✅ Implementado (Multiplataforma) |
| version      | Exibe a versão do JeffNix     | ✅ Implementado (Multiplataforma) |
| reboot       | Reinicia o sistema            | ✅ Implementado (Multiplataforma) |
| poweroff     | Desliga o sistema             | ✅ Implementado (Multiplataforma) |
| logout       | Encerra a sessão atual        | 🚫 Não necessário |
| sleep        | Coloca o sistema em suspensão | ✅ Implementado (Multiplataforma) |
| reload       | Recarrega o shell do desktop  | ✅ Implementado (GNOME/KDE via D-Bus) |
| clear-cache  | Limpa cache e memória         | ✅ Implementado (Multiplataforma) |

---

# Navegação

| Comando | Descrição                  | Status / Nota |
| ------- | -------------------------- | ------------- |
| pwd     | Diretório atual            | ✅ Implementado (Multiplataforma) |
| ls      | Lista arquivos             | ✅ Implementado (Multiplataforma) |
| tree    | Exibe árvore de diretórios | ✅ Implementado (Multiplataforma) |
| cd      | Altera o diretório atual   | ✅ Implementado (Mensagem instrutiva - builtin do shell) |

---

# Arquivos

| Comando  | Descrição                     | Status / Nota |
| -------- | ----------------------------- | ------------- |
| create   | Cria arquivos ou diretórios   | ✅ Implementado (Multiplataforma) |
| remove   | Remove arquivos ou diretórios | ✅ Implementado (Multiplataforma) |
| cp       | Copia arquivos                | ✅ Implementado (Multiplataforma) |
| mv       | Move arquivos                 | ✅ Implementado (Multiplataforma) |
| rename   | Renomeia arquivos             | ✅ Implementado (Multiplataforma) |
| link     | Cria links                    | ✅ Implementado (Multiplataforma) |
| find     | Localiza arquivos             | ✅ Implementado (Multiplataforma) |
| search   | Pesquisa conteúdo em arquivos | ✅ Implementado (Multiplataforma) |
| stat     | Informações detalhadas        | ✅ Implementado (Multiplataforma) |
| perms    | Permissões                    | ✅ Implementado (Multiplataforma) |
| compress | Compacta arquivos             | 🚫 Não necessário |
| extract  | Extrai arquivos               | 🚫 Não necessário |

---

# Disco

| Comando | Descrição                                   | Status / Nota |
| ------- | ------------------------------------------- | ------------- |
| disk    | Gerenciamento completo de discos            | ✅ Implementado (Multiplataforma) |
| mount   | Alias para `disk mount`                     | ✅ Wrapper de `disk` |
| umount  | Alias para `disk unmount`                   | ✅ Wrapper de `disk` |
| fscheck | Verifica integridade do sistema de arquivos | ✅ Implementado (Multiplataforma) |

---

# Memória

| Comando   | Descrição             | Status / Nota |
| --------- | --------------------- | ------------- |
| mem       | Uso de memória        | 🚫 Não necessário |
| memory    | Alias de `mem`        | ✅ Implementado (Multiplataforma) |
| mem-test  | Teste de memória      | ✅ Implementado (Multiplataforma) |
| zram      | Gerenciamento da ZRAM | ✅ Implementado (Multiplataforma) |
| zram-test | Benchmark da ZRAM     | ✅ Implementado (Multiplataforma) |

---

# Processos

| Comando | Descrição                  | Status / Nota |
| ------- | -------------------------- | ------------- |
| ps      | Lista processos            | ✅ Implementado (Multiplataforma via API) |
| top     | Monitor em tempo real      | 🚫 Não necessário (Temos `jtop` melhorado) |
| kill    | Finaliza processos         | ✅ Implementado (Multiplataforma via API) |
| nice    | Prioridade                 | ✅ Implementado (Multiplataforma) |
| jobs    | Processos em segundo plano | ✅ Implementado (Mensagem instrutiva - builtin do shell) |

---

# Rede

| Comando  | Descrição              | Status / Nota |
| -------- | ---------------------- | ------------- |
| net      | Informações de rede    | ✅ Implementado (Multiplataforma via API) |
| ping     | Teste de conectividade | ✅ Implementado (Ping TCP nativo) |
| dns      | Consulta DNS           | ✅ Implementado (Resolução nativa) |
| download | Download de arquivos   | 🚫 Não necessário |
| upload   | Upload de arquivos     | 🚫 Não necessário |

---

# Usuários

| Comando | Descrição         | Status / Nota |
| ------- | ----------------- | ------------- |
| user    | Gerencia usuários | 🚫 Não necessário |
| login   | Inicia sessão     | 🚫 Não necessário |
| passwd  | Altera senha      | ✅ Implementado (Multiplataforma) |
| whoami  | Usuário atual     | ✅ Implementado (Multiplataforma) |
| groups  | Grupos do usuário | ✅ Implementado (Multiplataforma) |

---

# Aplicações

| Comando | Descrição                   | Status / Nota |
| ------- | --------------------------- | ------------- |
| pkg     | Gerenciador de pacotes      | 🚫 Não necessário |
| open    | Abre arquivos ou aplicações | 🚫 Não necessário |
| exec    | Executa um programa         | 🚫 Não necessário |
| which   | Localiza executáveis        | ✅ Implementado (Multiplataforma) |

---

# Utilidades

| Comando | Descrição               | Status / Nota |
| ------- | ----------------------- | ------------- |
| texit   | Editor de texto nativo  | ✅ Implementado (Multiplataforma) |
| calc    | Calculadora             | 🚫 Não necessário |
| uuid    | Gera UUID               | ✅ Implementado (Multiplataforma via API) |
| hash    | Calcula hashes          | ✅ Implementado (Multiplataforma via API) |
| random  | Gera números aleatórios | 🚫 Não necessário |
| json    | Formata JSON            | 🚫 Não necessário |
| time    | Mede tempo de execução  | ✅ Implementado (Multiplataforma) |
| jsh     | Shell interativo com tema | ✅ Implementado (Multiplataforma) |
| sh      | Shell bruto (cru) básico  | ✅ Implementado (Multiplataforma) |

---

# create

Cria arquivos ou diretórios.

Por padrão, cria um arquivo.

## Sintaxe

```bash
create <destino>
```

### Flags

| Flag            | Descrição                      |
| --------------- | ------------------------------ |
| -f              | Cria arquivo                   |
| -d              | Cria diretório                 |
| -r, --recursive | Cria diretórios intermediários |
| -c, --content   | Conteúdo inicial do arquivo    |

## Exemplos

Criar arquivo

```bash
create roadmap.md
```

Criar diretório

```bash
create Projetos -d
```

Criar estrutura completa

```bash
create Projetos/JeffNix/docs/FHS.md -f -r
```

Criar arquivo com conteúdo

```bash
create README.md -c "# Meu Projeto"
```

---

# remove

Remove arquivos ou diretórios.

## Flags

| Flag    | Descrição             |
| ------- | --------------------- |
| -f      | Remove arquivo        |
| -d      | Remove diretório      |
| -r      | Remove recursivamente |
| --force | Ignora confirmações   |

## Exemplos

```bash
remove teste.txt
```

```bash
remove Projetos -d
```

```bash
remove Projetos -d -r
```

---

---

# perms

Gerencia permissões, proprietário, grupos e listas de controle de acesso (ACL) de arquivos e diretórios.

O comando `perms` substitui os tradicionais `chmod`, `chown`, `chgrp` e `setfacl` encontrados em sistemas Unix, reunindo todas essas funcionalidades em uma única interface consistente.

## Sintaxe

```bash
perms <caminho> <ação> [opções]
```

## Consultar permissões

```bash
perms README.md
```

Exemplo de saída:

```
Owner : Jefferson
Group : Users

Permissions

Owner : rwx
Group : r-x
Others: r--

Protected : No
Immutable : No
```

## Alterar permissões

```bash
perms README.md set rw-r--r--
```

ou

```bash
perms README.md set owner=rwx group=rx others=r
```

## Alterar proprietário

```bash
perms README.md owner Jefferson
```

## Alterar grupo

```bash
perms README.md group Developers
```

## Tornar executável

```bash
perms programa exec
```

## Somente leitura

```bash
perms README.md readonly
```

## Proteger arquivo

Impede modificações ou remoção, mesmo por usuários administrativos, exceto quando autorizado pelo kernel.

```bash
perms kernel.sys protect
```

## Remover proteção

```bash
perms kernel.sys unprotect
```

## Gerenciamento de ACL

Permitir acesso:

```bash
perms README.md allow Jefferson rw
```

Negar acesso:

```bash
perms README.md deny Jefferson
```

## Aplicação recursiva

```bash
perms Projetos --recursive
```

# help

Exibe a documentação de um comando.

```bash
help
```

Lista todos os comandos disponíveis.

```bash
help disk
```

Mostra a documentação do comando.

O `help` procura inicialmente por uma documentação em:

```
/Shared/help/<comando>
```

Caso exista, seu conteúdo é exibido.

Caso contrário, o JeffUtils apresenta apenas uma descrição resumida do comando.

---

# clock

Exibe a data e hora atuais.

## Exemplos

```bash
clock
```

```bash
clock HH:mm:ss
```

```bash
clock dd/MM/yyyy
```

Definir formato padrão

```bash
clock --config HH:mm:ss
```

A configuração é armazenada no perfil do usuário.

---

# Aliases Oficiais

| Alias  | Comando      |
| ------ | ------------ |
| c      | clear        |
| memory | mem          |
| mount  | disk mount   |
| umount | disk unmount |
| chmod  | perms set    |
| chown  | perms owner  |
| chgrp  | perms group  |

---

# Convenções

Todos os comandos oficiais devem:

- possuir `--help`;
- possuir `--version`;
- retornar `0` em caso de sucesso;
- retornar códigos padronizados em caso de erro;
- utilizar mensagens consistentes;
- respeitar as permissões do sistema;
- utilizar a API oficial do kernel sempre que possível e portável.

---

# Dependências

- **Rust** (edition 2021+) — todos os utilitários são escritos em Rust.
- **zbus** — utilizado pelo comando `reload` para comunicação D-Bus.
- **make** — para automação de build via `Makefile`.
- Opcional: cross-compilation tools para builds direcionados a outras plataformas.

---

# Build e Instalação

## Compilar todos os utilitários

```bash
make
```

## Compilar um utilitário específico

```bash
make ls
make texit
```

## Instalar no sistema

```bash
make install
```

Por padrão, instala em `/bin` no modo `unix` / `x86`.

### Plataformas

| MODE      | ARCH   | Diretório de instalação      |
| --------- | ------ | ---------------------------- |
| unix      | x86    | `/bin`                       |
| unix      | arm    | `/bin`                       |
| mac       | x86    | `/usr/local/bin`             |
| mac       | arm    | `/usr/local/bin`             |
| win       | x86    | `C:/System32/JeffUtils`      |
| win       | arm    | `C:/System32/JeffUtils`      |

Exemplos:

```bash
make install MODE=mac ARCH=arm
make install MODE=win
```

## Limpar artefatos de build

```bash
make clean
```

---

# Licença

MIT License

Copyright (c) 2025 Jefferson

Permissão é concedida, gratuitamente, a qualquer pessoa que obtenha uma cópia
deste software e dos arquivos de documentação associados (o "Software"), para
lidar com o Software sem restrições, incluindo, sem limitação, os direitos de
usar, copiar, modificar, mesclar, publicar, distribuir, sublicenciar e/ou
vender cópias do Software, e de permitir que as pessoas a quem o Software é
fornecido o façam, sob as seguintes condições:

O aviso de copyright acima e este aviso de permissão devem ser incluídos em
todas as cópias ou partes substanciais do Software.

O Software é fornecido "como está", sem garantia de qualquer tipo, expressa
ou implícita, incluindo, mas não se limitando às garantias de
comercialização, adequação a um propósito específico e não violação. Em
nenhum caso os autores ou detentores dos direitos autorais serão
responsáveis por qualquer reclamação, dano ou outra responsabilidade,
seja em ação contratual, delitual ou de outra forma, decorrente de,
ou em conexão com o Software ou o uso ou outras negociações no
Software.

---

# Contribuição

1. Faça um fork do repositório.
2. Crie um branch para sua feature (`git checkout -b feat/minha-feature`).
3. Commit suas alterações (`git commit -m "feat: adiciona minha feature"`).
4. Faça push para o branch (`git push origin feat/minha-feature`).
5. Abra um Pull Request.

### Diretrizes

- Mantenha a consistência de nomenclatura e sintaxe dos comandos existentes.
- Todos os novos comandos devem implementar `--help` e `--version`.
- Utilize a API do sistema (syscalls) sempre que possível, evitando wrappers de comandos externos.
- Prefira Rust como linguagem para novos utilitários.
- Atualize este README e a documentação do comando ao adicionar ou modificar funcionalidades.
