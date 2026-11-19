//! The suite's shell in the top bar: the Settings gear, the theme button and the command palette's search icon,
//! the Window › Theme menu with the same three modes, and Edit › Settings….

use egui::{Key, Modifiers, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use localpaint_engine::prefs::AppearanceMode;
use serde_json::json;

use crate::LocalPaintApp;

fn harness() -> Harness<'static, LocalPaintApp> {
    sized(vec2(1280.0, 800.0))
}

fn sized(size: egui::Vec2) -> Harness<'static, LocalPaintApp> {
    let mut h = Harness::builder().with_size(size).with_max_steps(64).build_eframe(|cc| {
        LocalPaintApp::setup_context(&cc.egui_ctx, Default::default());
        let mut app = LocalPaintApp::new(localpaint_engine::Session::new(), crate::Services::default());
        app.run("prefs.set", json!({"values": {"interface.language": "en", "interface.appearanceMode": "auto"}})).unwrap();
        app.run("file.new", json!({"width": 64, "height": 64})).unwrap();
        app.sync_views();
        app
    });
    h.run_steps(4);
    h
}

fn mode(h: &Harness<'_, LocalPaintApp>) -> AppearanceMode {
    h.state().session.prefs().interface.appearance_mode
}

fn settings_dialog(h: &Harness<'_, LocalPaintApp>) -> Option<(String, String)> {
    h.state().ui.dialogs.iter().find(|d| crate::prefs_ui::is_preferences(&d.fields)).map(|d| {
        let label = d.fields.get("__label").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let section = d.fields.get("section").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        (label, section)
    })
}

/// From the right edge: the gear, the theme button, the search icon, each 6 apart, off the edge.
#[test]
fn the_top_bar_has_the_gear_the_theme_and_the_search_in_the_suites_order() {
    let h = harness();
    let gear = h.get_by_label("Settings").rect();
    let theme = h.get_by_label("Theme: Automatic").rect();
    let search = h.get_by_label_contains("Search commands").rect();
    assert!(gear.left() > theme.right() && theme.left() > search.right(), "gear {gear:?}, theme {theme:?}, search {search:?}");
    assert!(1280.0 - gear.right() >= 8.0, "the gear is off the window's edge: {gear:?}");
    let gap = |a: egui::Rect, b: egui::Rect| a.left() - b.right();
    assert!((gap(gear, theme) - 6.0).abs() < 8.0 && (gap(theme, search) - 6.0).abs() < 8.0, "{gear:?} {theme:?} {search:?}");
}

/// The theme button goes Automatic, Light, Dark, Automatic, and the tooltip names the mode it shows.
#[test]
fn the_theme_button_cycles_automatic_light_dark() {
    let mut h = harness();
    assert_eq!(mode(&h), AppearanceMode::Auto);
    for (label, next) in [("Theme: Automatic", AppearanceMode::Light), ("Theme: Light", AppearanceMode::Dark), ("Theme: Dark", AppearanceMode::Auto)] {
        h.get_by_label(label).click();
        h.run_steps(3);
        assert_eq!(mode(&h), next, "after {label}");
    }
}

/// Window › Theme lists the same three modes, with the current one checked, and runs them.
#[test]
fn the_window_theme_menu_offers_the_three_modes() {
    let mut h = harness();
    let items = crate::menus::menu_items(h.state());
    let by = |id: &str| items.iter().find(|i| i.id == id).unwrap_or_else(|| panic!("{id}"));
    let modes = ["window.theme.system", "window.theme.light", "window.theme.dark"];
    assert_eq!(modes.map(|id| by(id).label.as_str()), ["Automatic", "Light", "Dark"]);
    assert!(modes.iter().all(|id| by(id).path == ["Window", "Theme"] && by(id).enabled));
    assert_eq!(modes.map(|id| by(id).checked), [Some(true), Some(false), Some(false)]);
    // The order of the menu is the order of the button.
    let at = |id: &str| items.iter().position(|i| i.id == id).unwrap();
    assert!(at(modes[0]) < at(modes[1]) && at(modes[1]) < at(modes[2]));

    let ctx = h.ctx.clone();
    for (id, expected) in
        [("window.theme.light", AppearanceMode::Light), ("window.theme.dark", AppearanceMode::Dark), ("window.theme.system", AppearanceMode::Auto)]
    {
        crate::menus::invoke(h.state_mut(), &ctx, id, json!({})).unwrap();
        h.run_steps(2);
        assert_eq!(mode(&h), expected, "{id}");
        let items = crate::menus::menu_items(h.state());
        let checked: Vec<Option<bool>> = modes.iter().map(|m| items.iter().find(|i| i.id == *m).and_then(|i| i.checked)).collect();
        assert_eq!(checked.iter().filter(|c| **c == Some(true)).count(), 1, "{id}: {checked:?}");
        if expected != AppearanceMode::Auto {
            assert_eq!(h.state().ui.theme.is_light(), expected == AppearanceMode::Light, "{id} shows its theme");
        }
    }
}

