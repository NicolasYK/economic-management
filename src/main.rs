#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use economic_management::{store::Store, ui::FinanceApp};
fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        if !std::env::args().any(|arg| arg == "--smoke-test") {
            rfd::MessageDialog::new()
                .set_title("Economic Management")
                .set_description(&message)
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
        std::process::exit(1);
    }
}
fn run() -> std::result::Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut dir = None;
    let mut smoke = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" => {
                println!("{} ({})", env!("CARGO_PKG_VERSION"), env!("BUILD_COMMIT"));
                return Ok(());
            }
            "--data-dir" => {
                dir = Some(std::path::PathBuf::from(
                    args.next().ok_or("Informe a pasta após --data-dir.")?,
                ))
            }
            "--smoke-test" => smoke = true,
            _ => {
                return Err(
                    "Opção desconhecida. Use --version, --data-dir PASTA ou --smoke-test.".into(),
                );
            }
        }
    }
    let dir = dir
        .or_else(|| {
            directories::ProjectDirs::from("", "", "EconomicManagement")
                .map(|p| p.data_local_dir().to_path_buf())
        })
        .ok_or("Não foi possível localizar a pasta de dados.")?;
    let store = Store::open(&dir.join("finance.sqlite")).map_err(|e| e.to_string())?;
    let app = FinanceApp::new(store);
    let app = if smoke { app.smoke_test() } else { app };
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 800.0])
            .with_min_inner_size([940.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Economic Management",
        options,
        Box::new(move |_| Ok(Box::new(app))),
    )
    .map_err(|error| {
        if smoke {
            eprintln!("Native graphics initialization failed: {error:?}");
        }
        "Não foi possível iniciar a interface gráfica. Verifique o driver gráfico do sistema."
            .into()
    })
}
