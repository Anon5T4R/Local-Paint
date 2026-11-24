//! The command palette (the search icon of the top bar, Edit › Search): every menu command and tool, in the
//! suite's `craft-palette` box. It lists what the menus list, with the menu's own label, shortcut and enabled
//! state, and runs the item through the same code as the menu (`menus::invoke`).

use craft_palette::{Command, Labels};

use crate::LocalRasterApp;

/// How many recently run commands the palette remembers.
const RECENT_CAP: usize = 8;

/// The id of the palette's recently-run list (egui temp data: session-scoped, like the query).
fn recent_id() -> egui::Id {
    egui::Id::unique("palette-recent")
}

/// Remember `id` as the most recently run command: deduplicated, newest first, capped.
pub(crate) fn note_recent(ctx: &egui::Context, id: &str) {
    ctx.data_mut(|d| {
        let mut list: Vec<String> = d.get_temp(recent_id()).unwrap_or_default();
        push_recent(&mut list, id, RECENT_CAP);
        d.insert_temp(recent_id(), list);
    });
}

/// The palette's recently run commands, newest first (for tests).
#[cfg(test)]
pub(crate) fn recent_ids(ctx: &egui::Context) -> Vec<String> {
    ctx.data(|d| d.get_temp(recent_id()).unwrap_or_default())
}

/// [`note_recent`]'s core: move `id` to the front, dropping the oldest past `cap`.
fn push_recent(list: &mut Vec<String>, id: &str, cap: usize) {
    list.retain(|x| x != id);
    list.insert(0, id.to_string());
    list.truncate(cap);
}

/// What the palette lists: the menu commands (separators are not commands: they would list as blank rows, #2479)
/// and the tools, with the menu path as the group. The recently run commands come first, so an empty query lists
/// them (UI-217-7) and a query ranks them ahead of equal matches.
pub(crate) fn commands(app: &LocalRasterApp, recents: &[String], lang: crate::i18n::Lang) -> Vec<Command> {
    let mut seen = std::collections::HashSet::new();
    let mut list: Vec<Command> = crate::menus::menu_items(app)
        .into_iter()
        .filter(|m| m.label != "---" && seen.insert(m.id.clone()))
        .map(|m| {
            let group = m.path.iter().map(|p| crate::i18n::tr(lang, p)).collect::<Vec<_>>().join(" › ");
            let label = crate::i18n::tr_id(lang, &m.id, &m.label).trim_end_matches('…').to_string();
            let command = Command::new(m.id, label, group).enabled(m.enabled);
            match m.shortcut {
                Some(sc) => command.shortcut(crate::shortcuts::pretty(&sc)),
                None => command,
            }
        })
        .collect();
    let tool_group = crate::i18n::tr(lang, "Tool").to_string();
    list.extend(
        crate::state::Tool::ALL
            .iter()
            .map(|tool| Command::new(format!("tool:{tool:?}"), tl!(tool.label()).to_string(), tool_group.clone()).shortcut(tool.key().to_string())),
    );
    let rank = |c: &Command| recents.iter().position(|r| *r == c.id).unwrap_or(usize::MAX);
    list.sort_by_key(rank);
    list
}

/// The palette, when open: runs what the user picked.
pub fn show(app: &mut LocalRasterApp, ctx: &egui::Context) {
    // `ui.palette_open` is the flag the menus, the shortcuts and the control channel set; the box follows it.
    if app.ui.palette_open != app.palette_box.is_open() {
        if app.ui.palette_open {
            app.palette_box.open();
        } else {
            app.palette_box.close();
        }
    }
    if !app.ui.palette_open {
        return;
    }
    let lang = crate::i18n::Lang::from_pref(&app.session.prefs().interface.language);
    let recents: Vec<String> = ctx.data(|d| d.get_temp(recent_id()).unwrap_or_default());
    let list = commands(app, &recents, lang);
    let labels = Labels { placeholder: tl!("Search commands, tools and panels…"), empty: tl!("No matching commands") };
    let ran = app.palette_box.show(ctx, &labels, &list);
    app.ui.palette_open = app.palette_box.is_open();
    let Some(id) = ran else { return };
    if let Some(tool) = id.strip_prefix("tool:") {
        if let Some(t) = crate::state::Tool::from_name(tool) {
            app.ui.tool = t;
        }
    } else if let Err(e) = crate::menus::invoke(app, ctx, &id, serde_json::json!({})) {
        app.ui.status = e;
    }
}

