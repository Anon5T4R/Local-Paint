//! The kittest width guard: with a document on screen, every dialog and window of the app fits
//! the main window's minimum width (`with_min_inner_size` in `apps/localpaint/src/main.rs`), and
//! so do the top bar, the command palette and the update card.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::NodeT;
use serde_json::json;

use crate::LocalPaintApp;

/// The main window's minimum width.
const MIN_WIDTH: f32 = 760.0;
/// Camera Raw opens its own OS window, whose minimum width it declares itself.
const CAMERA_RAW_MIN_WIDTH: f32 = 900.0;

/// An app with a document with real content: a layer filled grey over a blue background.
fn app_with_content() -> LocalPaintApp {
    let mut app = LocalPaintApp::new(localpaint_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 64, "height": 64, "background": "#3366cc"})).unwrap();
    app.run("layer.new.layer", json!({})).unwrap();
    app.run("edit.fill", json!({"color": "#808080"})).unwrap();
    app.sync_views();
    app
}

/// A harness at `width` running the real eframe app, with the dialog `open`ed inside the creator
/// (after `setup_context`, so the dialog's first frame already has the app's fonts).
fn harness_at(width: f32, open: impl FnOnce(&mut LocalPaintApp, &egui::Context)) -> Harness<'static, LocalPaintApp> {
    let mut h = Harness::builder().with_size(egui::vec2(width, 800.0)).with_max_steps(64).build_eframe(|cc| {
        LocalPaintApp::setup_context(&cc.egui_ctx, crate::theme::ThemeKind::ProMedium);
        let mut app = app_with_content();
        open(&mut app, &cc.egui_ctx);
        app
    });
    h.run_steps(4);
    h
}

/// Every node, in accesskit coordinates, inside the window of `width` points. A text run may
/// report the galley's full width before the clip egui paints; the field's node is the guard.
fn assert_inside(harness: &Harness<'_, LocalPaintApp>, what: &str, width: f32) {
    for node in harness.root().children_recursive() {
        if node.accesskit_node().role() == Role::TextRun {
            continue;
        }
        let Some(rect) = node.accesskit_node().bounding_box() else { continue };
        if rect.width() > 0.0 {
            assert!(
                rect.x0 >= -0.5 && rect.x1 <= f64::from(width) + 0.5,
                "{what}: {role:?} outside the {width}-point window: {rect:?}",
                role = node.accesskit_node().role()
            );
        }
    }
}

/// The commands that open a dialog of their own, derived from the engine's catalogue: the
/// adjustment editors, the filter/schema dialogs, the sizing dialogs and the modules with a
/// custom dialog behind a command id.
fn dialog_commands() -> Vec<(&'static str, &'static str)> {
    localpaint_engine::commands::command_specs()
        .iter()
        .filter(|spec| {
            let id = spec.id;
            crate::filter_dialog::has_dialog(id)
                || id.strip_prefix("image.adjustments.").is_some_and(crate::adjust_editors::has_editor)
                || crate::sizing::is_sizing(id)
                || crate::new_group_ui::fronts(id)
                || matches!(
                    id,
                    crate::stroke_ui::COMMAND
                        | crate::color_range_ui::COMMAND
                        | crate::fill_ui::COMMAND
                        | crate::new_layer_ui::COMMAND
                        | crate::snapshot_dialog::COMMAND
                        | crate::stack_dialog::COMMAND
                        | crate::export_dialog::COMMAND
                        | crate::channel_options_ui::COMMAND
                        | crate::history_options_ui::COMMAND
                        | crate::smart_filter_dialog::COMMAND
                )
        })
        .map(|spec| (spec.id, spec.label))
        .collect()
}

