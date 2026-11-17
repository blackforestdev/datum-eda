//! Guarded library reads use the existing owned service and its trusted policy.
use super::*;

/// One library intent cannot contain both a legacy definition and a revision.
#[derive(Debug, Clone)]
pub enum NativeLibraryEdit {
    Definition {
        create: bool,
        request: eda_engine::api::native_write::library_gui::LibraryDefinitionRequest,
    },
    Revision {
        request: Value,
        /// Authored purpose, separate from the request UUID used for transport
        /// reconciliation. Random correlation IDs must not enter provenance.
        reason: String,
    },
}

impl NativeLibraryEdit {
    pub(super) fn verb_params(&self) -> Result<(&'static str, Value)> {
        match self {
            Self::Definition { create, request } => Ok((
                if *create {
                    "datum.library.create"
                } else {
                    "datum.library.edit"
                },
                serde_json::to_value(request)?,
            )),
            Self::Revision { request, .. } => Ok(("datum.library.edit", request.clone())),
        }
    }

    pub(super) fn revision_reason(&self) -> Option<&str> {
        match self {
            Self::Revision { reason, .. } => Some(reason.trim()),
            Self::Definition { .. } => None,
        }
    }
}

impl NativeProjectClient {
    /// Preview the exact authored inputs through the same service builder as
    /// Apply. It performs no commit and conveys no review or publication approval.
    pub fn preview_library_edit(
        &self,
        intent: &NativeEditIntent,
    ) -> Result<crate::library_browser::revision_workflow::QualifiedLibraryPreview> {
        anyhow::ensure!(
            matches!(intent.action, NativeEditAction::Library),
            "Library intent required"
        );
        let edit = intent
            .library
            .as_ref()
            .context("Library edit payload missing")?;
        let reason = edit
            .revision_reason()
            .context("Revision preview requires a revision intent")?;
        anyhow::ensure!(
            !reason.is_empty(),
            "Record the purpose of the library revision"
        );
        let identity =
            crate::library_browser::revision_workflow::LibraryPreviewIdentity::from_intent(intent)
                .context("Library revision intent required")?;
        let (verb, mut params) = edit.verb_params()?;
        let action = params
            .get("action")
            .and_then(Value::as_str)
            .context("Library revision action is missing")?
            .to_owned();
        if action == "adopt_selection" && params["request"].get("physical_geometry").is_none() {
            let selection = json!({
                "expected_catalog": params["request"]["expected_catalog"],
                "selected": params["request"]["selected"],
            });
            let description = self.call(
                "native.describe",
                json!({"project_root": intent.project_root,
                    "library_geometry_selection": selection}),
                Uuid::new_v4(),
            )?;
            let geometry = qualify_geometry_selection(&description, intent, &selection)?;
            if !geometry["placements"].as_array().unwrap().is_empty() {
                params["request"]["physical_geometry"] = geometry.clone();
            }
        }
        let prepared_request = params.clone();
        let reply = self.call(
            "native.write",
            json!({"project_root":intent.project_root,"verb":verb,"params":params,
                "actor":"datum-gui","source":"manual","reason":reason,"dry_run":true,
                "batch_id":intent.request_id,"expected_model_revision":intent.expected_revision,
                "expected_transaction_tip":intent.expected_tip}),
            intent.request_id,
        )?;
        anyhow::ensure!(
            reply.get("project_id") == Some(&json!(intent.project_id))
                && reply.get("expected_model_revision").and_then(Value::as_str)
                    == Some(intent.expected_revision.as_str())
                && reply.get("transaction_tip") == Some(&json!(intent.expected_tip))
                && reply.get("status").and_then(Value::as_str) == Some("dry_run")
                && reply.get("verb").and_then(Value::as_str) == Some(verb)
                && reply.get("library_preview").is_some_and(Value::is_object),
            "Library preview does not match the current project; refresh before applying"
        );
        anyhow::ensure!(
            reply["library_preview"]
                .get("action")
                .and_then(Value::as_str)
                == Some(action.as_str()),
            "Library preview describes a different action; refresh before applying"
        );
        qualify_geometry_preview(&prepared_request, &reply["library_preview"])?;
        Ok(
            crate::library_browser::revision_workflow::QualifiedLibraryPreview {
                identity,
                summary: reply["library_preview"].clone(),
                prepared_request,
            },
        )
    }

