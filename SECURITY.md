# Security / Segurança

## Português

O aplicativo não usa contas, rede, telemetria nem serviços bancários. Dados e
backups contêm informações financeiras privadas e **não são criptografados**.
Proteja sua sessão do sistema operacional e use criptografia de disco quando necessário.

Formulários e importações são validados no domínio. SQL usa parâmetros; centavos
inteiros evitam arredondamentos binários. O backup limita arquivos a 50 MiB e
100 mil registros por tabela, exige o esquema canônico, valida as regras financeiras
e reconstrói os registros em um banco novo. A restauração substitui os registros
em uma única transação SQLite e mantém backup de recuperação em disco.

Não publique seu banco em issues. Para falhas, envie somente reprodução sintética
e versão do aplicativo, preferencialmente pelo recurso de relato privado de
segurança do GitHub quando disponível.

### Auditoria de dependências — 16/11/2026

`cargo audit 0.23.0` não identificou vulnerabilidades conhecidas no lockfile
avaliado. Avisos informativos de manutenção continuam visíveis no CI:

| Dependência indireta | Aviso | Avaliação |
| --- | --- | --- |
| ttf-parser 0.25.1 | RUSTSEC-2026-0192 | Parser de fontes do egui; o aplicativo usa somente fontes incorporadas e não importa fontes do usuário. |
| zbus_names 4.3.5 | RUSTSEC-2026-0371 | Infraestrutura de acessibilidade/diálogos no Linux; incorporada ao zbus 6 na linha sucessora. |
| zvariant 5.15.1 | RUSTSEC-2026-0370 | Serialização local D-Bus; incorporada ao zbus 6 na linha sucessora. |
| zvariant_derive 5.15.1 | RUSTSEC-2026-0372 | Macros de compilação da mesma família D-Bus. |

Não são avisos de vulnerabilidade com correção disponível para estas versões.
Mantidos nesta primeira versão para não forçar atualizações maiores da interface
sem validação. Revisar as bibliotecas que os incluem antes da próxima release ou
até **16/12/2026**, o que vier primeiro. Não há supressões no cargo-audit.

## English

The application has no account, network access, telemetry or bank integration.
Local databases and backups contain private financial information and are **not
encrypted**. Protect your OS account and consider disk encryption.

Domain validation, bound SQL parameters, integer cents, canonical backup schemas,
bounded imports and transactional replacement protect data integrity. Restore
creates a recovery backup before replacing the current records. Never attach your
personal database to an issue; use synthetic reproduction steps instead.

The 2026-11-16 audit found no known vulnerabilities. The four informational
maintenance warnings above concern embedded font parsing and local D-Bus support.
They remain visible in CI and must be reviewed before the next release or
2026-12-16, whichever is earlier. Audits do not guarantee absence of unknown flaws.
