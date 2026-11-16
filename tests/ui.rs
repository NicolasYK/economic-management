#![cfg(feature = "desktop")]
use economic_management::{domain::Month, store::Store, ui::FinanceApp};
use eframe::egui;
fn labels(shapes: &[egui::epaint::ClippedShape]) -> String {
    fn text(s: &egui::Shape) -> String {
        match s {
            egui::Shape::Text(t) => t.galley.job.text.clone(),
            egui::Shape::Vec(v) => v.iter().map(text).collect::<Vec<_>>().join(" "),
            _ => String::new(),
        }
    }
    shapes
        .iter()
        .map(|s| text(&s.shape))
        .collect::<Vec<_>>()
        .join(" ")
}
#[test]
fn onboarding_and_all_six_pages_render_without_a_window() {
    let ctx = egui::Context::default();
    let mut app = FinanceApp::new(Store::memory().unwrap());
    let output = ctx.run(Default::default(), |ctx| app.draw(ctx));
    assert!(labels(&output.shapes).contains("Bem-vindo"));
    let mut s = Store::memory().unwrap();
    let today = chrono::Local::now().date_naive();
    s.initialize(0, 10000, Month::from_date(today), today)
        .unwrap();
    let mut app = FinanceApp::new(s);
    for (page, title) in [
        "Visão geral",
        "Despesas",
        "Recorrentes e parcelas",
        "Relatório anual",
        "Configurações",
        "Sobre",
    ]
    .iter()
    .enumerate()
    {
        app.page = page;
        let output = ctx.run(Default::default(), |ctx| app.draw(ctx));
        assert!(labels(&output.shapes).contains(title), "{title}");
    }
}
