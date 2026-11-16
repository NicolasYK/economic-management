use super::*;
impl FinanceApp {
    pub(super) fn dashboard(&mut self, ui: &mut egui::Ui) {
        ui.columns(3, |columns| {
            style::card(
                &mut columns[0],
                "SALDO ACUMULADO",
                money(self.summary.balance),
            );
            style::card(
                &mut columns[1],
                "RECEITAS DO MÊS",
                money(self.summary.income),
            );
            style::card(
                &mut columns[2],
                "DESPESAS DO MÊS",
                money(self.summary.expenses),
            );
        });
        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Resultado do mês: {}",
                money(self.summary.income - self.summary.expenses)
            ));
            ui.separator();
            ui.label(format!(
                "Reserva: {}",
                money(self.settings.as_ref().map_or(0, |s| s.reserve))
            ));
            ui.separator();
            ui.strong(format!(
                "Livre após reserva: {}",
                money(self.summary.available)
            ));
        });
        if self.summary.available < 0 {
            ui.colored_label(
                style::warning(ui),
                "O saldo está abaixo da reserva planejada.",
            );
        }
        ui.add_space(23.0);
        ui.columns(2, |columns| {
            columns[0].heading("Para onde foi seu dinheiro");
            columns[0].add_space(12.0);
            if self.summary.categories.is_empty() {
                columns[0].label(
                    "Nenhum gasto neste mês. Inclua seu primeiro gasto para ver a distribuição.",
                );
            } else {
                donut(
                    &mut columns[0],
                    &self.summary.categories,
                    self.summary.expenses,
                );
                for (i, (name, amount)) in self.summary.categories.iter().enumerate() {
                    columns[0].horizontal(|ui| {
                        ui.colored_label(style::palette(i), "●");
                        ui.label(name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.strong(money(*amount));
                        });
                    });
                }
            }
            columns[1].heading("Últimos cinco gastos");
            columns[1].add_space(12.0);
            let mut recent: Vec<_> = self.expenses.iter().collect();
            recent.sort_by(|a, b| b.date.cmp(&a.date).then(b.id.cmp(&a.id)));
            if recent.is_empty() {
                columns[1].label("Seu histórico aparecerá aqui.");
            }
            for row in recent.into_iter().take(5) {
                columns[1].horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.strong(&row.description);
                        ui.small(format!("{} · {}", row.category, row.date));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(money(row.amount));
                    });
                });
                columns[1].separator();
            }
        });
        ui.add_space(24.0);
        ui.horizontal(|ui| {
            ui.heading("Compromissos do mês");
            if ui.button("+ Recorrência ou parcela").clicked() {
                self.new_entry(1);
            }
        });
        let mut any = false;
        for c in &self.commitments {
            if let Some(value) = self.charges.get(&c.id).and_then(|v| v.0) {
                any = true;
                ui.horizontal(|ui| {
                    ui.label(&c.description);
                    ui.strong(money(value));
                    if let Some(count) = c.installments {
                        ui.small(format!(
                            "Parcela {} de {count}",
                            self.month.0 - c.start.0 + 1
                        ));
                    } else {
                        ui.small("Mensal");
                    }
                });
            }
        }
        if !any {
            ui.label("Nenhum compromisso para este mês.");
        }
    }
}
fn donut(ui: &mut egui::Ui, categories: &[(String, i64)], total: i64) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(260.0, 220.0), egui::Sense::hover());
    let center = rect.center();
    let radius = 96.0;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    for (i, (_, value)) in categories.iter().enumerate() {
        let sweep = (*value as f64 / total as f64 * std::f64::consts::TAU) as f32;
        let steps = (sweep / 0.04).ceil().max(1.0) as usize;
        for step in 0..steps {
            let a = angle + sweep * step as f32 / steps as f32;
            let b = angle + sweep * (step + 1) as f32 / steps as f32;
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    center,
                    center + egui::vec2(a.cos(), a.sin()) * radius,
                    center + egui::vec2(b.cos(), b.sin()) * radius,
                ],
                style::palette(i),
                egui::Stroke::NONE,
            ));
        }
        angle += sweep;
    }
    ui.painter()
        .circle_filled(center, 67.0, ui.visuals().panel_fill);
    ui.painter().text(
        center - egui::vec2(0.0, 12.0),
        egui::Align2::CENTER_CENTER,
        "TOTAL DO MÊS",
        egui::FontId::proportional(11.0),
        ui.visuals().text_color(),
    );
    ui.painter().text(
        center + egui::vec2(0.0, 13.0),
        egui::Align2::CENTER_CENTER,
        money(total),
        egui::FontId::proportional(18.0),
        ui.visuals().text_color(),
    );
}
