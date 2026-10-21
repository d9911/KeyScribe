use eframe::egui::{self, Color32, FontId, Stroke, TextStyle};

pub fn app_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/icons/keyscribe-128.png"))
        .expect("The bundled Keyscribe icon must be a valid PNG")
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub surface: Color32,
    pub soft: Color32,
    pub border: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
}

impl Theme {
    pub fn palette(self) -> Palette {
        let rgb = Color32::from_rgb;
        match self {
            Self::Light => Palette {
                background: rgb(242, 240, 235),
                surface: rgb(255, 255, 252),
                soft: rgb(237, 235, 229),
                border: rgb(213, 217, 207),
                ink: rgb(30, 57, 50),
                muted: rgb(91, 106, 98),
                accent: rgb(0, 117, 74),
                accent_soft: rgb(212, 233, 226),
                on_accent: Color32::WHITE,
            },
            Self::Dark => Palette {
                background: rgb(18, 29, 25),
                surface: rgb(26, 41, 35),
                soft: rgb(34, 51, 44),
                border: rgb(58, 79, 68),
                ink: rgb(241, 242, 233),
                muted: rgb(172, 192, 179),
                accent: rgb(137, 211, 174),
                accent_soft: rgb(39, 75, 58),
                on_accent: rgb(18, 45, 32),
            },
        }
    }

    pub fn apply(self, ctx: &egui::Context) {
        // Pin the selected style before updating it: a native theme event must
        // not switch egui back to an uncustomized system style on the next frame.
        ctx.set_theme(match self {
            Self::Light => egui::Theme::Light,
            Self::Dark => egui::Theme::Dark,
        });
        let p = self.palette();
        let mut visuals = match self {
            Self::Light => egui::Visuals::light(),
            Self::Dark => egui::Visuals::dark(),
        };
        visuals.panel_fill = p.background;
        visuals.window_fill = p.surface;
        visuals.window_stroke = Stroke::new(1.0, p.border);
        visuals.extreme_bg_color = p.background;
        visuals.text_edit_bg_color = Some(p.background);
        visuals.faint_bg_color = p.soft;
        visuals.code_bg_color = p.soft;
        visuals.weak_text_color = Some(p.muted);
        visuals.selection.bg_fill = p.accent_soft;
        visuals.selection.stroke = Stroke::new(1.5, p.accent);
        visuals.hyperlink_color = p.accent;
        visuals.error_fg_color = match self {
            Self::Light => Color32::from_rgb(169, 36, 27),
            Self::Dark => Color32::from_rgb(255, 159, 143),
        };
        visuals.window_corner_radius = 12.into();
        visuals.menu_corner_radius = 10.into();
        for widget in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.corner_radius = 8.into();
            widget.bg_fill = p.soft;
            widget.weak_bg_fill = p.soft;
            widget.bg_stroke = Stroke::new(1.0, p.border);
            widget.fg_stroke = Stroke::new(1.5, p.ink);
            widget.expansion = 0.0;
        }
        visuals.widgets.noninteractive.bg_fill = p.surface;
        visuals.widgets.hovered.bg_fill = p.accent_soft;
        visuals.widgets.hovered.weak_bg_fill = p.accent_soft;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, p.accent);
        visuals.widgets.active.bg_fill = p.accent_soft;
        visuals.widgets.active.weak_bg_fill = p.accent_soft;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, p.accent);
        ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(match self {
            Self::Light => egui::SystemTheme::Light,
            Self::Dark => egui::SystemTheme::Dark,
        }));
        ctx.set_visuals(visuals);
        ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(10.0, 10.0);
            style.spacing.button_padding = egui::vec2(13.0, 8.0);
            style.spacing.interact_size = egui::vec2(36.0, 32.0);
            style.spacing.combo_width = 160.0;
            style
                .text_styles
                .insert(TextStyle::Body, FontId::proportional(15.0));
            style
                .text_styles
                .insert(TextStyle::Button, FontId::proportional(14.0));
            style
                .text_styles
                .insert(TextStyle::Small, FontId::proportional(12.0));
            style
                .text_styles
                .insert(TextStyle::Heading, FontId::proportional(26.0));
            style
                .text_styles
                .insert(TextStyle::Monospace, FontId::monospace(14.0));
        });
    }
}

pub fn palette(ui: &egui::Ui) -> Palette {
    if ui.visuals().dark_mode {
        Theme::Dark
    } else {
        Theme::Light
    }
    .palette()
}

pub fn card(ui: &egui::Ui) -> egui::Frame {
    let p = palette(ui);
    egui::Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(12)
        .inner_margin(20)
}

pub fn section(ui: &mut egui::Ui, title: &str, caption: &str) {
    ui.label(egui::RichText::new(title).size(18.0).strong());
    if !caption.is_empty() {
        ui.label(egui::RichText::new(caption).color(palette(ui).muted));
    }
    ui.add_space(4.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_icon_decodes_with_visible_artwork_and_transparent_corners() {
        let icon = app_icon();
        assert!(icon.width > 0);
        assert_eq!(icon.width, icon.height);
        assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
        assert!(icon.rgba.chunks_exact(4).any(|pixel| pixel[3] == 255));
        for index in [
            0,
            icon.width - 1,
            icon.width * (icon.height - 1),
            icon.width * icon.height - 1,
        ] {
            assert_eq!(icon.rgba[index as usize * 4 + 3], 0);
        }
    }

    #[test]
    fn explicit_theme_survives_native_system_theme_events() {
        let ctx = egui::Context::default();
        for (choice, expected, system) in [
            (Theme::Light, egui::Theme::Light, egui::Theme::Dark),
            (Theme::Dark, egui::Theme::Dark, egui::Theme::Light),
            (Theme::Light, egui::Theme::Light, egui::Theme::Dark),
        ] {
            choice.apply(&ctx);
            let _ = ctx.run(
                egui::RawInput {
                    system_theme: Some(system),
                    ..Default::default()
                },
                |_| {},
            );
            assert_eq!(ctx.theme(), expected);
            assert_eq!(ctx.style().visuals.panel_fill, choice.palette().background);
        }
    }
}