/// Window › Theme › Dark, driven the way a keyboard user does: Window opens, ↓ to Theme, → into it, ↓ to Dark, ↩.
#[test]
fn choosing_window_theme_dark_in_the_menu_turns_the_theme_dark() {
    use crate::menu_nav::Nav;
    // Tall enough for the whole Window menu: no scrolling.
    let mut h = sized(vec2(1280.0, 1400.0));
    let ctx = h.ctx.clone();
    crate::prefs_ui::set_appearance(h.state_mut(), &ctx, AppearanceMode::Light);
    h.run_steps(3);
    h.get_by_label("Window").click();
    h.run_steps(4);
    let key = |h: &mut Harness<'static, LocalPaintApp>, k: Key| {
        h.key_press(k);
        h.run_steps(3);
    };
    // Down to the Theme row (its rectangle is the accessible "Theme" node's).
    let theme = h.get_by_label("Theme").rect();
    for _ in 0..Nav::current(&h.ctx).rows[0].len() {
        let nav = Nav::current(&h.ctx);
        if nav.highlighted(0).and_then(|i| nav.rows[0].get(i)).is_some_and(|r| theme.contains(r.rect.center())) {
            break;
        }
        key(&mut h, Key::ArrowDown);
    }
    key(&mut h, Key::ArrowRight);
    let nav = Nav::current(&h.ctx);
    assert_eq!(
        nav.level,
        1,
        "→ opens Theme: theme {theme:?} highlighted {:?} rows {:?}",
        nav.highlighted(0),
        nav.rows[0].iter().map(|r| (r.rect.center().y, r.command.clone())).collect::<Vec<_>>()
    );
    // Down to Dark (a row whose rectangle holds the "Dark" label, not the "Dark Theme" submenu).
    let dark = h.get_by_label("Dark").rect();
    for _ in 0..Nav::current(&h.ctx).rows[1].len() {
        let nav = Nav::current(&h.ctx);
        if nav.highlighted(1).and_then(|i| nav.rows[1].get(i)).is_some_and(|r| dark.contains(r.rect.center())) {
            break;
        }
        key(&mut h, Key::ArrowDown);
    }
    key(&mut h, Key::Enter);
    assert_eq!(mode(&h), AppearanceMode::Dark);
    assert!(!h.state().ui.theme.is_light());
}

/// The gear opens Settings on the Interface section, where the language and the theme are; the screen's name is
/// Settings.
#[test]
fn the_gear_opens_settings_on_the_interface_section() {
    let mut h = harness();
    assert!(settings_dialog(&h).is_none());
    h.get_by_label("Settings").click();
    h.run_steps(4);
    assert_eq!(settings_dialog(&h), Some(("Settings".to_string(), "interface".to_string())));
    // The language is inside it.
    assert!(h.query_by_label("Interface").is_some() || h.query_by_label_contains("Language").is_some(), "the Interface section shows");
}

