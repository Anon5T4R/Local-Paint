//! The kittest size guard: with a document on screen, every dialog and window of the app fits the
//! main window's minimum size (`with_min_inner_size` in `apps/localraster/src/main.rs`), and so do
//! the top bar, the command palette and the update card.

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::NodeT;
use serde_json::json;

use crate::LocalRasterApp;

/// The main window's minimum width and height.
const MIN_WIDTH: f32 = 760.0;
const MIN_HEIGHT: f32 = 480.0;
/// Camera Raw opens its own OS window, whose minimum size it declares itself.
const CAMERA_RAW_MIN_WIDTH: f32 = 900.0;
const CAMERA_RAW_MIN_HEIGHT: f32 = 600.0;

/// An app with a document with real content: a layer filled grey over a blue background.
fn app_with_content() -> LocalRasterApp {
    let mut app = LocalRasterApp::new(localraster_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 64, "height": 64, "background": "#3366cc"})).unwrap();
    app.run("layer.new.layer", json!({})).unwrap();
    app.run("edit.fill", json!({"color": "#808080"})).unwrap();
    app.sync_views();
    app
}

/// A harness at `width`×[`MIN_HEIGHT`] running the real eframe app, with the dialog `open`ed inside
/// the creator (after `setup_context`, so the dialog's first frame already has the app's fonts).
fn harness_at(width: f32, open: impl FnOnce(&mut LocalRasterApp, &egui::Context)) -> Harness<'static, LocalRasterApp> {
    harness_sized(width, MIN_HEIGHT, open)
}

/// [`harness_at`] at any size, for the window that declares a minimum of its own.
fn harness_sized(width: f32, height: f32, open: impl FnOnce(&mut LocalRasterApp, &egui::Context)) -> Harness<'static, LocalRasterApp> {
    let mut h = Harness::builder().with_size(egui::vec2(width, height)).with_max_steps(64).build_eframe(|cc| {
        LocalRasterApp::setup_context(&cc.egui_ctx, crate::theme::ThemeKind::ProMedium);
        let mut app = app_with_content();
        open(&mut app, &cc.egui_ctx);
        app
    });
    h.run_steps(4);
    h
}

/// Where a window's content may go past the bottom edge: the body of a dialog whose controls do not
/// fit scrolls, with its title and its buttons outside the `ScrollArea`. The accesskit tree reports
/// the scrolled content unclipped, so what hangs below is accepted only when it comes before a
/// vertical scroll bar inside the window and in its column; the bar is drawn after the content and
/// the buttons after it.
fn below_is_scrolled(harness: &Harness<'_, LocalRasterApp>, width: f32, height: f32) -> impl Fn(usize, egui::accesskit::Rect) -> bool {
    const TOL: f64 = 0.5;
    let (w, h) = (f64::from(width), f64::from(height));
    let nodes: Vec<_> = harness.root().children_recursive().collect();
    let bars: Vec<(usize, egui::accesskit::Rect)> = nodes
        .iter()
        .enumerate()
        .filter_map(|(i, node)| {
            let rect = node.accesskit_node().bounding_box()?;
            let vertical = rect.y1 - rect.y0 > rect.x1 - rect.x0;
            let inside = rect.x0 >= -TOL && rect.x1 <= w + TOL && rect.y0 >= -TOL && rect.y1 <= h + TOL;
            (node.accesskit_node().role() == Role::ScrollBar && vertical && inside).then_some((i, rect))
        })
        .collect();
    move |i, rect| bars.iter().any(|&(bar_at, bar)| bar_at > i && rect.x1 <= bar.x1 + TOL && bar.y0 <= rect.y0 + TOL)
}

/// Every node, in accesskit coordinates, inside the window of `width`×`height` points, in both axes.
/// A text run may report the galley's full width before the clip egui paints; the field's node is
/// the guard.
fn assert_inside(harness: &Harness<'_, LocalRasterApp>, what: &str, width: f32, height: f32) {
    let scrolled = below_is_scrolled(harness, width, height);
    let (w, h) = (f64::from(width), f64::from(height));
    for (i, node) in harness.root().children_recursive().enumerate() {
        if node.accesskit_node().role() == Role::TextRun {
            continue;
        }
        let Some(rect) = node.accesskit_node().bounding_box() else { continue };
        if rect.width() <= 0.0 {
            continue;
        }
        let role = node.accesskit_node().role();
        assert!(rect.x0 >= -0.5 && rect.x1 <= w + 0.5, "{what}: {role:?} outside the {width}-point window: {rect:?}");
        assert!(rect.y0 >= -0.5, "{what}: {role:?} above the window: {rect:?}");
        if rect.y1 > h + 0.5 && !scrolled(i, rect) {
            panic!("{what}: {role:?} below the {height}-point window and outside a scrolling body: {rect:?}");
        }
    }
}

