//! Regenerates the shipped synthetic CMYK profile when `LOCALRASTER_REGEN_PROFILES=1`:
//!
//! ```sh
//! LOCALRASTER_REGEN_PROFILES=1 cargo test -p localraster-cms --release --test it -- regen::
//! ```
//!
//! Without the variable the test only checks that the generator's model still matches the
//! shipped AToB table (cheap), so model edits are not forgotten.

use localraster_cms::synth::{CmykModel, CmykParams};
use localraster_cms::{Builtin, Intent, Profile, Transform};

#[test]
fn regen_or_check_coated_cmyk() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/profiles/localraster-coated-cmyk.icc");
    if std::env::var_os("LOCALRASTER_REGEN_PROFILES").is_some() {
        let t = std::time::Instant::now();
        let p = localraster_cms::synth::coated_cmyk();
        let bytes = p.to_bytes();
        std::fs::write(path, &*bytes).unwrap();
        eprintln!("wrote {} bytes in {:?}", bytes.len(), t.elapsed());
        return;
    }
    // The shipped AToB (colorimetric) must reproduce the model.
    let shipped = Profile::parse(&std::fs::read(path).unwrap()).unwrap();
    let lab = Builtin::LabD50.profile();
    let t = Transform::new(&shipped, lab, Intent::RelativeColorimetric, false).unwrap();
    let model = CmykModel::coated(CmykParams::default().tvi);
    let mut worst = 0.0f64;
    for c in [0.0, 0.3, 0.7, 1.0] {
        for m in [0.0, 0.45, 1.0] {
            for y in [0.0, 0.55, 1.0] {
                for k in [0.0, 0.5, 1.0] {
                    let mut o = [0.0f32; 3];
                    t.eval(&[c as f32, m as f32, y as f32, k as f32], &mut o);
                    let got = [o[0] as f64 * 100.0, o[1] as f64 * 255.0 - 128.0, o[2] as f64 * 255.0 - 128.0];
                    let want = model.lab([c, m, y, k]);
                    worst = worst.max(localraster_cms::math::delta_e76(got, want));
                }
            }
        }
    }
    assert!(worst < 1.0, "shipped CMYK profile is stale (max ΔE {worst}); regenerate it");
}