/// Every command dialog, the four dialog kinds and the special windows fit the minimum width.
#[test]
fn every_dialog_fits_the_narrow_window() {
    for (id, label) in dialog_commands() {
        let h = harness_at(MIN_WIDTH, |app, _| {
            crate::dialogs::open_command_dialog(app, id, label);
        });
        assert!(!h.state().ui.dialogs.is_empty(), "{id} opened no dialog");
        assert_inside(&h, id, MIN_WIDTH);
    }
    type Open = fn(&mut LocalPaintApp, &egui::Context);
    let windows: [(&str, Open); 11] = [
        ("new-document", |app, _| {
            let fields = app.new_document_fields();
            app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
        }),
        ("about", |app, _| {
            app.ui.open_dialog(crate::state::DialogKind::About, Default::default());
        }),
        ("settings", |app, _| {
            crate::prefs_ui::open_preferences(app, "interface");
        }),
        ("layer-style", |app, _| {
            crate::layer_style::open(app, None).expect("a layer to style");
        }),
        ("color-picker", |app, _| {
            crate::color_picker_ui::open(app, "foreground");
        }),
        ("brush-preset", |app, _| {
            crate::brush_preset_dialog::open(app, None);
        }),
        // The unsaved-changes prompt: the filled layer above is the unsaved work.
        ("discard", |app, _| {
            assert!(crate::discard_ui::intercept(app, "file.close", &json!({})));
        }),
        ("filter-gallery", |app, _| {
            crate::gallery_ui::open(app).unwrap();
        }),
        ("liquify", |app, ctx| {
            crate::liquify_ui::open(app, ctx).unwrap();
        }),
        ("select-and-mask", |app, _| {
            // Select and Mask needs a selection: take the whole canvas.
            app.run("select.all", serde_json::json!({})).unwrap();
            crate::select_mask_ui::open(app).unwrap();
        }),
        ("palette", |app, _| {
            app.ui.palette_open = true;
        }),
    ];
    for (name, open) in windows {
        let h = harness_at(MIN_WIDTH, open);
        #[cfg(debug_assertions)]
        if std::env::var_os("NARROW_DEBUG").is_some() {
            for node in h.root().children_recursive() {
                if let Some(rect) = node.accesskit_node().bounding_box()
                    && rect.x1 > f64::from(MIN_WIDTH) + 0.5
                {
                    eprintln!("{name}: {:?} {rect:?}", node.accesskit_node().role());
                }
            }
        }
        let shown = match name {
            "settings" => h.state().ui.dialogs.iter().any(|d| crate::prefs_ui::is_preferences(&d.fields)),
            "discard" => h.state().discard.is_some(),
            "palette" => h.state().ui.palette_open,
            "layer-style" => h.state().ui.dialogs.iter().any(|d| d.kind == crate::state::DialogKind::LayerStyle),
            "filter-gallery" => h.state().distort.gallery.is_some(),
            "liquify" => h.state().distort.liquify.is_some(),
            "select-and-mask" => h.state().distort.select_mask.is_some(),
            _ => !h.state().ui.dialogs.is_empty(),
        };
        assert!(shown, "{name} is on screen");
        assert_inside(&h, name, MIN_WIDTH);
    }
    // Wide Angle needs a filter target the plain fixture lacks; give the layer a smart filter
    // preview path by opening it over the filled layer through its command.
    let h = harness_at(MIN_WIDTH, |app, ctx| {
        crate::wide_angle_ui::open(app, ctx).unwrap();
        assert!(app.wide_angle.is_some(), "wide angle is open");
    });
    assert_inside(&h, "wide-angle", MIN_WIDTH);

    // Camera Raw lives in its own OS window, whose minimum width is its own.
    let h = harness_at(CAMERA_RAW_MIN_WIDTH, |app, ctx| {
        crate::camera_raw_ui::open(app, ctx).unwrap();
    });
    assert!(crate::control::inspect(h.state(), &h.ctx)["cameraRaw"].is_object(), "camera raw is open");
    assert_inside(&h, "camera-raw", CAMERA_RAW_MIN_WIDTH);
}

/// The top bar and the update card fit the minimum width too.
#[test]
fn the_top_bar_and_the_update_card_fit_the_narrow_window() {
    let h = harness_at(MIN_WIDTH, |_, _| {});
    assert_inside(&h, "top-bar", MIN_WIDTH);

    let release =
        localpaint_engine::updates_cmds::Release { version: "99.0.0".into(), url: "https://github.com/Anon5T4R/Local-Paint/releases/tag/v99.0.0".into() };
    let mut h = harness_at(MIN_WIDTH, |app, _| {
        app.session.updates.status = localpaint_engine::updates_cmds::Status::Available { release };
        crate::updates_ui::tick(app, &egui::Context::default());
        assert!(app.ui.updates_open, "the available release opened the update card");
    });
    h.run_steps(4);
    assert_inside(&h, "update-card", MIN_WIDTH);
}
