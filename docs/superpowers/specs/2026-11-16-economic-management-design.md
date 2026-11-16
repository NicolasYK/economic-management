# Economic Management — projeto técnico

Data: 16/11/2026. Estado: **proposta para revisão; implementação não iniciada**.

O usuário solicitou revisar este projeto antes da implementação. As escolhas
abaixo são recomendações técnicas, ainda sujeitas à aprovação. O nome de exibição
inicial será **Economic Management**, acompanhando o repositório.

## 1. Objetivo e limites

Aplicativo desktop de finanças pessoais, simples e acolhedor, que mantém renda,
gastos, parcelas e histórico exclusivamente no computador. Sem cadastro, nuvem,
telemetria, integração bancária ou serviço em segundo plano.

A primeira versão contempla todas as seis áreas solicitadas, backups portáveis,
testes automatizados, documentação em português e inglês e distribuição por
sistema operacional. Interface inicialmente em português brasileiro, moeda BRL,
datas locais e valores com duas casas decimais. Múltiplas moedas, contas bancárias,
sincronização, pagamentos reais e aplicativo móvel ficam fora desta versão.

## 2. Abordagens consideradas

| Opção | Benefícios para este projeto | Custos e limites |
| --- | --- | --- |
| **Rust + egui/eframe + SQLite — recomendada** | Programa compilado, sem interpretador instalado pelo usuário; interface e regras no mesmo projeto; controle de alocações e dependências | Compilação inicial mais demorada; aparência personalizada em vez dos controles nativos de cada SO; depende da infraestrutura gráfica do sistema |
| Python + Tkinter + SQLite + PyInstaller | Implementação simples; acompanha a intenção do README inicial; SQLite e Tkinter disponíveis no ecossistema padrão | Precisa incluir o interpretador nos pacotes; aparência e integração de temas exigem mais trabalho; empacotamento separado por SO |
| Rust + Tauri + frontend web | Maior liberdade visual com HTML/CSS | Adiciona frontend e dependências de WebView; dificulta garantir execução sem instalação de componentes adicionais em todos os alvos |

Recomendação: **Rust com egui/eframe e rusqlite com SQLite incorporado**. A
prioridade é funcionamento offline e consumo contido. O README atual menciona
Python, mas não há código a migrar. A escolha definitiva depende desta revisão.

Versões exatas do compilador e bibliotecas serão fixadas durante a implementação,
com `Cargo.lock` versionado. Consumo real será medido; não se promete um valor de
RAM sem executar o aplicativo. A interface só solicitará redesenho quando houver
interação, mudança relevante ou verificação periódica da data, sem animação contínua.

## 3. Experiência e identidade visual

Navegação lateral: **Visão geral**, **Despesas**, **Recorrentes e parcelas**,
**Relatório anual**, **Configurações**, **Sobre**. Conteúdo com hierarquia clara,
espaçamento confortável, rótulos explícitos e ações de inclusão visíveis.

Paleta proposta: verde sálvia como cor principal, fundos creme no tema claro e
grafite no escuro; terracota para alertas. Cores de categorias terão contraste
entre si, acompanhadas de texto e valores. O contraste será verificado nos dois
temas. Tema **Sistema** acompanha a preferência disponível no SO; se ela não
estiver disponível, usa Claro. A preferência fica salva localmente.

Fluxo inicial: informar renda mensal, mês do primeiro crédito e saldo inicial.
Renda zero é válida. O saldo inicial significa o dinheiro existente **antes** do
primeiro crédito registrado no aplicativo; a tela mostra uma prévia para evitar
contar o salário duas vezes. Pode-se escolher o próximo mês para o primeiro crédito.

| Tela | Conteúdo e ações |
| --- | --- |
| Visão geral | Saldo acumulado; renda e despesas do mês; resultado mensal; saldo livre após a reserva; gráfico de rosca por categoria com legenda e valores; últimos cinco gastos; compromissos do mês e próximas parcelas; botão Novo gasto |
| Despesas | Lista por mês, agrupada por categoria, com data, descrição e valor; filtros; incluir, editar e excluir despesas manuais; criar categorias |
| Recorrentes e parcelas | Lista com valor mensal, categoria, início, próxima cobrança e progresso das parcelas; inclusão; encerramento de recorrências; cancelamento de parcelas futuras com confirmação |
| Relatório anual | Seletor de ano; doze linhas mensais de receitas, despesas e resultado; totais por categoria e do ano; saldo inicial/final do período; exportação CSV |
| Configurações | Renda e mês de vigência; reserva; tema; exportar/restaurar backup; localização do banco |
| Sobre | Versão, identificador do commit, licença, informação de armazenamento local |