/// The commands that open a dialog of their own, derived from the engine's catalogue: the
/// adjustment editors, the filter/schema dialogs, the sizing dialogs and the modules with a
/// custom dialog behind a command id.
fn dialog_commands() -> Vec<(&'static str, &'static str)> {
    localraster_engine::commands::command_specs()
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

/// Every command dialog, the four dialog kinds and the special windows fit the minimum size. One
/// failure is reported with the rest instead of hiding them: the whole list comes out of one run.
#[test]
fn every_dialog_fits_the_narrow_window() {
    let mut problems: Vec<String> = Vec::new();
    for (id, label) in dialog_commands() {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let h = harness_at(MIN_WIDTH, |app, _| {
                crate::dialogs::open_command_dialog(app, id, label);
            });
            assert!(!h.state().ui.dialogs.is_empty(), "{id} opened no dialog");
            assert_inside(&h, id, MIN_WIDTH, MIN_HEIGHT);
        }));
        if let Err(panic) = outcome {
            let message = panic.downcast_ref::<String>().map(String::as_str).or(panic.downcast_ref::<&str>().copied()).unwrap_or("?");
            problems.push(format!("{id}: {message}"));
        }
    }
    type Open = fn(&mut LocalRasterApp, &egui::Context);
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
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
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
            assert_inside(&h, name, MIN_WIDTH, MIN_HEIGHT);
        }));
        if let Err(panic) = outcome {
            let message = panic.downcast_ref::<String>().map(String::as_str).or(panic.downcast_ref::<&str>().copied()).unwrap_or("?");
            problems.push(format!("{name}: {message}"));
        }
    }
    // Wide Angle needs a filter target the plain fixture lacks; give the layer a smart filter
    // preview path by opening it over the filled layer through its command.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let h = harness_at(MIN_WIDTH, |app, ctx| {
            crate::wide_angle_ui::open(app, ctx).unwrap();
            assert!(app.wide_angle.is_some(), "wide angle is open");
        });
        assert_inside(&h, "wide-angle", MIN_WIDTH, MIN_HEIGHT);
    }));
    if let Err(panic) = outcome {
        let message = panic.downcast_ref::<String>().map(String::as_str).or(panic.downcast_ref::<&str>().copied()).unwrap_or("?");
        problems.push(format!("wide-angle: {message}"));
    }

    // Camera Raw lives in its own OS window, whose minimum size is its own.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let h = harness_sized(CAMERA_RAW_MIN_WIDTH, CAMERA_RAW_MIN_HEIGHT, |app, ctx| {
            crate::camera_raw_ui::open(app, ctx).unwrap();
        });
        assert!(crate::control::inspect(h.state(), &h.ctx)["cameraRaw"].is_object(), "camera raw is open");
        assert_inside(&h, "camera-raw", CAMERA_RAW_MIN_WIDTH, CAMERA_RAW_MIN_HEIGHT);
    }));
    if let Err(panic) = outcome {
        let message = panic.downcast_ref::<String>().map(String::as_str).or(panic.downcast_ref::<&str>().copied()).unwrap_or("?");
        problems.push(format!("camera-raw: {message}"));
    }
    assert!(problems.is_empty(), "{} diálogo(s) fora do padrão:\n{}", problems.len(), problems.join("\n"));
}

/// The top bar and the update card fit the minimum size too.
#[test]
fn the_top_bar_and_the_update_card_fit_the_narrow_window() {
    let h = harness_at(MIN_WIDTH, |_, _| {});
    assert_inside(&h, "top-bar", MIN_WIDTH, MIN_HEIGHT);

    let release =
        localraster_engine::updates_cmds::Release { version: "99.0.0".into(), url: "https://github.com/Anon5T4R/Local-Raster/releases/tag/v99.0.0".into() };
    let mut h = harness_at(MIN_WIDTH, |app, _| {
        app.session.updates.status = localraster_engine::updates_cmds::Status::Available { release };
        crate::updates_ui::tick(app, &egui::Context::default());
        assert!(app.ui.updates_open, "the available release opened the update card");
    });
    h.run_steps(4);
    assert_inside(&h, "update-card", MIN_WIDTH, MIN_HEIGHT);
}
