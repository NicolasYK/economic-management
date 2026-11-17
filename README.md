# Economic Management

Finanças pessoais no seu computador. **Sem conta, nuvem ou telemetria.**
Personal finances on your computer. **No account, cloud or telemetry.**

[Português](#português) · [English](#english)

## Português

Aplicativo desktop em Rust com SQLite incorporado e interface em português.

- Visão geral com gráfico de rosca, saldo, reserva e os cinco últimos gastos.
- Despesas por categoria: Alimentação, Hobby e categorias personalizadas.
- Recorrências e compras parceladas com cálculo exato em centavos.
- Renda mensal automática, histórico e relatório anual com exportação CSV.
- Temas Claro, Escuro e Sistema.
- Backup SQLite que pode ser restaurado em outro computador/sistema operacional.

### Baixar e abrir

Os pacotes serão disponibilizados em [Releases](https://github.com/NicolasYK/economic-management/releases).
Use o arquivo correspondente ao seu sistema e confira o `.sha256` incluído.
Não é necessário instalar Rust, Python, Node ou um servidor de banco de dados.

| Sistema | Pacote e requisitos |
| --- | --- |
| Windows | ZIP x86_64; extraia e abra `economic-management.exe`. Alvo Windows 10/11 de 64 bits. |
| Linux | AppImage x86_64; ambiente gráfico X11/Wayland e glibc 2.39+ (Ubuntu 24.04 ou equivalente), driver OpenGL. Diálogos de arquivos usam o portal do desktop. |
| macOS | ZIP com `.app`; arquitetura indicada no nome do pacote, macOS 14+. |

Linux: dê permissão de execução (`chmod +x arquivo.AppImage`) e abra o arquivo.
Sem FUSE, execute `./arquivo.AppImage --appimage-extract-and-run`.
Os pacotes iniciais não possuem assinatura comercial nem notarização Apple; o
sistema pode apresentar avisos. Compatibilidade não significa todas as versões
antigas ou arquiteturas de cada sistema.

### Primeiros passos

1. Informe o saldo que você possui **antes** do primeiro crédito, sua renda mensal
   e o mês do primeiro crédito (atual ou próximo). Confira a prévia para não
   contabilizar seu salário duas vezes. Renda zero é aceita.
2. Em **Novo gasto**, escolha Único, Mensal ou Parcelado. No parcelamento, informe
   o valor total: R$ 100,00 em 3x vira R$ 33,34 + R$ 33,33 + R$ 33,33.
3. Em **Despesas**, filtre por descrição/categoria, edite gastos manuais e gerencie categorias.
4. Em **Configurações**, programe renda futura, defina a reserva e salve seu tema.
5. Exporte regularmente um backup e guarde uma cópia em local seguro.

Renda e cobranças mensais são registradas no primeiro dia. Se o programa estiver
fechado, os meses pendentes são processados na próxima abertura. Quando aberto,
a data é verificada a cada 30 segundos. Isso é contabilidade local: **nenhum
dinheiro é movimentado em bancos**.

Reserva é um valor protegido no planejamento, não uma despesa mensal. Saldo livre
= saldo acumulado − reserva. Saldo negativo é mostrado sem esconder o déficit.
Alterações de renda e recorrências valem para meses futuros. Cancelar um compromisso
preserva o histórico. Ocorrências automáticas passadas não têm edição manual nesta versão.

Valores aceitam vírgula ou ponto decimal **sem separador de milhares**; limite por
valor: R$ 10 bilhões. Datas de 2000 a 2100; de 1 a 360 parcelas. A interface está
em português brasileiro e trabalha exclusivamente com BRL.

### Dados e backup

O caminho exato do banco aparece em Configurações. Normalmente fica na pasta de
dados local do usuário: `%LOCALAPPDATA%/EconomicManagement` no Windows,
`$XDG_DATA_HOME` ou `~/.local/share` no Linux, e `~/Library/Application Support`
no macOS. Use `--data-dir CAMINHO` para escolher uma pasta dedicada.

**Os arquivos não são criptografados.** Quem puder ler o arquivo poderá consultar
seu histórico financeiro. Proteja a conta do SO, o disco e as cópias exportadas.

O backup usa um snapshot SQLite consistente, não uma cópia de um banco aberto.
Para transferir, exporte no computador antigo e use **Restaurar backup** no novo.
A restauração valida o arquivo, mostra um resumo e pede confirmação: ela **substitui
toda a base**, sem mesclar. Uma cópia `recovery-*.sqlite` é salva antes da troca.
Limites de importação: 50 MiB e 100 mil registros por tabela. Arquivos incompatíveis,
adulterados ou de versões futuras são recusados. Ao exportar, escolha um nome novo;
o aplicativo preserva arquivos que já existem.

Se outra instância alterar o banco enquanto você o utiliza, reabra o aplicativo
antes de editar. Essa proteção impede que formulários antigos alterem dados restaurados.

### Desenvolvimento e Git

Toolchain fixada em `rust-toolchain.toml`; dependências em `Cargo.lock`. No Windows,
o build nativo exige C++ Build Tools/Windows SDK (MSVC). No Linux, instale os pacotes
de desenvolvimento de X11, xkbcommon, Wayland, GL/EGL e `pkg-config`.

```sh
cargo run -- --data-dir qa-output/manual
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
cargo install cargo-audit --version 0.23.0 --locked
cargo audit
```

`main` é a versão oficial/homologada. `codex/technical-design` é a branch permanente
de desenvolvimento alpha/beta, mantida após cada promoção. Mudanças chegam à `main`
por pull request, com testes Linux/Windows/macOS, auditoria, branch atualizada e
conversas resolvidas. As regras também se aplicam aos administradores e bloqueiam
force-push e exclusão da `main`. Não há aprovação humana obrigatória neste projeto
individual; a revisão técnica e os checks são parte da promoção.
Cada commit tem título convencional e corpo explicativo. Para versões, altere
`Cargo.toml`, atualize [CHANGELOG.md](CHANGELOG.md) e crie uma tag correspondente,
como `v0.1.0`. Tags estáveis devem apontar para commits já integrados à `main`;
tags como `v0.2.0-beta.1` devem pertencer à branch dev e são publicadas como
pré-releases, sem substituir a versão estável mais recente.
O workflow rejeita divergência entre tag e versão, testa os três SOs,
audita dependências e publica os pacotes somente depois de todos os builds passarem.
Sobre exibe a versão e o commit incorporados na compilação.

Empacotamento: `python scripts/package.py linux|windows|macos` após o build.
Python é necessário somente para esse script de desenvolvimento. Ferramentas
AppImage são verificadas por SHA-256; alterações upstream exigem revisão dos hashes.
As builds locais Linux foram preparadas no Ubuntu 24.04 via WSL; os resultados de
validação e limitações estão em [docs/verification.md](docs/verification.md).

## English

A Rust desktop application with bundled SQLite, a Portuguese interface and BRL
integer-cent accounting. It includes a dashboard/donut chart, five recent expenses,
custom categories, monthly income, recurring bills, installments, emergency reserve,
annual reports/CSV export, three themes and portable SQLite backups.

### Run

Download the matching package from [Releases](https://github.com/NicolasYK/economic-management/releases)
and verify its `.sha256`. Extract the Windows x86_64 ZIP and run the EXE. On Linux,
mark the x86_64 AppImage executable and run it; use `--appimage-extract-and-run` if
FUSE is unavailable. Linux requires a desktop, glibc 2.39+, an OpenGL driver and a
desktop portal for file dialogs. On macOS, extract the `.app`; the filename identifies
its architecture, targeting macOS 14+. Initial binaries are unsigned/not notarized.
No Rust, Python, Node or database-server installation is needed to use the app.

### Accounting and privacy

Enter an opening balance that excludes the first salary credit, then income and
the first credit month (current or next). Income and bills post on the first day.
Closed applications catch up at the next launch; running applications check every
30 seconds. There is no background daemon, bank access or real money transfer.

The reserve reduces the available balance without becoming a recurring expense.
Income/bill changes apply to future months and preserve history. Enter installment
**purchase totals**; R$100.00 / 3 is exactly 33.34 + 33.33 + 33.33. Past automatic
entries cannot be manually edited. Monetary input accepts up to two decimal digits
without thousands separators, up to R$10 billion per value; dates span 2000–2100,
and installment counts span 1–360.

Data resides in your OS user data directory; Settings shows the exact location.
`--data-dir PATH` selects a dedicated directory. **Database and backup files are
not encrypted.** Protect your OS account, disk and exported copies.

Backup exports are consistent SQLite snapshots usable across operating systems.
Restore validates the file and asks for confirmation, then replaces the entire
local dataset transactionally, retaining a `recovery-*.sqlite` copy. It does not
merge databases. Import limits are 50 MiB and 100,000 rows per table. Existing
export files are never overwritten; choose a new name.

If another instance changes the database, reopen the application before editing.
This prevents stale forms from modifying restored records.

### Contributing and releases

Use the pinned Rust toolchain and lockfile. Run the commands in the development
section above. `main` contains official releases; `codex/technical-design` is the
permanent alpha/beta development branch. Promote through pull requests with all
three platform checks and security audit passing, an up-to-date branch and resolved
conversations. Administrators are also covered; force pushes and deletion are blocked.
This individual project does not require another human approval.
Stable tags must belong to `main`; prerelease tags must belong to the development
branch and are published as prereleases without replacing the latest stable release.
CI runs tests, formatting, Clippy and dependency audits. Tags must match
the Cargo version; release packages are published only after successful platform
builds and checks. Python is a packaging tool only, never an application dependency.

See [CHANGELOG.md](CHANGELOG.md), [SECURITY.md](SECURITY.md), and
[verification notes](docs/verification.md). Licensed under [Apache 2.0](LICENSE).
