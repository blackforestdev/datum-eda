//! Finite S5A native recipes; neither input replay nor acceptance evidence.
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
#[path = "s5a_fixture/annotations.rs"]
mod annotations;
#[path = "s5a_fixture/artifacts.rs"]
mod artifacts;
#[path = "s5a_fixture/bus_run.rs"]
mod bus_run;
#[path = "s5a_fixture/classes.rs"]
mod classes;
#[path = "s5a_fixture/collision.rs"]
mod collision;
#[path = "s5a_fixture/library.rs"]
mod library;
#[path = "s5a_fixture/lifecycle.rs"]
mod lifecycle;
#[path = "s5a_fixture/majority.rs"]
mod majority;
#[path = "s5a_fixture/parents.rs"]
mod parents;
#[path = "s5a_fixture/recipe.rs"]
mod recipe;
#[path = "s5a_fixture/scope.rs"]
mod scope;
#[path = "s5a_fixture/semantic.rs"]
mod semantic;
#[path = "s5a_fixture/wire_run_hidden.rs"]
mod wire_run_hidden;
#[path = "s5a_fixture/workspace.rs"]
mod workspace;

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let family = args.next().context(
        "usage: s5a_fixture <electrical-scope|compound-output|dense-and-cancel|revision-lifetime|class-qualification|semantic-scope|scope-and-reopen|library-base|library-parents|annotation-scope|non-authored-channels|symbol-majority|maximal-channel-collision|bus-run-refusal|bus-run-refusal-256|wire-run-hidden-257> [<fresh-root>|--managed] [existing CLI binary]",
    )?;
    let family = family.to_str().context("family must be UTF-8")?;
    // External explicit roots remain valid for pinned replay. New/default
    // workspaces use external, owner-labelled placement with repeat refusal.
    let root = match args.next() {
        None => workspace::managed_root(family)?,
        Some(value) if value == "--managed" => workspace::managed_root(family)?,
        Some(value) => PathBuf::from(value),
    };
    workspace::validate_output(&root)?;
    if matches!(
        family,
        "library-parents"
            | "annotation-scope"
            | "non-authored-channels"
            | "symbol-majority"
            | "maximal-channel-collision"
    ) {
        let cli = PathBuf::from(
            args.next()
                .context("existing datum-eda CLI binary required")?,
        );
        if args.next().is_some() {
            bail!("unexpected argument");
        }
        return if family == "maximal-channel-collision" {
            collision::build(&root, &cli)
        } else if family == "symbol-majority" {
            majority::build(&root, &cli)
        } else if family == "non-authored-channels" {
            artifacts::build(&root, &cli)
        } else if family == "annotation-scope" {
            annotations::build(&root, &cli)
        } else {
            parents::build(&root, &cli)
        };
    }
    if args.next().is_some() {
        bail!("unexpected argument");
    }
    run(family, &root)
}
fn run(family: &str, root: &Path) -> Result<()> {
    match family {
        "bus-run-refusal" => return bus_run::build(root, 257),
        "bus-run-refusal-256" => return bus_run::build(root, 256),
        "wire-run-hidden-257" => return wire_run_hidden::build(root),
        _ => {}
    }
    if family == "library-base" {
        return library::prepare(root);
    }
    let count = match family {
        "electrical-scope"
        | "revision-lifetime"
        | "class-qualification"
        | "semantic-scope"
        | "scope-and-reopen" => 1,
        "compound-output" => 257,
        "dense-and-cancel" => 100_000,
        _ => bail!("unsupported family; no substitute recipe is credited"),
    };
    if root.exists() {
        bail!("output must be a fresh root; existing artifacts are preserved");
    }
    let report = recipe::build_and_check(root, count)?;
    let variants = if family == "revision-lifetime" {
        Some(lifecycle::build(root)?)
    } else {
        None
    };
    let class_variant = if family == "class-qualification" {
        Some(classes::build(root)?)
    } else {
        None
    };
    let semantic_variant = if family == "semantic-scope" {
        Some(semantic::build(root)?)
    } else {
        None
    };
    let scope_variant = if family == "scope-and-reopen" {
        Some(scope::build(root)?)
    } else {
        None
    };
    let manifest = serde_json::json!({
        "schema_version": 1,
        "recipe": "s5a-native-core-v1",
        "family": family,
        "engine_observation": report,
        "engine_observation_basis": if family == "scope-and-reopen" { "construction baseline before canonical units initialization; scope_variant.projects binds the current source revisions" } else { "initial canonical core; added variants bind their own revisions" },
        "revision_variants": variants,
        "class_variant": class_variant,
        "semantic_variant": semantic_variant,
        "scope_variant": scope_variant,
        "native_input": "pending: bind candidate, binary, environment and actual input/observation before C04",
        "acceptance": "not established; C04/C05/C06 remain separate",
        "coverage": {
            "electrical-scope": "board Run: connected pair, disconnected same-Net track and foreign-Net contact; hierarchy/semantic Bus remain additional recipes",
            "compound-output": "257 actual schematic texts, exact Track widths 5080000/5080001 nm; GUI aggregation and context-cap proof remain pending",
            "dense-and-cancel": "100000 actual schematic texts; native selection masks and cancellation boundary remain pending",
            "revision-lifetime": "canonical preserved identity, deletion, undo and replacement variants; native selection lifecycle replay remains pending",
            "class-qualification": "actual Pad/Via/Track arc/Zone fill and schematic Line/Rect/Circle/Arc sources; incomplete placed-parent/Pin/graphics/native input cases remain pending",
            "scope-and-reopen": "two distinct actual Projects with intentionally shared object UUIDs and preconfigured mm/mil quantities; native panes/consumer restore remains pending",
            "other_families": "not generated by this recipe",
            "standalone_board_graphics": "blocked by dat-s5a-board-graphic-authority-h8xm; required positive cases remain pending",
            "manufacturing": "CAM/T04 remains open; this recipe grants no manufacturing acceptance"
        }
    });
    std::fs::write(
        root.join("s5a-fixture.json"),
        eda_test_harness::canonical_json(&manifest)?,
    )?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separate_projects_preserve_authored_quantity_contexts() {
        let root = std::env::temp_dir().join(format!("s5a-scope-test-{}", uuid::Uuid::new_v4()));
        run("scope-and-reopen", &root).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
        let projects = report["scope_variant"]["projects"].as_array().unwrap();
        assert_eq!(projects.len(), 2);
        assert_ne!(projects[0]["project_id"], projects[1]["project_id"]);
        assert_ne!(projects[0]["profile"], projects[1]["profile"]);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn semantic_variant_preserves_complete_bus_and_occurrence_membership() {
        let root = std::env::temp_dir().join(format!("s5a-semantic-test-{}", uuid::Uuid::new_v4()));
        run("semantic-scope", &root).unwrap();
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
        assert_eq!(
            report["semantic_variant"]["expected_global_net_members"]
                .as_array()
                .unwrap()
                .len(),
            21
        );
        assert_eq!(
            report["semantic_variant"]["expected_bus_members"]
                .as_array()
                .unwrap()
                .len(),
            8
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn class_variant_preserves_actual_geometry_and_current_generated_fill() {
        let root = std::env::temp_dir().join(format!("s5a-class-test-{}", uuid::Uuid::new_v4()));
        run("class-qualification", &root).unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
        assert_eq!(
            value["class_variant"]["pad_anchors"]
                .as_array()
                .unwrap()
                .len(),
            8
        );
        assert_eq!(
            value["class_variant"]["fill_islands"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn persisted_lifetime_variants_preserve_or_replace_actual_identity() {
        let root = std::env::temp_dir().join(format!("s5a-lifetime-test-{}", uuid::Uuid::new_v4()));
        run("revision-lifetime", &root).unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
        assert_eq!(value["revision_variants"].as_array().unwrap().len(), 4);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn native_recipe_checks_real_membership_reopen_and_nonmutation() {
        let root = std::env::temp_dir().join(format!("s5a-fixture-test-{}", uuid::Uuid::new_v4()));
        run("compound-output", &root).unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
        assert_eq!(value["engine_observation"]["text_population"]["count"], 257);
        assert_eq!(
            value["engine_observation"]["expected_run_members"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(run("compound-output", &root).is_err());
        assert!(
            value["acceptance"]
                .as_str()
                .unwrap()
                .starts_with("not established")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