#[cfg(test)]
mod tests {
    use egui::{Event, Key, Modifiers, vec2};
    use egui_kittest::Harness;
    use serde_json::json;

    use super::push_recent;
    use crate::LocalRasterApp;

    #[test]
    fn open_palette_interrupts_the_ime_only_when_it_takes_focus() {
        let mut app = LocalRasterApp::new(localraster_engine::Session::new(), crate::Services::default());
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        // Fonts bind on the next pass; this one opens nothing.
        ctx.run_ui(Default::default(), |_| {}).textures_delta.clear();
        app.ui.palette_open = true;
        // The field takes focus on the first pass and owns the IME from the second on.
        let interrupts: Vec<Option<bool>> = (0..4)
            .map(|_| {
                let mut out = ctx.run_ui(Default::default(), |ui| super::show(&mut app, ui.ctx()));
                out.textures_delta.clear();
                out.platform_output.ime.map(|ime| ime.should_interrupt_composition)
            })
            .collect();
        assert_eq!(interrupts, [None, Some(false), Some(false), Some(false)]);
    }

    #[test]
    fn push_recent_moves_to_the_front_and_caps() {
        let mut l = Vec::new();
        push_recent(&mut l, "b", 3);
        push_recent(&mut l, "a", 3);
        push_recent(&mut l, "c", 3);
        assert_eq!(l, ["c", "a", "b"]);
        push_recent(&mut l, "a", 3);
        assert_eq!(l, ["a", "c", "b"], "re-running a command moves it to the front");
        push_recent(&mut l, "d", 3);
        assert_eq!(l, ["d", "a", "c"], "the oldest falls off the cap");
    }