Estados vazios explicam a próxima ação, sem gráficos fictícios. Erros preservam os
campos preenchidos. Listas grandes usam paginação ou virtualização. Formulários
têm navegação por teclado e foco visível. Exclusão e restauração pedem confirmação
na própria interface. Gastos automáticos são identificados como tais.

## 4. Regras financeiras

### Valores, saldo e reserva

- Armazenar e calcular dinheiro em **centavos inteiros**, nunca ponto flutuante.
- Aceitar vírgula decimal na interface; rejeitar valores inválidos, mais de duas
  casas decimais e valores fora do limite documentado. Valores de despesas devem
  ser positivos; renda e reserva podem ser zero. Saldo inicial pode ser negativo.
- Saldo acumulado = saldo inicial + receitas efetivadas − despesas efetivadas.
- Reserva é um valor protegido no planejamento, **não uma despesa mensal nem
  dinheiro separado automaticamente**. Saldo livre = saldo acumulado − reserva.
  Valores negativos permanecem visíveis com aviso, sem impedir lançamentos.
- Mudar a reserva não altera o histórico nem o saldo acumulado.
- Relatório anual usa a data contábil dos lançamentos; saldo inicial não é renda.

### Renda e virada do mês

- Crédito da renda no primeiro dia do mês, a partir do mês escolhido no cadastro.
- Guardar alterações de renda com mês de vigência. Por padrão, mudanças valem a
  partir do próximo mês; não reescrever receitas de meses já processados.
- Na abertura, processar todos os meses pendentes cronologicamente. Se ficar três
  meses fechado, registrar as rendas e cobranças dos três meses, usando as regras
  vigentes em cada um. Datas contábeis continuam sendo as datas de vencimento.
- Enquanto aberto, verificar a mudança do mês sem executar consultas a cada frame.
  Após suspensão do computador, a próxima verificação também reconcilia pendências.
- Cada ocorrência tem chave única, por exemplo `(tipo, origem, competência)`.
  Inserir ocorrências e atualizar controles na mesma transação. Reabrir, repetir
  a rotina ou executar duas instâncias não pode duplicar créditos/cobranças.
- Se o relógio retroceder, não apagar nem duplicar registros. Meses futuros já
  lançados devem ser sinalizados para revisão. Não tentar adivinhar o horário real
  pela internet; a data do dispositivo é a referência.
- Não há execução com o programa fechado: a atualização ocorre na próxima abertura.
  “Desconto” significa registro contábil local, sem movimentar dinheiro em bancos.

### Despesas e categorias

- Categorias iniciais: **Alimentação** e **Hobby**, além das personalizadas.
- Cada gasto tem descrição, data, valor e categoria. Categorias em uso podem ser
  renomeadas, mas não removidas sem reclassificação explícita dos registros.
- Despesas manuais são efetivadas em sua data, até o dia atual; despesas futuras
  pertencem ao cadastro de compromissos. Não aceitar datas anteriores ao início
  do controle financeiro definido no cadastro inicial.
- Editar/excluir uma despesa manual recalcula os resumos. Lançamentos automáticos
  não podem ser excluídos isoladamente e depois recriados pela reconciliação.
  A primeira versão permite alterar/cancelar compromissos futuros, preservando
  ocorrências já registradas. Correções retroativas de ocorrências automáticas
  ficam fora da edição comum e devem ser identificadas como limitação no manual.

### Recorrências e parcelamento

- Recorrência: descrição, categoria, valor mensal, primeiro mês e término opcional.
  Cobrança no primeiro dia de cada mês. O início pode ser no mês atual ou seguinte.
  Começando no atual, mostrar antes de salvar a cobrança imediata a ser registrada.
- Parcela: informar **valor total** e quantidade de parcelas (1 a 360), com prévia
  da primeira e última cobrança. O valor exibido como mensal é o valor da parcela.
- Dividir centavos exatamente: R$ 100,00 em 3x resulta em R$ 33,34 + R$ 33,33 +
  R$ 33,33. Distribuir o resto nas primeiras parcelas e exigir total suficiente
  para que nenhuma parcela seja zero. O total cobrado deve ser igual ao informado.
- Parar automaticamente após a última parcela. Não cobrar períodos anteriores
  ao início nem posteriores ao término/cancelamento.
- Alterações em recorrências valem no próximo mês. Antes de alterar, reconciliar
  meses vencidos com a configuração anterior. Parcelamentos não são redivididos
  depois de iniciados; cancelar futuras parcelas e cadastrar um novo compromisso
  é uma ação explícita, sem alterar o histórico anterior.

