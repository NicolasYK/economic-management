use eframe::egui::{self, Color32, FontId, TextStyle};
pub fn apply(ctx: &egui::Context, theme: &str) {
    ctx.set_theme(match theme {
        "light" => egui::ThemePreference::Light,
        "dark" => egui::ThemePreference::Dark,
        _ => egui::ThemePreference::System,
    });
    ctx.options_mut(|o| o.fallback_theme = egui::Theme::Light);
    let dark = ctx.style().visuals.dark_mode;
    let key = (theme.to_owned(), dark);
    let previous = ctx.data(|d| d.get_temp::<(String, bool)>(egui::Id::new("finance-style")));
    if previous.as_ref() == Some(&key) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("finance-style"), key));
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.button_padding = egui::vec2(13.0, 9.0);
    style.spacing.interact_size.y = 34.0;
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(15.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(12.0));
    style.visuals.panel_fill = if dark {
        Color32::from_rgb(25, 32, 29)
    } else {
        Color32::from_rgb(247, 246, 240)
    };
    style.visuals.window_fill = if dark {
        Color32::from_rgb(32, 41, 36)
    } else {
        Color32::from_rgb(255, 254, 249)
    };
    style.visuals.selection.bg_fill = if dark {
        Color32::from_rgb(66, 99, 80)
    } else {
        Color32::from_rgb(193, 219, 196)
    };
    style.visuals.selection.stroke.color = if dark {
        Color32::WHITE
    } else {
        Color32::from_rgb(25, 57, 38)
    };
    style.visuals.override_text_color = Some(if dark {
        Color32::from_rgb(233, 237, 226)
    } else {
        Color32::from_rgb(35, 53, 43)
    });
    ctx.set_style(style);
}
pub fn accent(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(64, 97, 77)
    } else {
        Color32::from_rgb(193, 219, 196)
    }
}
pub fn warning(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(255, 172, 136)
    } else {
        Color32::from_rgb(159, 56, 28)
    }
}
pub fn palette(index: usize) -> Color32 {
    [
        Color32::from_rgb(82, 132, 103),
        Color32::from_rgb(196, 133, 92),
        Color32::from_rgb(96, 136, 159),
        Color32::from_rgb(167, 146, 64),
        Color32::from_rgb(157, 119, 149),
        Color32::from_rgb(104, 156, 149),
    ][index % 6]
}
pub fn card(ui: &mut egui::Ui, title: &str, value: String) {
    egui::Frame::group(ui.style())
        .fill(ui.visuals().window_fill)
        .inner_margin(18)
        .corner_radius(12)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width() - 4.0);
            ui.small(title);
            ui.add_space(7.0);
            ui.label(egui::RichText::new(value).size(24.0).strong());
        });
}
