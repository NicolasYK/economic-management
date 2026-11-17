# Changelog / Histórico de versões

Versionamento semântico / Semantic versioning: MAJOR.MINOR.PATCH.

## [0.1.0] - 2026-11-16

### Added / Adicionado

- Aplicativo desktop offline com SQLite, seis áreas e temas Claro/Escuro/Sistema.
  Offline desktop app with SQLite, six sections and Light/Dark/System themes.
- Renda mensal, gastos por categoria, recorrências e parcelas em centavos exatos.
  Monthly income, categorized expenses, recurring bills and exact-cent installments.
- Reconciliação de meses pendentes, preservação de histórico e reserva de emergência.
  Catch-up reconciliation, historical schedules and an emergency reserve.
- Relatório anual, CSV protegido contra fórmulas e backup portátil validado.
  Annual reporting, formula-safe CSV and validated portable backups.
- Testes automatizados, CI multiplataforma, SemVer/commit na interface e pacotes
  independentes para Windows/Linux/macOS. Automated tests, cross-platform CI,
  version/commit display and separate Windows/Linux/macOS packages.

### Security / Segurança

- SQL parametrizado, validação no domínio, importação com limite de tamanho,
  esquema canônico e substituição transacional com cópia de recuperação.
  Bound SQL, domain validation, size-bounded canonical imports and transactional
  restore with a recovery copy.
- Auditoria sem vulnerabilidades conhecidas em 16/11/2026; avisos de manutenção
  documentados em SECURITY.md. Audit found no known vulnerabilities on 2026-11-16;
  maintenance advisories are documented in SECURITY.md.

### Known limitations / Limitações conhecidas

- Interface em português e moeda BRL; backups sem criptografia; pacotes não
  assinados/notarizados. Portuguese UI/BRL only; unencrypted backups; unsigned packages.
- Sem execução com o aplicativo fechado: atualização na próxima abertura.
  No background execution: closed months are reconciled on the next launch.
- Ocorrências automáticas passadas não podem ser editadas; mudanças valem para o
  futuro. Past automatic entries cannot be edited; schedule changes apply in future.
- Linux requer ambiente gráfico, glibc 2.39+ e driver OpenGL. Linux requires a
  desktop, glibc 2.39+ and an OpenGL driver.

Primeira versão: não há regressões de versões públicas anteriores a listar.
Initial release: no fixes to earlier public versions are claimed.