## 5. Arquitetura e armazenamento

Separar módulos de **domínio financeiro**, **persistência SQLite**, **interface**,
**backup** e **metadados de versão**. A interface chama serviços do domínio;
regras de centavos, calendário e parcelamento não dependem da interface gráfica.
O relógio é injetável nos testes, permitindo simular anos e viradas de mês.

Estrutura conceitual do banco:

- `settings`: tema, reserva, data de início e saldo inicial.
- `income_schedules`: valor da renda e primeiro mês de vigência.
- `categories`: identificador e nome único normalizado.
- `commitments`: recorrência/parcela, valores, início, término e cancelamento.
- `commitment_revisions`: alterações futuras de recorrências, preservando regras antigas.
- `transactions`: receitas e despesas, data contábil, origem, competência,
  categoria, descrição e valor; unicidade das ocorrências automáticas.
- `schema_migrations`: controle da evolução do banco, separado da versão do programa.

SQL parametrizado, chaves estrangeiras, restrições de domínio e transações
atômicas. Migrações são versionadas e precedidas de backup. Banco de versão mais
nova é recusado com mensagem clara, sem tentativa destrutiva de downgrade.

Banco na pasta de dados do usuário: `%LOCALAPPDATA%` no Windows,
`$XDG_DATA_HOME` (ou `~/.local/share`) no Linux e `~/Library/Application Support`
no macOS. A distribuição portátil não exige gravar ao lado do executável, que
pode estar em local sem permissão. Nenhuma gravação exige privilégios administrativos.

## 6. Backup e restauração

Backup exportado é um arquivo SQLite consistente, criado pela API de backup,
sem copiar apenas o arquivo principal de um banco em uso. Formato independente
do SO, contendo todos os dados e a versão do esquema.

Na restauração: limitar tamanho do arquivo (proposta inicial: 50 MiB), abrir
somente para leitura, desabilitar extensões, verificar integridade, versão,
estrutura, tipos, referências, limites e invariantes financeiros. Não executar
SQL fornecido pelo arquivo nem confiar em triggers/views importadas. Reconstruir
os dados validados em banco novo criado pelo aplicativo.

Mostrar resumo do conteúdo e solicitar confirmação. A operação **substitui** os
dados locais; não mescla bases. Antes de substituir, criar backup de recuperação.
Efetivar a troca de forma recuperável somente após validação completa e com acesso
exclusivo ao banco; falha mantém o banco anterior utilizável. Testar esse protocolo
separadamente em Windows, Linux e macOS.

## 7. Segurança e privacidade

Fronteiras de confiança: formulários, arquivos de backup, caminhos escolhidos pelo
usuário, banco local que pode ter sido alterado e dependências de compilação.
Ativos: integridade do histórico, saldo e privacidade das finanças.

Os valores e descrições são informações financeiras privadas mesmo sem CPF ou
dados bancários. O banco ficará sem criptografia na primeira versão, como permitido
na solicitação. A interface/manual explicam que quem tem acesso ao arquivo pode
lê-lo; permissões do SO e criptografia de disco são a proteção disponível.

Controles: validação também no domínio; limites de tamanho e quantidade; SQL
parametrizado; nenhuma chamada de shell baseada em dados; logs sem valores ou
descrições pessoais; mensagens de erro sem stack traces; sem rede em execução.
Exportação CSV neutraliza campos de texto interpretáveis como fórmulas.

Revisão de dependências, licenças e avisos de segurança com Cargo.lock fixado e
`cargo audit` antes da release. Vulnerabilidades críticas/altas alcançáveis sem
mitigação bloqueiam publicação. Workflows usam permissões mínimas e não recebem
dados financeiros. Não adicionar autenticação ou serviços externos.

## 8. Testes e critérios de aceite

| Área | Evidências necessárias |
| --- | --- |
| Cálculos | Centavos exatos, saldo negativo, limites, divisão com resto, reserva sem dupla subtração |
| Calendário | Dezembro/janeiro, fevereiro bissexto, meses fechados, relógio retrocedendo, suspensão |
| Automação | Renda uma vez por mês, mudança de vigência, últimas parcelas, cancelamento, repetição e concorrência sem duplicação |
| Persistência | Reabrir sem perda, rollback em falha, migrações, integridade e consultas parametrizadas |
| Backup | Transferência entre SOs, restauração válida, corrupção, versão futura, valores inválidos, arquivo grande, falha durante troca |
| Relatórios | Doze meses, ano vazio, totais consistentes com lançamentos, saldo de abertura e fechamento |
| Interface | Cadastro completo, despesas, recorrências, categorias, temas, teclado, estados vazios, confirmação de restauração |
| Segurança | Entradas abusivas, nomes com aspas/SQL, backups adulterados, fórmulas em CSV, auditoria das dependências |
| Distribuição | Executar pacote em ambiente limpo sem toolchain; conferir versão/commit e criar/restaurar dados locais |