    /// Read common records from the same service that owns native writes. A
    /// separate GUI-process policy is neither configured nor inferred here.
    pub fn inspect_library_snapshot(
        &self,
        project_root: &Path,
        project_id: Uuid,
        expected_revision: &str,
        expected_tip: Option<Uuid>,
    ) -> Result<Value> {
        let description = self.call(
            "native.describe",
            json!({"project_root":project_root,"include_library":true}),
            Uuid::new_v4(),
        )?;
        qualify_library_snapshot(&description, project_id, expected_revision, expected_tip)?;
        Ok(description)
    }
}

fn qualify_geometry_preview(request: &Value, summary: &Value) -> Result<()> {
    let Some(geometry) = request["request"].get("physical_geometry") else {
        return Ok(());
    };
    let requested = geometry
        .get("placements")
        .and_then(Value::as_array)
        .context("Physical update placement intent is unavailable")?;
    if requested.is_empty() {
        return Ok(());
    }
    let reviewed = summary
        .get("physical_geometry_updates")
        .and_then(Value::as_array)
        .context("PCB geometry changes are unavailable for this preview")?;
    let identities = |users: &[Value], require_geometry: bool| -> Result<_> {
        let mut result = std::collections::BTreeMap::new();
        let mut all_pads = std::collections::BTreeSet::new();
        let mut packages = std::collections::BTreeSet::new();
        for user in users {
            if require_geometry {
                anyhow::ensure!(
                    user["reference"].as_str().is_some_and(|r| !r.is_empty()),
                    "PCB component reference is unavailable"
                );
            }
            let package: Uuid = serde_json::from_value(user["placed_package"].clone())?;
            anyhow::ensure!(
                packages.insert(package),
                "PCB placement appears more than once"
            );
            let component: Option<Uuid> = serde_json::from_value(
                user.get("component_instance")
                    .context("Component identity unavailable")?
                    .clone(),
            )?;
            let pads = user["pads"]
                .as_array()
                .context("Pad correspondence unavailable")?;
            anyhow::ensure!(!pads.is_empty(), "Pad correspondence is empty");
            let mut correspondence = std::collections::BTreeMap::new();
            let mut library_pads = std::collections::BTreeSet::new();
            for pad in pads {
                let placed: Uuid = serde_json::from_value(pad["placed_pad"].clone())?;
                let library: Uuid = serde_json::from_value(pad["library_pad"].clone())?;
                anyhow::ensure!(
                    all_pads.insert(placed) && library_pads.insert(library),
                    "Pad correspondence is ambiguous"
                );
                correspondence.insert(placed, library);
                if require_geometry {
                    for side in ["before", "after"] {
                        serde_json::from_value::<
                            eda_engine::substrate::component_pad_source::FrozenPadGeometry,
                        >(
                            pad.get(side).context("Pad geometry unavailable")?.clone()
                        )?;
                    }
                }
            }
            anyhow::ensure!(
                result
                    .insert((component, package), correspondence)
                    .is_none(),
                "PCB placement appears more than once"
            );
        }
        Ok(result)
    };
    anyhow::ensure!(
        identities(requested, false)? == identities(reviewed, true)?,
        "PCB preview omits or changes the requested placement or pad identities"
    );
    Ok(())
}

fn qualify_geometry_selection<'a>(
    description: &'a Value,
    intent: &NativeEditIntent,
    selection: &Value,
) -> Result<&'a Value> {
    let geometry = &description["library_geometry_selection"];
    for snapshot in [description, geometry] {
        anyhow::ensure!(
            snapshot.get("project_id") == Some(&json!(intent.project_id))
                && snapshot.get("model_revision").and_then(Value::as_str)
                    == Some(intent.expected_revision.as_str())
                && snapshot.get("transaction_tip") == Some(&json!(intent.expected_tip)),
            "Project changed during PCB update inspection; refresh before previewing"
        );
    }
    anyhow::ensure!(
        geometry.get("expected_catalog") == selection.get("expected_catalog")
            && geometry.get("selected") == selection.get("selected")
            && geometry["physical_geometry"]["placements"].is_array(),
        "PCB update inspection does not match the selected exact revision"
    );
    Ok(&geometry["physical_geometry"])
}

