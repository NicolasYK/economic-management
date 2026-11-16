use super::*;
impl FinanceApp {
    pub(super) fn expenses_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Buscar descrição ou categoria");
            ui.add(egui::TextEdit::singleline(&mut self.filter).desired_width(260.0));
        });
        ui.add_space(15.0);
        let needle = self.filter.to_lowercase();
        let prefix = self.month.to_string();
        let rows: Vec<_> = self
            .expenses
            .iter()
            .filter(|e| {
                e.date.starts_with(&prefix)
                    && (e.description.to_lowercase().contains(&needle)
                        || e.category.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect();
        ui.label(format!("{} gastos · agrupados por categoria", rows.len()));
        if rows.is_empty() {
            ui.label("Nenhum gasto encontrado. Use Novo gasto para começar.");
        }
        egui::ScrollArea::vertical()
            .id_salt("expense-list")
            .max_height(460.0)
            .show_rows(ui, 54.0, rows.len(), |ui, range| {
                for index in range {
                    let e = &rows[index];
                    ui.push_id(e.id, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&e.description);
                                ui.small(format!(
                                    "{} · {}{}",
                                    e.category,
                                    e.date,
                                    if e.automatic { " · automático" } else { "" }
                                ));
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if !e.automatic {
                                        if ui.small_button("Excluir").clicked() {
                                            self.confirm = Some(Confirmation::Expense(e.id));
                                        }
                                        if ui.small_button("Editar").clicked() {
                                            self.form =
                                                Some(forms::EntryForm::expense(e, self.today));
                                        }
                                    }
                                    ui.strong(money(e.amount));
                                },
                            );
                        });
                        ui.separator();
                    });
                }
            });
        ui.add_space(20.0);
        ui.collapsing("Gerenciar categorias",|ui|{
            ui.label("Alimentação, Hobby ou categorias criadas por você. Categorias em uso podem ser renomeadas.");
            ui.horizontal(|ui|{ui.text_edit_singleline(&mut self.category_name);if ui.button(if self.category_edit.is_some(){"Salvar nome"}else{"Criar categoria"}).clicked(){let result=if let Some(id)=self.category_edit{self.store.rename_category(id,&self.category_name)}else{self.store.add_category(&self.category_name).map(|_|())};if self.finish(result){self.category_name.clear();self.category_edit=None;}}});
            for c in self.categories.clone(){ui.horizontal(|ui|{ui.label(&c.name);if ui.small_button("Renomear").clicked(){self.category_edit=Some(c.id);self.category_name=c.name.clone();}
if ui.small_button("Excluir").clicked(){self.confirm=Some(Confirmation::Category(c.id));}});}
        });
    }
    pub(super) fn commitments_page(&mut self, ui: &mut egui::Ui) {
        ui.label("Cobranças no primeiro dia do mês. Com o aplicativo fechado, são registradas na próxima abertura.");
        ui.add_space(15.0);
        let now = Month::from_date(self.today);
        if self.commitments.is_empty() {
            ui.label("Inclua aluguel, assinaturas ou uma compra parcelada em Novo gasto.");
        }
        for c in self.commitments.clone() {
            let next = if c.start > now { c.start } else { now.next() };
            let next_charge = self.charges.get(&c.id).and_then(|v| v.1);
            egui::Frame::group(ui.style())
                .inner_margin(15)
                .corner_radius(10)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(&c.description);
                            let category = self
                                .categories
                                .iter()
                                .find(|x| x.id == c.category_id)
                                .map_or("", |x| x.name.as_str());
                            if let Some(n) = c.installments {
                                let paid = self.paid.get(&c.id).copied().unwrap_or(0);
                                ui.label(format!(
                                    "{category} · {paid}/{n} parcelas · total {}",
                                    money(c.amount)
                                ));
                            } else {
                                ui.label(format!("{category} · mensal · desde {}", c.start));
                            }
                            if let Some(value) = next_charge {
                                ui.small(format!("Próxima: {next} · {}", money(value)));
                            } else {
                                ui.small("Sem cobranças futuras");
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(next_amount) = next_charge {
                                if ui.button("Cancelar futuras").clicked() {
                                    self.confirm = Some(Confirmation::Cancel(c.id));
                                }
                                if c.installments.is_none() && ui.button("Alterar valor").clicked()
                                {
                                    let mut f = forms::EntryForm::new(3, self.today, c.category_id);
                                    f.id = Some(c.id);
                                    f.amount = input_money(next_amount);
                                    self.form = Some(f);
                                }
                            }
                        });
                    });
                });
            ui.add_space(10.0);
        }
    }
    pub(super) fn report_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Ano");
            if ui
                .add(egui::DragValue::new(&mut self.year).range(2000..=2100))
                .changed()
            {
                let r = self.refresh();
                if let Err(e) = r {
                    self.error = Some(e.to_string());
                }
            }
            if ui.button("Exportar CSV").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .set_file_name(format!("relatorio-{}.csv", self.year))
                    .save_file()
            {
                let result = self.store.annual_csv(self.year).and_then(|csv| {
                    use std::io::Write;
                    let mut f = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(path)?;
                    f.write_all(csv.as_bytes())?;
                    Ok(())
                });
                if self.finish(result) {
                    self.notice = Some("Relatório exportado.".into());
                }
            }
        });
        ui.add_space(18.0);
        egui::Grid::new("annual-table")
            .striped(true)
            .min_col_width(145.0)
            .spacing([20.0, 14.0])
            .show(ui, |ui| {
                for title in ["Mês", "Receitas", "Despesas", "Resultado"] {
                    ui.strong(title);
                }
                ui.end_row();
                for m in &self.summary.annual {
                    ui.label(format!("{:02}/{}", m.month, self.year));
                    ui.label(money(m.income));
                    ui.label(money(m.expenses));
                    ui.label(money(m.income - m.expenses));
                    ui.end_row();
                }
                let income: i64 = self.summary.annual.iter().map(|m| m.income).sum();
                let expenses: i64 = self.summary.annual.iter().map(|m| m.expenses).sum();
                ui.strong("Total");
                ui.strong(money(income));
                ui.strong(money(expenses));
                ui.strong(money(income - expenses));
                ui.end_row();
            });
        ui.add_space(18.0);
        ui.label(format!(
            "Saldo de abertura: {} · Saldo de fechamento: {}",
            money(self.summary.opening_year),
            money(self.summary.closing_year)
        ));
        ui.add_space(20.0);
        ui.heading("Despesas por categoria");
        for (name, amount) in &self.annual_categories {
            ui.horizontal(|ui| {
                ui.label(name);
                ui.strong(money(*amount));
            });
        }
    }
    pub(super) fn settings_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Renda mensal");
        forms::field(ui, "Nova renda (R$)", &mut self.income);
        forms::field(ui, "Vigência (AAAA-MM)", &mut self.first);
        ui.small("Alterações começam no próximo mês ou depois, preservando rendas já registradas.");
        if ui.button("Programar renda").clicked() {
            let result = (|| {
                self.store.schedule_income(
                    parse_money(&self.income)?,
                    self.first.parse()?,
                    self.today,
                )
            })();
            if self.finish(result) {
                self.notice = Some("Nova renda programada.".into());
            }
        }
        ui.add_space(22.0);
        ui.heading("Reserva e aparência");
        forms::field(ui, "Reserva de emergência (R$)", &mut self.reserve);
        ui.small("Valor protegido no planejamento. Não gera desconto mensal.");
        ui.horizontal(|ui| {
            for (value, title) in [
                ("light", "Claro"),
                ("dark", "Escuro"),
                ("system", "Sistema"),
            ] {
                ui.selectable_value(&mut self.theme, value.to_owned(), title);
            }
        });
        if ui.button("Salvar preferências").clicked() {
            let result = (|| {
                self.store
                    .set_preferences(parse_money(&self.reserve)?, &self.theme)
            })();
            if self.finish(result) {
                self.notice = Some("Preferências salvas.".into());
            }
        }
        ui.add_space(22.0);
        ui.heading("Seus dados");
        ui.label(
            "Os dados ficam neste computador, sem criptografia. Proteja o arquivo e seus backups.",
        );
        if let Some(path) = self.store.path() {
            ui.small(path.display().to_string());
        }
        ui.horizontal(|ui| {
            if ui.button("Exportar backup").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .set_file_name(format!("economia-{}.sqlite", self.today))
                    .add_filter("Backup SQLite", &["sqlite"])
                    .save_file()
            {
                let result = self.store.export_backup(&path);
                if self.finish(result) {
                    self.notice = Some(format!("Backup salvo em {}", path.display()));
                }
            }
            if ui.button("Restaurar backup").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Backup SQLite", &["sqlite"])
                    .pick_file()
            {
                match Store::inspect_backup(&path) {
                    Ok(info) => {
                        self.confirm = Some(Confirmation::Restore(
                            path,
                            format!(
                                "{} lançamentos · {} categorias · saldo {}",
                                info.transactions,
                                info.categories,
                                money(info.balance)
                            ),
                        ))
                    }
                    Err(e) => self.error = Some(e.to_string()),
                }
            }
        });
        ui.small("Escolha um novo nome ao exportar. Arquivos existentes são preservados.");
    }
    pub(super) fn about_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Economic Management");
        ui.label(format!(
            "Versão {} · commit {}",
            env!("CARGO_PKG_VERSION"),
            env!("BUILD_COMMIT")
        ));
        ui.add_space(20.0);
        ui.label("Planejamento financeiro pessoal, local e sem cadastro.");
        ui.label("Licença Apache 2.0");
        ui.label("Rust · egui · SQLite");
        ui.add_space(15.0);
        ui.label("Renda e cobranças são registros contábeis. Este aplicativo não acessa contas bancárias nem movimenta dinheiro.");
        ui.label("Com o programa fechado, os meses pendentes são atualizados na próxima abertura.");
    }
}
