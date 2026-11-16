use super::*;
pub(super) struct EntryForm {
    pub kind: usize,
    pub id: Option<i64>,
    pub name: String,
    pub amount: String,
    pub category: i64,
    pub date: String,
    pub start: String,
    pub end: String,
    pub count: u32,
    pub error: Option<String>,
}
impl EntryForm {
    pub fn new(kind: usize, today: chrono::NaiveDate, category: i64) -> Self {
        Self {
            kind,
            id: None,
            name: String::new(),
            amount: String::new(),
            category,
            date: today.to_string(),
            start: Month::from_date(today).to_string(),
            end: String::new(),
            count: 12,
            error: None,
        }
    }
    pub fn expense(e: &Expense, today: chrono::NaiveDate) -> Self {
        Self {
            kind: 0,
            id: Some(e.id),
            name: e.description.clone(),
            amount: input_money(e.amount),
            category: e.category_id,
            date: e.date.clone(),
            ..Self::new(0, today, e.category_id)
        }
    }
}
pub(super) fn field(ui: &mut egui::Ui, title: &str, value: &mut String) {
    ui.label(title);
    ui.add(egui::TextEdit::singleline(value).desired_width(350.0));
}
impl FinanceApp {
    pub(super) fn entry_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut form) = self.form.take() else {
            return;
        };
        let mut close = false;
        let mut save = false;
        egui::Window::new(if form.kind == 3 {
            "Alterar valor mensal"
        } else if form.id.is_some() {
            "Editar despesa"
        } else {
            "Novo gasto"
        })
        .collapsible(false)
        .resizable(false)
        .default_width(430.0)
        .show(ctx, |ui| {
            if form.id.is_none() {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut form.kind, 0, "Único");
                    ui.selectable_value(&mut form.kind, 1, "Mensal");
                    ui.selectable_value(&mut form.kind, 2, "Parcelado");
                });
                ui.separator();
            }
            if let Some(error) = &form.error {
                ui.colored_label(style::warning(ui), error);
            }
            if form.kind != 3 {
                field(ui, "Descrição", &mut form.name);
                ui.label("Categoria");
                egui::ComboBox::from_id_salt("entry-category")
                    .selected_text(
                        self.categories
                            .iter()
                            .find(|c| c.id == form.category)
                            .map_or("Selecione", |c| c.name.as_str()),
                    )
                    .show_ui(ui, |ui| {
                        for c in &self.categories {
                            ui.selectable_value(&mut form.category, c.id, &c.name);
                        }
                    });
            }
            field(
                ui,
                if form.kind == 2 {
                    "Valor total da compra (R$)"
                } else {
                    "Valor (R$)"
                },
                &mut form.amount,
            );
            if form.kind == 0 {
                field(ui, "Data (AAAA-MM-DD)", &mut form.date);
            }
            if form.kind == 1 || form.kind == 2 {
                field(ui, "Primeira cobrança (AAAA-MM)", &mut form.start);
                if form.kind == 1 {
                    field(ui, "Último mês (opcional, AAAA-MM)", &mut form.end);
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Número de parcelas");
                        ui.add(egui::DragValue::new(&mut form.count).range(1..=360));
                    });
                }
                if let (Ok(amount), Ok(start)) =
                    (parse_money(&form.amount), form.start.parse::<Month>())
                {
                    if form.kind == 2
                        && let (Ok(first), Ok(last)) = (
                            installment(amount, form.count, 0),
                            installment(amount, form.count, form.count - 1),
                        )
                    {
                        ui.label(format!(
                            "{} parcelas · primeira {} · última {}",
                            form.count,
                            money(first),
                            money(last)
                        ));
                        ui.small(format!(
                            "De {start} até {}",
                            Month(start.0 + form.count as i32 - 1)
                        ));
                    }
                    if start == Month::from_date(self.today) {
                        ui.colored_label(
                            style::warning(ui),
                            "A primeira cobrança será registrada agora, no mês atual.",
                        );
                    }
                }
            }
            if form.kind == 3 {
                ui.label("O novo valor passa a valer no próximo mês. O histórico permanece igual.");
            }
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                if ui.button("Cancelar").clicked() {
                    close = true;
                }
                if ui
                    .add(egui::Button::new("Salvar").fill(style::accent(ui)))
                    .clicked()
                {
                    save = true;
                }
            });
        });
        if save {
            let result = (|| -> Result<()> {
                let amount = parse_money(&form.amount)?;
                match form.kind {
                    0 => {
                        let date = parse_date(&form.date)?;
                        if let Some(id) = form.id {
                            self.store.edit_expense(
                                id,
                                &form.name,
                                amount,
                                form.category,
                                date,
                                self.today,
                            )
                        } else {
                            self.store
                                .add_expense(&form.name, amount, form.category, date, self.today)
                                .map(|_| ())
                        }
                    }
                    3 => self
                        .store
                        .revise_commitment(form.id.unwrap(), amount, self.today),
                    _ => self
                        .store
                        .add_commitment(
                            NewCommitment {
                                description: form.name.clone(),
                                category_id: form.category,
                                amount,
                                installments: if form.kind == 2 {
                                    Some(form.count)
                                } else {
                                    None
                                },
                                start: form.start.parse()?,
                                end: if form.kind == 1 && !form.end.trim().is_empty() {
                                    Some(form.end.parse()?)
                                } else {
                                    None
                                },
                            },
                            self.today,
                        )
                        .map(|_| ()),
                }
            })();
            match result {
                Ok(()) => {
                    self.finish(Ok(()));
                    close = true;
                }
                Err(e) => form.error = Some(e.to_string()),
            }
        }
        if !close {
            self.form = Some(form);
        }
    }
}