fn qualify_library_snapshot(
    description: &Value,
    project_id: Uuid,
    expected_revision: &str,
    expected_tip: Option<Uuid>,
) -> Result<()> {
    anyhow::ensure!(
        description.get("project_id") == Some(&json!(project_id))
            && description.get("model_revision").and_then(Value::as_str) == Some(expected_revision)
            && description.get("transaction_tip") == Some(&json!(expected_tip)),
        "Project changed during library inspection; refresh before reviewing revisions"
    );
    let library = description
        .get("library")
        .context("Library inspection is unavailable")?;
    anyhow::ensure!(
        library.get("project_id") == Some(&json!(project_id))
            && library.get("model_revision").and_then(Value::as_str) == Some(expected_revision)
            && library.get("managed").and_then(Value::as_bool).is_some()
            && library.get("assets").is_some_and(Value::is_array)
            && library.get("available_assets").is_some_and(Value::is_array),
        "Library inspection does not match the requested native snapshot"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Protocol controls use an owned socket, not fabricated CAD/journal proof.
    fn companion(
        replies: Vec<Value>,
    ) -> (NativeProjectClient, std::thread::JoinHandle<Vec<Value>>) {
        let directory = std::env::temp_dir().join(format!("datum-library-rpc-{}", Uuid::new_v4()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        let socket = directory.join("engine.sock");
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let worker = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut requests = Vec::new();
            for reply in replies {
                let (mut stream, _) = loop {
                    match listener.accept() {
                        Ok(pair) => break pair,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "expected native request missing");
                            std::thread::sleep(Duration::from_millis(2));
                        }
                        Err(error) => panic!("{error}"),
                    }
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut line = String::new();
                BufReader::new(stream.try_clone().unwrap())
                    .read_line(&mut line)
                    .unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                let envelope = json!({"jsonrpc":"2.0","id":request["id"],"result":reply});
                serde_json::to_writer(&mut stream, &envelope).unwrap();
                stream.write_all(b"\n").unwrap();
                requests.push(request);
            }
            requests
        });
        let child = Command::new("true").spawn().unwrap();
        (
            NativeProjectClient {
                child,
                directory,
                socket,
            },
            worker,
        )
    }

    fn revision_intent() -> NativeEditIntent {
        let mut intent = NativeEditIntent::new(
            PathBuf::from("/native/project"),
            Uuid::new_v4(),
            "current".into(),
            None,
            NativeEditAction::Library,
        );
        intent.library = Some(NativeLibraryEdit::Revision {
            request: json!({"action":"adopt_profile"}),
            reason: "  Adopt the reviewed library format  ".into(),
        });
        intent
    }

    #[test]
    fn geometry_discovery_binds_the_exact_selected_preview_request() {
        let mut intent = revision_intent();
        let request = json!({"action":"adopt_selection","request":{
            "expected_catalog":"catalog","selected":{"asset_id":Uuid::new_v4(),
                "revision_id":"exact"}}});
        intent.library = Some(NativeLibraryEdit::Revision {
            request: request.clone(),
            reason: "Review PCB geometry".into(),
        });
        let geometry = json!({"placements":[{"component_instance":Uuid::new_v4(),"placed_package":Uuid::new_v4(),
            "pads":[{"placed_pad":Uuid::new_v4(),"library_pad":Uuid::new_v4()}]}]});
        let frozen = json!({"name":"1","position":{"x":0,"y":0},"layer":1,
            "aperture":{"circle":{"diameter_nm":1000000}},"drill_nm":null,
            "layer_span":"pad_layer","plated":null});
        let mut reviewed = geometry["placements"].clone();
        reviewed[0]["reference"] = json!("U1");
        reviewed[0]["pads"][0]["before"] = frozen.clone();
        reviewed[0]["pads"][0]["after"] = frozen;
        reviewed[0]["pads"][0]["after"]["aperture"]["circle"]["diameter_nm"] = json!(1200000);
        let description = json!({"project_id":intent.project_id,"model_revision":"current",
            "transaction_tip":null,"library_geometry_selection":{
                "project_id":intent.project_id,"model_revision":"current","transaction_tip":null,
                "expected_catalog":"catalog","selected":request["request"]["selected"],
                "physical_geometry":geometry}});
        let dry_run = json!({"project_id":intent.project_id,"expected_model_revision":"current",
            "transaction_tip":null,"verb":"datum.library.edit","status":"dry_run",
            "library_preview":{"action":"adopt_selection","physical_geometry_updates":reviewed}});
        let initial = json!({"project_id":intent.project_id,"model_revision":"current",
            "transaction_tip":null,"durability_confirmed":true});
        let mut after = initial.clone();
        after["transactions"] =
            json!([{"reason":"Review PCB geometry","batch_id":intent.request_id}]);
        let (client, worker) = companion(vec![
            description,
            dry_run,
            initial,
            json!({"status":"applied"}),
            after,
        ]);
        let preview = client.preview_library_edit(&intent).unwrap();
        let valid_summary = preview.summary.clone();
        let mut expected = request.clone();
        expected["request"]["physical_geometry"] = geometry;
        assert_eq!(preview.prepared_request, expected);
        assert_eq!(
            Some(preview.identity),
            crate::library_browser::revision_workflow::LibraryPreviewIdentity::from_intent(&intent)
        );
        intent.library = Some(NativeLibraryEdit::Revision {
            request: preview.prepared_request,
            reason: "Review PCB geometry".into(),
        });
        client.execute(&intent).unwrap();
        let sent = worker.join().unwrap();
        assert_eq!(sent.len(), 5);
        assert_eq!(sent[0]["method"], "native.describe");
        assert_eq!(
            sent[0]["params"]["library_geometry_selection"],
            request["request"]
        );
        assert_eq!(sent[1]["method"], "native.write");
        assert_eq!(sent[1]["params"]["dry_run"], true);
        assert_eq!(sent[1]["params"]["params"], expected);
        assert_eq!(sent[1]["params"]["batch_id"], json!(intent.request_id));
        assert_eq!(sent[3]["method"], "native.write");
        assert_ne!(sent[3]["params"]["dry_run"], true);
        assert_eq!(sent[3]["params"]["params"], expected);
        assert_eq!(sent[3]["params"]["batch_id"], json!(intent.request_id));
        for corruption in 0..8 {
            let mut changed = valid_summary.clone();
            match corruption {
                0 => changed["physical_geometry_updates"] = json!([]),
                1 => changed["physical_geometry_updates"][0]["reference"] = Value::Null,
                2 => {
                    changed["physical_geometry_updates"][0]["placed_package"] =
                        json!(Uuid::new_v4())
                }
                3 => {
                    changed["physical_geometry_updates"][0]["pads"][0]["library_pad"] =
                        json!(Uuid::new_v4())
                }
                4 => changed["physical_geometry_updates"][0]["pads"][0]["before"] = Value::Null,
                5 => {
                    changed["physical_geometry_updates"][0]["pads"][0]["after"]["drill_nm"] =
                        json!(0.5)
                }
                6 => {
                    let duplicate = changed["physical_geometry_updates"][0].clone();
                    changed["physical_geometry_updates"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }
                _ => {
                    let duplicate = changed["physical_geometry_updates"][0]["pads"][0].clone();
                    changed["physical_geometry_updates"][0]["pads"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }
            }
            assert!(
                qualify_geometry_preview(&expected, &changed).is_err(),
                "{corruption}"
            );
        }
        // The same package cannot acquire a second owner with disjoint pads,
        // even when discovery and the displayed summary agree on both rows.
        let mut duplicate_owner = expected.clone();
        let mut extra = duplicate_owner["request"]["physical_geometry"]["placements"][0].clone();
        extra["component_instance"] = json!(Uuid::new_v4());
        extra["pads"][0]["placed_pad"] = json!(Uuid::new_v4());
        extra["pads"][0]["library_pad"] = json!(Uuid::new_v4());
        duplicate_owner["request"]["physical_geometry"]["placements"]
            .as_array_mut()
            .unwrap()
            .push(extra.clone());
        let mut summary = valid_summary;
        let mut row = summary["physical_geometry_updates"][0].clone();
        row["component_instance"] = extra["component_instance"].clone();
        row["pads"][0]["placed_pad"] = extra["pads"][0]["placed_pad"].clone();
        row["pads"][0]["library_pad"] = extra["pads"][0]["library_pad"].clone();
        summary["physical_geometry_updates"]
            .as_array_mut()
            .unwrap()
            .push(row);
        assert!(qualify_geometry_preview(&duplicate_owner, &summary).is_err());
    }

    #[test]
    fn geometry_discovery_refuses_mixed_or_missing_snapshot_and_selection() {
        let intent = revision_intent();
        let selection = json!({"expected_catalog":"catalog","selected":{"asset_id":"asset",
            "revision_id":"exact"}});
        let valid = json!({"project_id":intent.project_id,"model_revision":"current",
            "transaction_tip":null,"library_geometry_selection":{
                "project_id":intent.project_id,"model_revision":"current","transaction_tip":null,
                "expected_catalog":"catalog","selected":selection["selected"],
                "physical_geometry":{"placements":[]}}});
        qualify_geometry_selection(&valid, &intent, &selection).unwrap();
        for (nested, key, replacement) in [
            (false, "project_id", json!(Uuid::new_v4())),
            (false, "model_revision", json!("old")),
            (false, "transaction_tip", json!(Uuid::new_v4())),
            (true, "project_id", json!(Uuid::new_v4())),
            (true, "model_revision", json!("old")),
            (true, "transaction_tip", json!(Uuid::new_v4())),
            (true, "expected_catalog", json!("other")),
            (
                true,
                "selected",
                json!({"asset_id":"other","revision_id":"exact"}),
            ),
            (true, "physical_geometry", json!({"placements":null})),
        ] {
            let mut changed = valid.clone();
            let field = if nested {
                &mut changed["library_geometry_selection"]
            } else {
                &mut changed
            };
            field[key] = replacement;
            assert!(
                qualify_geometry_selection(&changed, &intent, &selection).is_err(),
                "{key}"
            );
        }
        for nested in [false, true] {
            let mut changed = valid.clone();
            let field = if nested {
                &mut changed["library_geometry_selection"]
            } else {
                &mut changed
            };
            field.as_object_mut().unwrap().remove("transaction_tip");
            assert!(qualify_geometry_selection(&changed, &intent, &selection).is_err());
        }
    }

    #[test]
    fn revision_preview_refuses_wrong_action_and_snapshot_anchors() {
        let intent = revision_intent();
        let valid = json!({"project_id":intent.project_id,"expected_model_revision":"current",
            "transaction_tip":null,"verb":"datum.library.edit","status":"dry_run",
            "library_preview":{"action":"adopt_profile"}});
        let mut replies = vec![valid.clone()];
        for (key, value) in [
            ("project_id", json!(Uuid::new_v4())),
            ("expected_model_revision", json!("old")),
            ("transaction_tip", json!(Uuid::new_v4())),
            ("library_preview", json!({"action":"adopt"})),
        ] {
            let mut changed = valid.clone();
            changed[key] = value;
            replies.push(changed);
        }
        let mut missing = valid;
        missing.as_object_mut().unwrap().remove("transaction_tip");
        replies.push(missing);
        let (client, worker) = companion(replies);
        client.preview_library_edit(&intent).unwrap();
        for _ in 0..5 {
            assert!(client.preview_library_edit(&intent).is_err());
        }
        for request in worker.join().unwrap() {
            assert_eq!(request["method"], "native.write");
            assert_eq!(request["params"]["dry_run"], true);
            assert_eq!(
                request["params"]["reason"],
                "Adopt the reviewed library format"
            );
            assert_eq!(request["params"]["batch_id"], json!(intent.request_id));
        }
    }

    #[test]
    fn revision_apply_reconciles_only_its_batch_with_the_authored_preview_reason() {
        for matching_batch in [false, true] {
            let intent = revision_intent();
            let initial = json!({"project_id":intent.project_id,"model_revision":"current",
                "transaction_tip":null,"durability_confirmed":true});
            let mut after = initial.clone();
            after["transactions"] = json!([{"reason":"Adopt the reviewed library format",
                "batch_id":if matching_batch { intent.request_id } else { Uuid::new_v4() }}]);
            let (client, worker) = companion(vec![initial, json!({"status":"applied"}), after]);
            let result = client.execute(&intent);
            assert_eq!(result.is_ok(), matching_batch);
            let requests = worker.join().unwrap();
            assert_eq!(
                requests[1]["params"]["reason"],
                "Adopt the reviewed library format"
            );
            assert_eq!(requests[1]["params"]["batch_id"], json!(intent.request_id));
            assert_eq!(
                requests[1]["params"]["params"],
                json!({"action":"adopt_profile"})
            );
        }
    }

    #[test]
    fn changed_or_mixed_library_descriptions_cannot_replace_the_current_snapshot() {
        let project = Uuid::new_v4();
        let tip = Uuid::new_v4();
        let valid = json!({"project_id":project,"model_revision":"current",
            "transaction_tip":tip,"library":{"project_id":project,
                "model_revision":"current","managed":false,"assets":[],"available_assets":[]}});
        qualify_library_snapshot(&valid, project, "current", Some(tip)).unwrap();
        for (path, replacement) in [
            (vec!["project_id"], json!(Uuid::new_v4())),
            (vec!["model_revision"], json!("old")),
            (vec!["transaction_tip"], Value::Null),
            (vec!["library", "project_id"], json!(Uuid::new_v4())),
            (vec!["library", "model_revision"], json!("old")),
            (vec!["library", "managed"], json!("unknown")),
            (vec!["library", "available_assets"], Value::Null),
        ] {
            let mut changed = valid.clone();
            let mut field = &mut changed;
            for key in path {
                field = &mut field[key];
            }
            *field = replacement;
            assert!(qualify_library_snapshot(&changed, project, "current", Some(tip)).is_err());
        }
        let mut empty = valid;
        empty["transaction_tip"] = Value::Null;
        qualify_library_snapshot(&empty, project, "current", None).unwrap();
        empty.as_object_mut().unwrap().remove("transaction_tip");
        assert!(qualify_library_snapshot(&empty, project, "current", None).is_err());
    }
}
