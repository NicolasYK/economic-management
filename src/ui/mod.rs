mod dashboard;
mod forms;
mod pages;
mod style;
use crate::{Result, domain::*, models::*, store::Store};
use eframe::egui::{self, RichText};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
pub const PAGES: [&str; 6] = [
    "Visão geral",
    "Despesas",
    "Recorrentes e parcelas",
    "Relatório anual",
    "Configurações",
    "Sobre",
];
pub struct FinanceApp {
    pub page: usize,
    store: Store,
    settings: Option<Settings>,
    summary: Snapshot,
    categories: Vec<Category>,
    expenses: Vec<Expense>,
    commitments: Vec<Commitment>,
    charges: std::collections::HashMap<i64, (Option<i64>, Option<i64>)>,
    annual_categories: Vec<(String, i64)>,
    month: Month,
    year: i32,
    error: Option<String>,
    notice: Option<String>,
    form: Option<forms::EntryForm>,
    confirm: Option<Confirmation>,
    opening: String,
    income: String,
    first: String,
    reserve: String,
    theme: String,
    category_name: String,
    category_edit: Option<i64>,
    filter: String,
    checked: Instant,
    today: chrono::NaiveDate,
    smoke_frames: Option<u32>,
}
enum Confirmation {
    Expense(i64),
    Category(i64),
    Cancel(i64),
    Restore(PathBuf, String),
}
impl FinanceApp {
    pub fn new(store: Store) -> Self {
        let today = chrono::Local::now().date_naive();
        let month = Month::from_date(today);
        let mut app = Self {
            page: 0,
            store,
            settings: None,
            summary: Snapshot::default(),
            categories: vec![],
            expenses: vec![],
            commitments: vec![],
            charges: Default::default(),
            annual_categories: vec![],
            month,
            year: month.0 / 12,
            error: None,
            notice: None,
            form: None,
            confirm: None,
            opening: "0,00".into(),
            income: "0,00".into(),
            first: month.to_string(),
            reserve: "0,00".into(),
            theme: "system".into(),
            category_name: String::new(),
            category_edit: None,
            filter: String::new(),
            checked: Instant::now(),
            today,
            smoke_frames: None,
        };
        let result = app.store.reconcile(today);
        app.finish(result);
        app.load_preferences();
        app
    }
    pub fn smoke_test(mut self) -> Self {
        self.smoke_frames = Some(0);
        self
    }
    fn load_preferences(&mut self) {
        if let Some(s) = &self.settings {
            self.reserve = input_money(s.reserve);
            self.theme = s.theme.clone();
            self.first = Month::from_date(self.today).next().to_string();
            self.income = input_money(
                self.store
                    .income_at(Month::from_date(self.today).next())
                    .unwrap_or(0),
            );
        }
    }
    fn refresh(&mut self) -> Result<()> {
        self.settings = self.store.settings()?;
        self.categories = self.store.categories()?;
        self.summary = self.store.snapshot(self.month, self.year)?;
        self.expenses = self.store.expenses(None)?;
        self.commitments = self.store.commitments()?;
        self.annual_categories = self.store.annual_categories(self.year)?;
        self.charges.clear();
        let now = Month::from_date(self.today);
        for c in &self.commitments {
            let next = if c.start > now { c.start } else { now.next() };
            self.charges.insert(
                c.id,
                (
                    self.store.charge_at(c, self.month)?,
                    self.store.charge_at(c, next)?,
                ),
            );
        }
        Ok(())
    }
    fn finish<T>(&mut self, result: Result<T>) -> bool {
        match result {
            Ok(_) => {
                self.error = None;
                if let Err(e) = self.refresh() {
                    self.error = Some(e.to_string());
                    return false;
                }
                true
            }
            Err(e) => {
                self.error = Some(e.to_string());
                false
            }
        }
    }
    fn new_entry(&mut self, kind: usize) {
        self.form = Some(forms::EntryForm::new(
            kind,
            self.today,
            self.categories.first().map_or(1, |c| c.id),
        ));
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        style::apply(ctx, &self.theme);
        if self.checked.elapsed() > Duration::from_secs(30) {
            self.checked = Instant::now();
            self.today = chrono::Local::now().date_naive();
            let result = self.store.reconcile(self.today);
            self.finish(result);
        }
        ctx.request_repaint_after(Duration::from_secs(30));
        egui::SidePanel::left("navigation")
            .exact_width(205.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(23.0);
                ui.label(RichText::new("economic").size(25.0).strong());
                ui.label(RichText::new("M A N A G E M E N T").size(10.5));
                ui.add_space(36.0);
                for (index, title) in PAGES.iter().enumerate() {
                    if ui
                        .add_sized(
                            [180.0, 42.0],
                            egui::Button::new(*title).selected(self.page == index),
                        )
                        .clicked()
                    {
                        self.page = index;
                    }
                    ui.add_space(5.0);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(15.0);
                    ui.small(format!(
                        "v{} · {}",
                        env!("CARGO_PKG_VERSION"),
                        env!("BUILD_COMMIT")
                    ));
                    ui.label("Seus dados ficam com você.");
                    ui.small("100% local · sem conta");
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            if let Some(error) = &self.error {
                ui.colored_label(style::warning(ui), error);
                ui.add_space(10.0);
            }
            if let Some(notice) = &self.notice {
                ui.label(notice);
                ui.add_space(8.0);
            }
            if self.settings.is_none() {
                self.onboarding(ui);
                return;
            }
            if self
                .settings
                .as_ref()
                .is_some_and(|s| s.last_seen > Month::from_date(self.today))
            {
                ui.colored_label(
                    style::warning(ui),
                    "O relógio está anterior ao histórico. Confira a data do computador.",
                );
            }
            ui.horizontal(|ui| {
                ui.heading(RichText::new(PAGES[self.page]).size(29.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.page < 3
                        && ui
                            .add(egui::Button::new("+ Novo gasto").fill(style::accent(ui)))
                            .clicked()
                    {
                        self.new_entry(if self.page == 2 { 1 } else { 0 });
                    }
                });
            });
            ui.add_space(7.0);
            ui.label(
                [
                    "Um olhar tranquilo para o seu dinheiro.",
                    "Cada gasto no seu lugar.",
                    "Organize os próximos meses.",
                    "Veja o caminho do seu dinheiro ao longo do ano.",
                    "Seu planejamento, do seu jeito.",
                    "Simples, privado e feito para durar.",
                ][self.page],
            );
            ui.add_space(22.0);
            if self.page <= 1 {
                ui.horizontal(|ui| {
                    if ui.button("<").on_hover_text("Mês anterior").clicked()
                        && self.month.0 > 24000
                    {
                        self.month.0 -= 1;
                        let r = self.refresh();
                        if let Err(e) = r {
                            self.error = Some(e.to_string());
                        }
                    }
                    ui.strong(self.month.to_string());
                    if ui.button(">").on_hover_text("Próximo mês").clicked() && self.month.0 < 25211
                    {
                        self.month.0 += 1;
                        let r = self.refresh();
                        if let Err(e) = r {
                            self.error = Some(e.to_string());
                        }
                    }
                });
                ui.add_space(15.0);
            }
            egui::ScrollArea::vertical()
                .id_salt("page-scroll")
                .show(ui, |ui| match self.page {
                    0 => self.dashboard(ui),
                    1 => self.expenses_page(ui),
                    2 => self.commitments_page(ui),
                    3 => self.report_page(ui),
                    4 => self.settings_page(ui),
                    _ => self.about_page(ui),
                });
        });
        self.entry_dialog(ctx);
        self.confirmation_dialog(ctx);
    }
    fn onboarding(&mut self, ui: &mut egui::Ui) {
        ui.add_space(45.0);
        ui.heading(RichText::new("Bem-vindo ao seu espaço financeiro").size(30.0));
        ui.add_space(15.0);
        ui.label("Comece com sua renda e o dinheiro que você já possui. Tudo será salvo neste computador.");
        ui.add_space(25.0);
        forms::field(
            ui,
            "Saldo antes do primeiro crédito (R$)",
            &mut self.opening,
        );
        forms::field(ui, "Renda mensal (R$)", &mut self.income);
        forms::field(ui, "Primeiro crédito (AAAA-MM)", &mut self.first);
        ui.small("Escolha este mês ou o próximo. Não inclua no saldo inicial o salário que será creditado agora.");
        if let (Ok(opening), Ok(income), Ok(first)) = (
            parse_money(&self.opening),
            parse_money(&self.income),
            self.first.parse::<Month>(),
        ) {
            ui.add_space(12.0);
            ui.strong(format!(
                "Saldo após começar: {}",
                money(
                    opening
                        + if first <= Month::from_date(self.today) {
                            income
                        } else {
                            0
                        }
                )
            ));
        }
        ui.add_space(20.0);
        if ui
            .add_sized([240.0, 42.0], egui::Button::new("Começar meu planejamento"))
            .clicked()
        {
            let result = (|| {
                self.store.initialize(
                    parse_money(&self.opening)?,
                    parse_money(&self.income)?,
                    self.first.parse()?,
                    self.today,
                )
            })();
            if self.finish(result) {
                self.load_preferences();
            }
        }
    }
    fn confirmation_dialog(&mut self, ctx: &egui::Context) {
        let Some(action) = self.confirm.take() else {
            return;
        };
        let mut accepted = false;
        let mut closed = false;
        let text = match &action {
            Confirmation::Expense(_) => {
                "Excluir esta despesa manual? O saldo será recalculado.".into()
            }
            Confirmation::Category(_) => {
                "Excluir esta categoria? Categorias em uso não podem ser excluídas.".into()
            }
            Confirmation::Cancel(_) => {
                "Cancelar cobranças a partir do próximo mês? O histórico será preservado.".into()
            }
            Confirmation::Restore(_, summary) => format!(
                "{summary}\n\nSubstituir todos os dados atuais? Um backup de recuperação será salvo antes da troca."
            ),
        };
        egui::Window::new("Confirmar alteração")
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(text);
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Voltar").clicked() {
                        closed = true;
                    }
                    if ui.button("Confirmar").clicked() {
                        accepted = true;
                    }
                });
            });
        if accepted {
            let result = match &action {
                Confirmation::Expense(id) => self.store.delete_expense(*id),
                Confirmation::Category(id) => self.store.delete_category(*id),
                Confirmation::Cancel(id) => self.store.cancel_commitment(*id, self.today),
                Confirmation::Restore(path, _) => self.store.restore_backup(path).map(|recovery| {
                    self.notice = Some(recovery.map_or("Backup restaurado.".into(), |p| {
                        format!("Restaurado. Recuperação: {}", p.display())
                    }));
                }),
            };
            if self.finish(result) {
                if matches!(action, Confirmation::Restore(..)) {
                    let r = self.store.reconcile(self.today);
                    self.finish(r);
                    self.load_preferences();
                }
            } else {
                self.confirm = Some(action);
            }
        } else if !closed {
            self.confirm = Some(action);
        }
    }
}
impl eframe::App for FinanceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
        if let Some(frames) = &mut self.smoke_frames {
            *frames += 1;
            if *frames >= 3 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ctx.request_repaint();
            }
        }
    }
}
