# rscommit

> **CLI agnóstica para validação de Conventional Commits** — independente de linguagem, sem dependências de runtime.

[![CI](https://github.com/RafaelQSantos-RQS/rscommit/workflows/CI/badge.svg)](https://github.com/RafaelQSantos-RQS/rscommit/actions)
[![Release](https://github.com/RafaelQSantos-RQS/rscommit/workflows/Release/badge.svg)](https://github.com/RafaelQSantos-RQS/rscommit/actions)
[![license](https://img.shields.io/crate/l/rscommit.svg)](https://github.com/RafaelQSantos-RQS/rscommit/blob/main/LICENSE)

---

## 🎯 Por que rscommit?

Ferramentas de padronização de commits (como Commitlint, Husky) costumam estar atreladas a:
- **Node.js / npm** — precisa instalar o ecossistema JavaScript
- **Python / pip** — depende do runtime Python
- **Gerenciadores de pacote específicos** — não funciona em repositórios Go, Rust, Java, etc.

**rscommit resolve isso:** é um **binário standalone** escrito em Rust. Zero dependências de runtime. Funciona em qualquer repositório, independentemente da linguagem ou stack.

---

## 📦 Instalação

### 📥 Binário standalone (recomendado)
Baixe o binário para sua plataforma na [página de releases](https://github.com/RafaelQSantos-RQS/rscommit/releases):

```bash
# Linux (x86_64)
curl -L https://github.com/RafaelQSantos-RQS/rscommit/releases/latest/download/rscommit-x86_64-unknown-linux-gnu -o rscommit
chmod +x rscommit
sudo mv rscommit /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/RafaelQSantos-RQS/rscommit/releases/latest/download/rscommit-aarch64-apple-darwin -o rscommit
chmod +x rscommit
sudo mv rscommit /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/RafaelQSantos-RQS/rscommit/releases/latest/download/rscommit-x86_64-apple-darwin -o rscommit
chmod +x rscommit
sudo mv rscommit /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/RafaelQSantos-RQS/rscommit/releases/latest/download/rscommit-x86_64-pc-windows-msvc.exe" -OutFile "rscommit.exe"
# Adicione ao PATH
```

> **Por que binário?** Zero dependências, instalação instantânea, funciona em qualquer máquina sem Rust, Node, Python ou gerenciadores de pacote.

### Alternativas (se preferir)

<details>
<summary><strong>Via cargo-binstall</strong></summary>

> ⚠️ **Em desenvolvimento (backlog)** — Publicação no cargo-binstall está planejada para futuras versões.

```bash
# cargo binstall rscommit
```
</details>

<details>
<summary><strong>Via cargo</strong></summary>

```bash
cargo install rscommit
```
</details>

<details>
<summary><strong>Build from source</strong></summary>

```bash
git clone https://github.com/RafaelQSantos-RQS/rscommit
cd rscommit
cargo build --release
# Binário em target/release/rscommit
```
</details>

### Verificar instalação
```bash
rscommit --version
# rscommit 0.1.0
```

---

## 🚀 Quickstart

```bash
# 1. Inicializa configuração padrão no repositório
rscommit init

# 2. Instala o git hook (commit-msg)
rscommit install

# 3. Commite normalmente - mensagens inválidas são bloqueadas
git commit -m "feat: adiciona login do usuário"
# ✓ Commit message válida

git commit -m "invalid: mensagem ruim"
# Error: Tipo de commit inválido: 'invalid'. Tipos permitidos: build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test
```

---

## ⚙️ Configuração

O arquivo `rscommit.toml` controla todas as regras. Gere um config comentado completo:

```bash
rscommit init --full
```

### Estrutura completa

```toml
# rscommit - Conventional Commits configuration
# https://github.com/RafaelQSantos-RQS/rscommit

[commit]
# Tipos de commit permitidos (padrão: especificação Conventional Commits)
allowed_types = ["build", "chore", "ci", "docs", "feat", "fix", "perf", "refactor", "revert", "style", "test"]

# Scopes permitidos (vazio = qualquer scope)
# allowed_scopes = ["auth", "api", "ui", "db", "infra"]

# Tamanho máximo do subject em caracteres (padrão: 72)
max_subject_length = 72

# Exigir scope no commit (padrão: false)
require_scope = false

# Case do subject: lower, upper, sentence, pascal, kebab, any (padrão: lower)
subject_case = "lower"

# Case do type: lower, upper, sentence, pascal, kebab, any (padrão: lower)
type_case = "lower"

# Proibir ponto final no subject (padrão: false)
subject_full_stop = false

# Tamanho máximo do subject (regra separada, padrão: 72)
subject_max_length = 72

# Tamanho máximo do header completo (padrão: 72)
header_max_length = 72

# Case do scope (padrão: lower)
scope_case = "lower"

# Exigir linha em branco antes do body (padrão: false)
body_leading_blank = false

# Tamanho máximo de cada linha do body (padrão: 100, URLs permitidas)
body_max_line_length = 100

# Exigir linha em branco antes do footer (padrão: false)
footer_leading_blank = false

# Tamanho máximo de cada linha do footer (padrão: 100)
footer_max_line_length = 100

# Exigir XNOR: ! no header E "BREAKING CHANGE:" no footer (padrão: false)
breaking_change_exclamation_mark = false

# Exigir footer "Signed-off-by:" (padrão: false)
signed_off_by = false

[hooks]
# Instalar hook commit-msg automaticamente ao rodar 'rscommit install' (padrão: false)
commit_msg = false
```

### Herança de configuração (`extends`)

Compartilhe configurações entre repositórios:

```toml
# repo-a/rscommit.toml (base)
[commit]
allowed_types = ["feat", "fix", "chore"]
max_subject_length = 50

# repo-b/rscommit.toml (herda de repo-a)
extends = "../repo-a/rscommit.toml"

[commit]
allowed_types = ["feat", "fix"]  # sobrescreve
subject_full_stop = true        # adiciona regra
```

- Suporta paths relativos, absolutos
- Detecção de ciclos (max 10 níveis)
- URLs (http/https): **não implementado ainda**

---

## 📋 Referência de Regras

| Regra | Config | Padrão | Descrição |
|-------|--------|--------|-----------|
| **type-enum** | `allowed_types` | 11 tipos | Restringe tipo a lista permitida |
| **type-case** | `type_case` | `lower` | Case do tipo (lower, upper, sentence, pascal, kebab, any) |
| **type-empty** | — | obrigatório | Tipo não pode ser vazio |
| **subject-empty** | — | obrigatório | Subject não pode ser vazio |
| **subject-case** | `subject_case` | `lower` | Case do subject |
| **subject-full-stop** | `subject_full_stop` | `false` | Proíbe ponto final no subject |
| **subject-max-length** | `subject_max_length` | `72` | Tamanho máximo do subject |
| **header-max-length** | `header_max_length` | `72` | Tamanho máximo do header (type+scope+subject) |
| **scope-enum** | `allowed_scopes` | `[]` (qualquer) | Restringe scopes permitidos |
| **scope-case** | `scope_case` | `lower` | Case do scope |
| **scope-empty** | `require_scope` | `false` | Exige scope |
| **header-max-length** | `header_max_length` | `72` | Header completo ≤ 72 chars |
| **body-leading-blank** | `body_leading_blank` | `false` | Linha em branco antes do body |
| **body-max-line-length** | `body_max_line_length` | `100` | Linhas do body ≤ 100 chars (URLs ok) |
| **footer-leading-blank** | `footer_leading_blank` | `false` | Linha em branco antes do footer |
| **footer-max-line-length** | `footer_max_line_length` | `100` | Linhas do footer ≤ 100 chars |
| **breaking-change-xnor** | `breaking_change_exclamation_mark` | `false` | Exige `!` no header **E** `BREAKING CHANGE:` no footer |
| **signed-off-by** | `signed_off_by` | `false` | Exige footer `Signed-off-by:` |

---

## 💡 Exemplos Práticos

### Commits válidos
```bash
# Básico
feat: adiciona autenticação JWT

# Com scope
feat(auth): adiciona login com Google

# Breaking change
feat(api)!: remove endpoint deprecated

# Com body
fix(bug): corrige vazamento de memória

O leak ocorria quando a conexão não era fechada
corretamente em caso de timeout.

# Breaking change com footer
feat(api)!: remove endpoint /v1/users

BREAKING CHANGE: Use /v2/users instead.

# Signed-off-by
feat: adiciona cache Redis

Signed-off-by: João Silva <joao@exemplo.com>
```

### Commits inválidos (bloqueados)
```bash
# Tipo inválido
invalid: mensagem

# Tipo em case errado
FEAT: adiciona feature

# Subject com ponto final
feat: adiciona login.

# Subject muito longo
feat: esta mensagem de commit tem um subject extremamente longo que excede o limite máximo de setenta e dois caracteres permitidos

# Scope não permitido
feat(invalido): mensagem

# Scope em case errado
feat(AUTH): login

# Falta scope (quando require_scope=true)
feat: mensagem

# Body sem linha em branco
feat: mensagem
body direto sem linha em branco

# Footer sem linha em branco
feat: msg
BREAKING CHANGE: breaking

# Breaking change incompleto (apenas ! ou apenas footer)
feat!: breaking
feat: msg
BREAKING CHANGE: breaking

# Falta Signed-off-by (quando signed_off_by=true)
feat: mensagem
```

### Configuração por tipo de projeto

**Projeto com scopes obrigatórios:**
```toml
[commit]
require_scope = true
allowed_scopes = ["auth", "api", "ui", "db", "infra"]
```

**Monorepo com config compartilhada:**
```toml
# packages/ui/rscommit.toml
extends = "../../rscommit.toml"

[commit]
allowed_scopes = ["button", "form", "modal"]
```

**Projeto com breaking changes rigorosos:**
```toml
[commit]
breaking_change_exclamation_mark = true
signed_off_by = true
```

---

## 🔧 Comandos

| Comando | Descrição |
|---------|-----------|
| `rscommit init` | Cria `rscommit.toml` padrão |
| `rscommit init --full` | Cria `rscommit.toml` com todas as opções comentadas |
| `rscommit validate` | Valida mensagem (stdin / `-m` / `.git/COMMIT_EDITMSG`) |
| `rscommit validate -m "feat: msg"` | Valida mensagem direta |
| `rscommit install` | Instala hook `commit-msg` |
| `rscommit uninstall` | Remove hook (restaura backup se houver) |
| `rscommit --help` | Ajuda geral |
| `rscommit <cmd> --help` | Ajuda do subcomando |
| `rscommit -V` / `--version` | Mostra versão |

---

## 🔄 Como funciona o hook

```bash
rscommit install
```

Cria `.git/hooks/commit-msg`:
```sh
#!/bin/sh
# rscommit git hook - automatically generated
# Do not edit manually, use 'rscommit uninstall' to remove

exec "/caminho/para/rscommit" validate 2>&1
```

- **Idempotente**: rode quantas vezes quiser
- **Backup**: hook existente → `.git/hooks/commit-msg.rscommit.bak`
- **Path detection**: usa `target/debug/rscommit` (dev) ou `rscommit` do PATH (produção)
- **Restauração**: `rscommit uninstall` restaura backup se existir

---

## 🤝 Contribuindo

```bash
# Clone
git clone https://github.com/RafaelQSantos-RQS/rscommit
cd rscommit

# Desenvolvimento
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Build
cargo build --release
```

---

## 📄 Licença

Licenciado sob **MIT OR Apache-2.0** — veja [LICENSE](LICENSE) para detalhes.

---

## 🔗 Links

- [Conventional Commits Spec](https://www.conventionalcommits.org/)
- [Commitlint](https://commitlint.js.org/) — inspiração para as regras
- [git-conventional](https://github.com/crate-ci/git-conventional) — parser Rust usado internamente
- [Issues](https://github.com/RafaelQSantos-RQS/rscommit/issues) — reporte bugs ou sugira features