    #[test]
    fn the_palette_records_and_replays_recent_commands_on_an_empty_query() {
        let mut h = Harness::builder().with_size(vec2(1200.0, 800.0)).with_max_steps(64).build_eframe(|cc| {
            LocalRasterApp::setup_context(&cc.egui_ctx, Default::default());
            LocalRasterApp::new(localraster_engine::Session::new(), crate::Services::default())
        });
        h.state_mut().run("file.new", json!({"width": 64, "height": 64})).unwrap();
        h.state_mut().sync_views();
        let ctx = h.ctx.clone();
        // Run "Keyboard Shortcuts" through the palette: it becomes the recent command.
        crate::menus::invoke(h.state_mut(), &ctx, "edit.search", json!({})).unwrap();
        h.run_steps(2);
        h.event(Event::Text("keyboard shortcuts".into()));
        h.run_steps(2);
        h.event(Event::Key { key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
        h.run_steps(3);
        assert_eq!(super::recent_ids(&ctx).len(), 1, "the command the palette ran is remembered");
        // Close its dialog, reopen the palette with an empty query and press Enter: the recent
        // command is the first hit, so the same dialog opens again.
        let dialog = h
            .state()
            .ui
            .dialogs
            .iter()
            .find(|d| d.fields.get("__prefsui").and_then(|v| v.as_str()) == Some("shortcuts"))
            .map(|d| d.id)
            .expect("the shortcuts dialog");
        h.state_mut().ui.close_dialog(dialog);
        h.run_steps(2);
        crate::menus::invoke(h.state_mut(), &ctx, "edit.search", json!({})).unwrap();
        h.run_steps(2);
        h.event(Event::Key { key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
        h.run_steps(3);
        assert!(
            h.state().ui.dialogs.iter().any(|d| d.fields.get("__prefsui").and_then(|v| v.as_str()) == Some("shortcuts")),
            "the empty-query palette runs the recent command"
        );
        assert_eq!(super::recent_ids(&ctx).len(), 1, "re-running it moves it to the front, no duplicate");
    }

    /// #2479: menu separators are not commands: they matched on their menu path and filled the palette with blank
    /// "---" rows. Every row the palette lists has a name, a menu and the menu's own enabled state.
    #[test]
    fn separators_are_not_palette_commands() {
        let app = LocalRasterApp::new(localraster_engine::Session::new(), crate::Services::default());
        assert!(crate::menus::menu_items(&app).iter().any(|m| m.label == "---"), "the menus have separators to filter");
        let list = super::commands(&app, &[], crate::i18n::Lang::EN);
        assert!(!list.is_empty());
        assert!(list.iter().all(|c| c.id != "---" && c.label != "---" && !c.label.is_empty()));
        let mut ids: Vec<&str> = list.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), list.len(), "one row per command");
    }

    /// The palette lists what the menus list, with the menu path as the group, the menu's shortcut and the menu's
    /// enabled state, and the tools next to them; the recent commands come first.
    #[test]
    fn the_palette_lists_the_menus_the_shortcut_and_the_enabled_state() {
        let app = LocalRasterApp::new(localraster_engine::Session::new(), crate::Services::default());
        let list = super::commands(&app, &[], crate::i18n::Lang::EN);
        let find = |id: &str| list.iter().find(|c| c.id == id).unwrap_or_else(|| panic!("{id} is listed"));
        let settings = find("edit.preferences.general");
        assert_eq!((settings.label.as_str(), settings.group.as_str()), ("Settings", "Edit › Settings"));
        assert!(!settings.shortcut.is_empty(), "the menu's shortcut shows");
        assert!(settings.enabled);
        assert!(!find("file.save").enabled, "Save is disabled with no document, as in the menu");
        assert!(find("help.about").enabled && find("file.exit").enabled);
        let _ = find("help.checkForUpdates");
        assert!(list.iter().any(|c| c.id.starts_with("tool:")), "the tools are listed");
        let first = super::commands(&app, &["help.about".to_string()], crate::i18n::Lang::EN);
        assert_eq!(first.first().map(|c| c.id.as_str()), Some("help.about"), "a recent command leads");
    }

    /// #536: Edit › Search → "Keyboard Shortcuts" must open the dialog, with and without a document.
    #[test]
    fn palette_opens_the_keyboard_shortcuts_dialog() {
        for with_doc in [false, true] {
            let mut h = Harness::builder().with_size(vec2(1200.0, 800.0)).with_max_steps(64).build_eframe(|cc| {
                LocalRasterApp::setup_context(&cc.egui_ctx, Default::default());
                LocalRasterApp::new(localraster_engine::Session::new(), crate::Services::default())
            });
            if with_doc {
                h.state_mut().run("file.new", json!({"width": 64, "height": 64})).unwrap();
                h.state_mut().sync_views();
            }
            let ctx = h.ctx.clone();
            crate::menus::invoke(h.state_mut(), &ctx, "edit.search", json!({})).unwrap();
            h.run_steps(2);
            assert!(h.state().ui.palette_open, "Edit > Search opens the palette (with_doc={with_doc})");
            h.event(Event::Text("keyboard shortcuts".into()));
            h.run_steps(2);
            h.event(Event::Key { key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
            h.run_steps(3);
            let app = h.state();
            assert!(!app.ui.palette_open, "Enter closes the palette (with_doc={with_doc})");
            let dialog = app
                .ui
                .dialogs
                .iter()
                .find(|d| d.fields.get("__prefsui").and_then(|v| v.as_str()) == Some("shortcuts"))
                .map(|d| d.id)
                .unwrap_or_else(|| panic!("#536: the Keyboard Shortcuts dialog did not open (with_doc={with_doc})"));
            // The dialog keeps its own Enter: a second press is its OK and closes it.
            h.event(Event::Key { key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
            h.run_steps(3);
            assert!(h.state().ui.dialogs.iter().all(|d| d.id != dialog), "a second Enter closes the dialog as its OK (with_doc={with_doc})");
        }
    }
}
