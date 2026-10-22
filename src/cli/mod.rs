pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rscommit",
    version,
    about = "CLI agnóstica para padrões de engenharia - Conventional Commits",
    long_about = "rscommit é uma ferramenta independente de linguagem para validar mensagens de commit seguindo a especificação Conventional Commits.\n\nEla pode ser usada como:\n  - CLI standalone para validar mensagens\n  - Git hook (commit-msg) para bloquear commits inválidos automaticamente\n\nConfiguração via rscommit.toml no repositório.",
    after_help = "EXEMPLOS:\n    rscommit init                          # Cria rscommit.toml com config padrão\n    rscommit validate -m \"feat: add login\"  # Valida mensagem direta\n    echo \"feat: add login\" | rscommit validate  # Valida via stdin\n    rscommit install                       # Instala git hook commit-msg\n    rscommit uninstall                     # Remove git hook\n\nCONFIGURAÇÃO:\n    O arquivo rscommit.toml controla as regras de validação.\n    Use 'rscommit init --full' para gerar config comentada completa.\n\nRECURSOS:\n    https://github.com/RafaelQSantos-RQS/rscommit\n    https://www.conventionalcommits.org/"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inicializa configuração padrão no repositório atual
    #[command(
        about = "Cria rscommit.toml com configuração padrão",
        long_about = "Cria um arquivo rscommit.toml no diretório atual com configuração padrão.\nSe o arquivo já existe, não sobrescreve.\n\nApós criar, revise o arquivo se necessário e rode 'rscommit install' para ativar o hook."
    )]
    Init,

    /// Valida uma mensagem de commit
    #[command(
        about = "Valida uma mensagem contra as regras do rscommit.toml",
        long_about = "Valida uma mensagem de commit seguindo a especificação Conventional Commits.\n\nA mensagem pode ser passada via:\n  - Argumento -m/--message\n  - Stdin (pipe)\n  - Arquivo .git/COMMIT_EDITMSG (quando usado como hook)\n\nRegras validadas (configuráveis no rscommit.toml):\n  - Tipo do commit (feat, fix, chore, etc.)\n  - Scope opcional ou obrigatório\n  - Tamanho máximo do subject e header\n  - Case do subject e type\n  - Ponto final no subject\n  - Linha em branco antes do body/footer\n  - Breaking changes (XNOR: ! no header + BREAKING CHANGE no footer)\n  - Signed-off-by obrigatório"
    )]
    Validate {
        /// Mensagem do commit (ou usa stdin/.git/COMMIT_EDITMSG)
        #[arg(short, long, value_name = "MSG")]
        message: Option<String>,
    },

    /// Instala git hook commit-msg
    #[command(
        about = "Instala hook commit-msg que valida commits automaticamente",
        long_about = "Cria o arquivo .git/hooks/commit-msg que executa 'rscommit validate' a cada commit.\n\nO hook:\n  - Usa o binary rscommit do PATH (ou target/debug/rscommit em desenvolvimento)\n  - Bloqueia commits que não seguem as regras\n  - Mostra erro claro com sugestão de correção\n  - É idempotente (rode múltiplas vezes sem problema)\n  - Faz backup de hook existente antes de sobrescrever"
    )]
    Install,

    /// Remove git hook commit-msg
    #[command(
        about = "Remove hook commit-msg instalado pelo rscommit",
        long_about = "Remove o hook .git/hooks/commit-msg criado pelo rscommit.\n\nSe existia um hook anterior (backup), ele será restaurado.\nHooks não criados pelo rscommit são mantidos intactos."
    )]
    Uninstall,
}
