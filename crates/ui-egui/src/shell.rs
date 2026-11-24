//! The suite's shell in the top bar: the Settings gear, the theme button and the command palette's search icon, drawn
//! with the suite's `craft-icons`. What each does is the app's own command (Edit › Settings…, the appearance mode in
//! the preferences, Edit › Search…), so the buttons, the menus and the control channel agree.

use craft_icons::{Icon, ThemeMode};
use egui::{Response, Role, Ui, WidgetInfo};
use localraster_engine::prefs::AppearanceMode;

use crate::LocalRasterApp;

/// The suite's mode for Local Raster's appearance setting (the ids on disk are unchanged: auto, light, dark).
pub fn theme_mode(mode: AppearanceMode) -> ThemeMode {
    match mode {
        AppearanceMode::Auto => ThemeMode::System,
        AppearanceMode::Light => ThemeMode::Light,
        AppearanceMode::Dark => ThemeMode::Dark,
    }
}

/// The appearance setting a suite mode stands for.
pub fn appearance_mode(mode: ThemeMode) -> AppearanceMode {
    match mode {
        ThemeMode::System => AppearanceMode::Auto,
        ThemeMode::Light => AppearanceMode::Light,
        ThemeMode::Dark => AppearanceMode::Dark,
    }
}

/// "Theme: Automatic": the theme button's tooltip, translated.
pub fn theme_tip(mode: ThemeMode) -> String {
    format!("{}: {}", tl!("Theme"), tl!(mode.label()))
}

/// One suite icon button: its tooltip is also the control's accessible name.
fn button(ui: &mut Ui, icon: Icon, tip: &str) -> Response {
    let response = craft_icons::ui::button(ui, icon, tip);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, tip));
    response
}

/// The gear: opens Settings on the Interface section (the theme and the language live there).
pub fn settings_button(app: &mut LocalRasterApp, ui: &mut Ui) {
    if button(ui, Icon::Settings, tl!("Settings")).clicked() {
        crate::prefs_ui::open_preferences(app, "interface");
    }
}

/// The theme button: shows the mode and moves to the next (Automatic, Light, Dark); the right-click menu keeps
/// Appearance Settings… and the favourite themes.
pub fn theme_button(app: &mut LocalRasterApp, ui: &mut Ui) {
    let mode = theme_mode(app.session.prefs().interface.appearance_mode);
    let response = button(ui, mode.icon(), &theme_tip(mode));
    if response.clicked() {
        crate::prefs_ui::set_appearance(app, ui.ctx(), appearance_mode(mode.next()));
    }
    response.context_menu(|ui| crate::prefs_ui::appearance_context_menu(app, ui));
}

/// The search icon: opens the command palette (Edit › Search…).
pub fn search_button(app: &mut LocalRasterApp, ui: &mut Ui) {
    let tip = crate::shortcuts::tip_label(app, "Search commands", "edit.search");
    if button(ui, Icon::Search, &tip).clicked() {
        app.ui.palette_open = !app.ui.palette_open;
    }
}

/// The suite's close X for panel headers and the app-drawn dialog title bars, with the tooltip as its accessible
/// name too.
pub fn close_button(ui: &mut Ui, tip: &str) -> Response {
    let response = craft_icons::ui::close_button(ui, tip);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, tip));
    response
}
