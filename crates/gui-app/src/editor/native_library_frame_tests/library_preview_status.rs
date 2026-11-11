//! Read-only truthful Unit/Symbol preview and retained refusal Console access.
use super::*;
use serde_json::json;

impl Fixture {
    pub(super) fn library_preview_status(&mut self, stage: usize) {
        let r = self.app.runtime.as_mut().unwrap();
        let state = r.workspace();
        let d = state.library.draft.as_ref().unwrap();
        let refused = stage >= 2;
        assert_eq!(d.kind, if stage == 0 { "units" } else { "symbols" });
        let reason = d.refusal.clone();
        assert_eq!(reason.is_some(), refused);
        assert_eq!(d.preview.is_some(), !refused);
        if let Some(reason) = &reason {
            assert!(reason.contains("positive radius"), "{reason}");
        }
        let inspector = r.prepared_scene().unwrap().native_content_text();
        assert!(
            !inspector
                .iter()
                .any(|t| t.region == "inspector" && t.text.starts_with("Drill:"))
        );
        assert!(inspector.iter().any(|t| t.text
            == if refused {
                "Preview refused"
            } else {
                "Validated preview · no source / journal writes"
            }));
        let mut notice_in_cycle = false;
        if stage < 3 {
            let initial = self
                .app
                .runtime
                .as_ref()
                .unwrap()
                .workspace()
                .library
                .focus
                .clone();
            assert!(initial.is_some());
            let mut closed = false;
            for _ in 0..128 {
                self.key(Key::Named(NamedKey::Tab));
                let focus = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .focus
                    .clone();
                notice_in_cycle |= focus == Some(C::Notice);
                if focus == initial {
                    closed = true;
                    break;
                }
            }
            assert!(
                closed,
                "actual Tab cycle must return to its starting control"
            );
            assert_eq!(notice_in_cycle, refused, "no phantom Notice tab stop");
        } else {
            let latest = self
                .app
                .runtime
                .as_ref()
                .unwrap()
                .workspace()
                .ui
                .console
                .latest()
                .unwrap();
            assert_eq!(Some(latest.message.as_str()), reason.as_deref());
        }
        std::fs::write(self.output.join(format!("preview-status-{stage}.json")),
            serde_json::to_vec_pretty(&json!({"stage":stage,"refusal":reason,
                "notice_in_tab_cycle":notice_in_cycle,"inspector":inspector.iter().map(|t| &t.text).collect::<Vec<_>>() })).unwrap()).unwrap();
    }
}

#[test]
#[ignore = "requires saved QNGV circuit and owned isolated display"]
fn ordinary_electrical_preview_success_and_genuine_refusal_are_truthful() {
    let root = PathBuf::from(std::env::var("DATUM_LIBRARY_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var("DATUM_LIBRARY_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let c = Step::KeyControl;
    let s = Step::LibraryPreviewStatus;
    let steps = vec![
        Step::Snapshot,
        Step::Open,
        Step::Kind(0),
        Step::Search("Teru logic".into()),
        Step::Select("Teru logic".into()),
        c(C::Edit),
        c(C::Preview),
        s(0),
        Step::Capture("status-unit-preview"),
        c(C::Cancel),
        Step::Invariant,
        Step::Kind(1),
        Step::Search("Teru component".into()),
        Step::Select("Teru component".into()),
        c(C::Edit),
        c(C::Preview),
        s(1),
        Step::Capture("status-symbol-preview"),
        c(C::Circle(0)),
        Step::Field("circle_radius", "0".into()),
        c(C::Preview),
        Step::Focus(C::Notice),
        s(2),
        Step::Capture("status-symbol-refused"),
        c(C::Notice),
        s(3),
        Step::Capture("status-refusal-console"),
        c(C::Cancel),
        Step::Invariant,
        Step::Reopen,
        Step::Invariant,
    ];
    scenario::run(root, output, steps, uuid::Uuid::nil());
}