/// Edit › Settings › Settings… (Ctrl+K) opens the same screen; the old "Preferences" name is gone from the menu.
#[test]
fn edit_settings_opens_settings_and_ctrl_k_stays_the_settings_shortcut() {
    let mut h = harness();
    let items = crate::menus::menu_items(h.state());
    let general = items.iter().find(|i| i.id == "edit.preferences.general").unwrap();
    assert_eq!((general.label.as_str(), general.path.as_slice()), ("Settings…", ["Edit".to_string(), "Settings".to_string()].as_slice()));
    assert!(items.iter().all(|i| !i.path.iter().any(|p| p == "Preferences")), "no menu is still called Preferences");
    let ctx = h.ctx.clone();
    crate::menus::invoke(h.state_mut(), &ctx, "edit.preferences.general", json!({})).unwrap();
    h.run_steps(3);
    assert_eq!(settings_dialog(&h).map(|d| d.0), Some("Settings".to_string()));
}

#[test]
fn ctrl_k_opens_settings_not_the_palette() {
    let mut h = harness();
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(4);
    assert!(settings_dialog(&h).is_some(), "Ctrl+K opens Settings");
    assert!(!h.state().ui.palette_open, "the palette opens from the search icon only");
}

/// The search icon opens the palette with every menu command; typing narrows it and Enter runs the command through
/// the menu's own code.
#[test]
fn the_search_icon_opens_the_palette_and_runs_a_command() {
    let mut h = harness();
    assert!(!h.state().ui.palette_open);
    h.get_by_label_contains("Search commands").click();
    h.run_steps(3);
    assert!(h.state().ui.palette_open && h.state().palette_box.is_open());
    h.event(egui::Event::Text("about local".into()));
    h.run_steps(3);
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(!h.state().ui.palette_open, "running a command closes the palette");
    assert!(h.state().ui.dialogs.iter().any(|d| d.kind == crate::state::DialogKind::About), "About ran as the Help menu item does");
}

/// A disabled menu item is listed but does not run, as in the menu.
#[test]
fn a_command_the_menu_disables_does_not_run_from_the_palette() {
    let h = harness();
    let list = crate::palette::commands(h.state(), &[], crate::i18n::Lang::EN);
    let disabled = list.iter().find(|c| !c.enabled).expect("a command the menu disables with no selection");
    assert!(!crate::menus::menu_items(h.state()).iter().find(|i| i.id == disabled.id).is_some_and(|i| i.enabled));
}

/// Screenshots of the top bar for review: `cargo test -p localpaint-ui-egui casca_shots -- --ignored`.
/// They land in `target/qa-casca/`.
#[test]
#[ignore = "QA: writes PNGs of the top bar to target/qa-casca"]
fn casca_shots() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/qa-casca");
    std::fs::create_dir_all(&directory).unwrap();
    let shot = |h: &mut Harness<'_, LocalPaintApp>, name: &str| {
        h.run_steps(4);
        let image = h.render().unwrap();
        image.save(directory.join(format!("{name}.png"))).unwrap();
    };
    let mut h = harness();
    let ctx = h.ctx.clone();
    crate::prefs_ui::set_appearance(h.state_mut(), &ctx, AppearanceMode::Light);
    shot(&mut h, "bar-claro");
    crate::prefs_ui::set_appearance(h.state_mut(), &ctx, AppearanceMode::Dark);
    shot(&mut h, "bar-escuro");
    crate::prefs_ui::open_preferences(h.state_mut(), "interface");
    shot(&mut h, "configuracoes-escuro");
    let id = h.state().ui.dialogs.iter().find(|d| crate::prefs_ui::is_preferences(&d.fields)).map(|d| d.id).unwrap();
    h.state_mut().ui.close_dialog(id);
    crate::prefs_ui::set_appearance(h.state_mut(), &ctx, AppearanceMode::Light);
    h.run_steps(3);
    h.state_mut().ui.palette_open = true;
    h.run_steps(3);
    h.event(egui::Event::Text("sett".into()));
    shot(&mut h, "paleta-claro");
    h.state_mut().ui.palette_open = false;
    crate::prefs_ui::set_appearance(h.state_mut(), &ctx, AppearanceMode::Dark);
    h.run_steps(3);
    h.state_mut().ui.palette_open = true;
    h.run_steps(3);
    shot(&mut h, "paleta-escuro");
}