CI executa formatação, Clippy, testes unitários e integração com bancos temporários
em Windows, Linux e macOS. Testes de interface automatizam o que for confiável;
inspeção visual documentada complementa os fluxos e temas. Medir RAM, CPU ociosa e
tempo de abertura em ambiente identificado com base sintética de 10 mil gastos;
definir o limite de regressão a partir dessa medição, sem inventar resultados.

## 9. Git, versões e releases

`main` estável e branches curtas `codex/...`. Uma `dev` permanente não é necessária
para este projeto neste estágio. Este documento fica em `codex/technical-design`.
Não é necessário criar um worktree adicional para documentação em repositório limpo.

Commits por incremento, com título convencional e corpo explicando mudanças e
motivação. Cada incremento passa pelos testes aplicáveis e revisão. PRs usam um
modelo com comportamento esperado, validação e riscos. Não incluir bancos pessoais,
backups, credenciais ou binários gerados no histórico Git.

Versão semântica em `Cargo.toml` como fonte do aplicativo; a tag de release deve
coincidir (`vMAJOR.MINOR.PATCH`), verificada pelo CI. Sobre mostra versão e commit
incorporados na compilação. `CHANGELOG.md` traz Added/Changed/Fixed/Security e
resumos em português e inglês; `README.md` traz instalação, uso e desenvolvimento
nos dois idiomas. Primeira versão funcional candidata: `0.1.0`.

Workflow de CI em push/PR; workflow de release em tag, com builds nativos e testes
antes da publicação. Publicar uma release por versão contendo pacotes separados:

- Windows x86_64: executável portátil em ZIP, sem instalador.
- Linux x86_64: AppImage e instruções; verificar opção de extração para sistemas
  sem FUSE. Biblioteca gráfica/driver e versões mínimas do sistema serão documentados.
- macOS Apple Silicon: aplicativo em ZIP, compilado no runner correspondente.
  Suporte Intel depende de build/teste adicional e será declarado separadamente.

A solicitação exige releases Linux/Windows e compatibilidade macOS; incluir o
pacote macOS torna essa compatibilidade verificável. Compatibilidade universal
com toda versão/arquitetura de SO não é prometida. Os alvos mínimos exatos serão
definidos e documentados após validar a toolchain e os artefatos.

Release contém notas derivadas do changelog, limitações conhecidas e checksums
SHA-256. Assinatura de código/notarização dependem de certificados do proprietário;
sem eles, informar que os pacotes não são assinados. Publicação depende de acesso
ao GitHub e sucesso real dos builds; configurar workflow não equivale a publicar.

## 10. Ordem de execução após aprovação

1. Fixar toolchain, dependências, estrutura e CI inicial.
2. Implementar domínio financeiro e SQLite com testes de cálculos e calendário.
3. Implementar reconciliação mensal, recorrências e parcelas com testes de falha.
4. Construir telas e temas; validar fluxos com dados sintéticos.
5. Adicionar backup, restauração, relatório e exportação, incluindo testes adversariais.
6. Revisar segurança, medir consumo, concluir documentação e verificar pacotes.
7. Publicar versão com artefatos testados, changelog e checksums.

## 11. Decisões principais para esta revisão

1. Aprovar Rust + egui/eframe ou preferir outra opção da seção 2.
2. Aprovar atualização na próxima abertura quando fechado, sem serviço residente.
3. Aprovar reserva como valor protegido, sem desconto mensal automático.
4. Aprovar cobrança no primeiro dia e edição de recorrências com efeito futuro.
5. Aprovar backup que substitui a base com recuperação, sem mesclagem.

Detalhes de implementação, nomes internos de módulos, layout final e seleção de
bibliotecas auxiliares podem ser resolvidos durante o desenvolvimento dentro destes
limites. Este documento não constitui evidência de implementação, testes ou releases.

## Referências técnicas

- [eframe: aplicativo desktop](https://docs.rs/eframe/latest/eframe/).
- [egui: temas Claro, Escuro e Sistema](https://docs.rs/egui/latest/egui/enum.ThemePreference.html).
- [rusqlite: SQLite incorporado](https://github.com/rusqlite/rusqlite/blob/master/README.md).
- [PyInstaller: builds por sistema operacional](https://www.pyinstaller.org/en/stable/).
