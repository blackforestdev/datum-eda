use std::{collections::BTreeMap, path::PathBuf};

use super::{
    CommitDiff, DesignModel, DomainObject, EngineError, ObjectRevision, Operation,
    SourceShardDirtyState, SourceShardKind, SourceShardRef, collect_uuid_objects,
    operation_application_board_payloads::{
        board_dimension_payload_objects, board_keepout_payload_objects,
        board_net_class_payload_objects, board_net_payload_objects, board_pad_payload_objects,
        board_payload_objects, board_text_payload_objects, board_track_payload_objects,
        board_via_payload_objects, board_zone_payload_objects, materialized_payload_objects,
    },
    operation_application_dispatch::apply_pre_match_operation,
    operation_application_object_revision::bump_existing_object,
    operation_application_objects::apply_operation_to_objects,
    operation_application_production::{
        apply_production_create, apply_production_delete, apply_production_set,
    },
    operation_application_relationship::{
        apply_relationship_create, apply_relationship_delete, apply_relationship_set,
        apply_variant_create, apply_variant_delete, apply_variant_set,
    },
    operation_application_schematic::{
        apply_schematic_map_create, apply_schematic_map_delete, apply_schematic_map_set,
    },
    source_shard::source_shard_taxon_for_path,
    source_shard_authority_for_kind,
    zone_fill::validated_zone_fill_payload,
};

pub(super) fn apply_operation(
    model: &mut DesignModel,
    operation: &Operation,
    diff: &mut CommitDiff,
) -> Result<(), EngineError> {
    if apply_pre_match_operation(model, operation, diff)? {
        return Ok(());
    }
    match operation {
        Operation::SetProjectName { project_id, name } => {
            model.project.name = name.clone();
            let object = model
                .objects
                .get_mut(project_id)
                .ok_or(EngineError::NotFound {
                    object_type: "domain_object",
                    uuid: *project_id,
                })?;
            object.object_revision = ObjectRevision(object.object_revision.0 + 1);
            diff.modified.push(*project_id);
            Ok(())
        }
        Operation::SetProjectRules { rules_root_id, .. } => {
            bump_existing_object(&mut model.objects, *rules_root_id, Some(diff))
        }
        Operation::AddProjectPoolRef { .. } | Operation::DeleteProjectPoolRef { .. } => Ok(()),
        Operation::CreateProjectRule {
            rules_root_id,
            rule_id,
            rule,
        } => {
            bump_existing_object(&mut model.objects, *rules_root_id, Some(diff))?;
            if model.objects.contains_key(rule_id) {
                return Err(EngineError::Validation(format!(
                    "project rule {rule_id} already exists"
                )));
            }
            let root = model
                .objects
                .get(rules_root_id)
                .ok_or(EngineError::NotFound {
                    object_type: "domain_object",
                    uuid: *rules_root_id,
                })?;
            model.objects.insert(
                *rule_id,
                DomainObject {
                    object_id: *rule_id,
                    object_revision: ObjectRevision(
                        rule.get("object_revision")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0),
                    ),
                    source_shard_id: root.source_shard_id,
                    domain: "rules".to_string(),
                    kind: "rules".to_string(),
                },
            );
            diff.created.push(*rule_id);
            Ok(())
        }
        Operation::SetProjectRule {
            rules_root_id,
            rule_id,
            ..
        } => {
            bump_existing_object(&mut model.objects, *rules_root_id, Some(diff))?;
            bump_existing_object(&mut model.objects, *rule_id, Some(diff))
        }
        Operation::DeleteProjectRule {
            rules_root_id,
            rule_id,
            ..
        } => {
            bump_existing_object(&mut model.objects, *rules_root_id, Some(diff))?;
            if model.objects.remove(rule_id).is_some() {
                diff.deleted.push(*rule_id);
                Ok(())
            } else {
                Err(EngineError::NotFound {
                    object_type: "project_rule",
                    uuid: *rule_id,
                })
            }
        }
        Operation::CreateBoardPackage {
            package_id,
            package,
            materialized,
        } => {
            let created = board_payload_objects(model, *package_id, package, materialized)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::DeleteBoardPackage {
            package_id,
            package,
            materialized,
        } => {
            let deleted = board_payload_objects(model, *package_id, package, materialized)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::SetBoardPackagePackage {
            package_id,
            previous_materialized,
            materialized,
            ..
        } => {
            let deleted = materialized_payload_objects(model, *package_id, previous_materialized)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            let created = materialized_payload_objects(model, *package_id, materialized)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            let object = model
                .objects
                .get_mut(package_id)
                .ok_or(EngineError::NotFound {
                    object_type: "domain_object",
                    uuid: *package_id,
                })?;
            object.object_revision = ObjectRevision(object.object_revision.0 + 1);
            diff.modified.push(*package_id);
            Ok(())
        }
        Operation::CreateBoardPad { pad_id, pad } => {
            let created = board_pad_payload_objects(model, *pad_id, pad)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::DeleteBoardPad { pad_id, pad } => {
            let deleted = board_pad_payload_objects(model, *pad_id, pad)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateBoardTrack { track_id, track } => {
            let created = board_track_payload_objects(model, *track_id, track)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::SetBoardTrack { track_id, .. } => {
            bump_existing_object(&mut model.objects, *track_id, Some(diff))
        }
        Operation::DeleteBoardTrack { track_id, track } => {
            let deleted = board_track_payload_objects(model, *track_id, track)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateBoardVia { via_id, via } => {
            let created = board_via_payload_objects(model, *via_id, via)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::SetBoardVia { via_id, .. } => {
            bump_existing_object(&mut model.objects, *via_id, Some(diff))
        }
        Operation::DeleteBoardVia { via_id, via } => {
            let deleted = board_via_payload_objects(model, *via_id, via)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateBoardZone { zone_id, zone } => {
            let created = board_zone_payload_objects(model, *zone_id, zone)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::SetBoardZone { zone_id, .. } => {
            if model.objects.contains_key(zone_id) {
                bump_existing_object(&mut model.objects, *zone_id, Some(diff))
            } else {
                Ok(())
            }
        }
        Operation::DeleteBoardZone { zone_id, zone } => {
            let deleted = board_zone_payload_objects(model, *zone_id, zone)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateBoardNet { net_id, net } => {
            let created = board_net_payload_objects(model, *net_id, net)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::DeleteBoardNet { net_id, net } => {
            let deleted = board_net_payload_objects(model, *net_id, net)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateBoardNetClass {
            net_class_id,
            net_class,
        } => {
            let created = board_net_class_payload_objects(model, *net_class_id, net_class)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::DeleteBoardNetClass {
            net_class_id,
            net_class,
        } => {
            let deleted = board_net_class_payload_objects(model, *net_class_id, net_class)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateBoardDimension {
            dimension_id,
            dimension,
        } => {
            let created = board_dimension_payload_objects(model, *dimension_id, dimension)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::DeleteBoardDimension {
            dimension_id,
            dimension,
        } => {
            let deleted = board_dimension_payload_objects(model, *dimension_id, dimension)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateBoardText { text_id, text } => {
            let created = board_text_payload_objects(model, *text_id, text)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::DeleteBoardText { text_id, text } => {
            let deleted = board_text_payload_objects(model, *text_id, text)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateBoardKeepout {
            keepout_id,
            keepout,
        } => {
            let created = board_keepout_payload_objects(model, *keepout_id, keepout)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            Ok(())
        }
        Operation::DeleteBoardKeepout {
            keepout_id,
            keepout,
        } => {
            let deleted = board_keepout_payload_objects(model, *keepout_id, keepout)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(())
        }
        Operation::CreateSchematicWire {
            sheet_id,
            wire_id,
            wire,
        } => apply_schematic_map_create(model, diff, *sheet_id, "wires", *wire_id, wire),
        Operation::DeleteSchematicWire {
            sheet_id,
            wire_id,
            wire,
        } => apply_schematic_map_delete(model, diff, *sheet_id, "wires", *wire_id, wire),
        Operation::CreateSchematicJunction {
            sheet_id,
            junction_id,
            junction,
        } => {
            apply_schematic_map_create(model, diff, *sheet_id, "junctions", *junction_id, junction)
        }
        Operation::DeleteSchematicJunction {
            sheet_id,
            junction_id,
            junction,
        } => {
            apply_schematic_map_delete(model, diff, *sheet_id, "junctions", *junction_id, junction)
        }
        Operation::CreateSchematicNoConnect {
            sheet_id,
            noconnect_id,
            noconnect,
        } => apply_schematic_map_create(
            model,
            diff,
            *sheet_id,
            "noconnects",
            *noconnect_id,
            noconnect,
        ),
        Operation::PlaceSchematicMarker {
            sheet_id,
            marker_id,
            marker_kind,
            marker,
        } => apply_schematic_map_create(
            model,
            diff,
            *sheet_id,
            marker_kind.map_name(),
            *marker_id,
            marker,
        ),
        Operation::DeleteSchematicNoConnect {
            sheet_id,
            noconnect_id,
            noconnect,
        } => apply_schematic_map_delete(
            model,
            diff,
            *sheet_id,
            "noconnects",
            *noconnect_id,
            noconnect,
        ),
        Operation::CreateSchematicSheet {
            schematic_id,
            sheet_id,
            relative_path,
            sheet,
            ..
        } => {
            if model.objects.contains_key(sheet_id) {
                return Err(EngineError::Validation(format!(
                    "schematic sheet {sheet_id} already exists"
                )));
            }
            let root = model
                .objects
                .get_mut(schematic_id)
                .ok_or(EngineError::NotFound {
                    object_type: "schematic_root",
                    uuid: *schematic_id,
                })?;
            root.object_revision = ObjectRevision(root.object_revision.0 + 1);
            diff.modified.push(*schematic_id);
            let shard_id = uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_URL,
                format!("datum-eda:source-shard:schematic/{relative_path}").as_bytes(),
            );
            let sheet_object = DomainObject {
                object_id: *sheet_id,
                object_revision: ObjectRevision(0),
                source_shard_id: shard_id,
                domain: "schematic".to_string(),
                kind: "schematic_sheet".to_string(),
            };
            model.objects.insert(*sheet_id, sheet_object.clone());
            diff.created.push(*sheet_id);
            let payload_objects = schematic_sheet_payload_objects(relative_path, sheet);
            for (object_id, object) in payload_objects {
                if object_id == *sheet_id {
                    continue;
                }
                if model.objects.insert(object_id, object).is_none() {
                    diff.created.push(object_id);
                }
            }
            Ok(())
        }
        Operation::DeleteSchematicSheet {
            schematic_id,
            sheet_id,
            relative_path,
            sheet,
            ..
        } => {
            let root = model
                .objects
                .get_mut(schematic_id)
                .ok_or(EngineError::NotFound {
                    object_type: "schematic_root",
                    uuid: *schematic_id,
                })?;
            root.object_revision = ObjectRevision(root.object_revision.0 + 1);
            diff.modified.push(*schematic_id);
            let payload_objects = schematic_sheet_payload_objects(relative_path, sheet);
            for object_id in payload_objects.keys() {
                if object_id != sheet_id && model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            if model.objects.remove(sheet_id).is_some() {
                diff.deleted.push(*sheet_id);
                Ok(())
            } else {
                Err(EngineError::NotFound {
                    object_type: "schematic_sheet",
                    uuid: *sheet_id,
                })
            }
        }
        Operation::SetSchematicSheetName { .. } => Ok(()),
        Operation::CreateSchematicLabel {
            sheet_id,
            label_id,
            label,
        } => apply_schematic_map_create(model, diff, *sheet_id, "labels", *label_id, label),
        Operation::DeleteSchematicLabel {
            sheet_id,
            label_id,
            label,
        } => apply_schematic_map_delete(model, diff, *sheet_id, "labels", *label_id, label),
        Operation::CreateSchematicPort {
            sheet_id,
            port_id,
            port,
        } => apply_schematic_map_create(model, diff, *sheet_id, "ports", *port_id, port),
        Operation::DeleteSchematicPort {
            sheet_id,
            port_id,
            port,
        } => apply_schematic_map_delete(model, diff, *sheet_id, "ports", *port_id, port),
        Operation::CreateSchematicBus {
            sheet_id,
            bus_id,
            bus,
        } => apply_schematic_map_create(model, diff, *sheet_id, "buses", *bus_id, bus),
        Operation::DeleteSchematicBus {
            sheet_id,
            bus_id,
            bus,
        } => apply_schematic_map_delete(model, diff, *sheet_id, "buses", *bus_id, bus),
        Operation::CreateSchematicBusEntry {
            sheet_id,
            bus_entry_id,
            bus_entry,
        } => apply_schematic_map_create(
            model,
            diff,
            *sheet_id,
            "bus_entries",
            *bus_entry_id,
            bus_entry,
        ),
        Operation::DeleteSchematicBusEntry {
            sheet_id,
            bus_entry_id,
            bus_entry,
        } => apply_schematic_map_delete(
            model,
            diff,
            *sheet_id,
            "bus_entries",
            *bus_entry_id,
            bus_entry,
        ),
        Operation::CreateSchematicText {
            sheet_id,
            text_id,
            text,
        } => apply_schematic_map_create(model, diff, *sheet_id, "texts", *text_id, text),
        Operation::DeleteSchematicText {
            sheet_id,
            text_id,
            text,
        } => apply_schematic_map_delete(model, diff, *sheet_id, "texts", *text_id, text),
        Operation::CreateSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing,
        } => apply_schematic_map_create(model, diff, *sheet_id, "drawings", *drawing_id, drawing),
        Operation::DeleteSchematicDrawing {
            sheet_id,
            drawing_id,
            drawing,
        } => apply_schematic_map_delete(model, diff, *sheet_id, "drawings", *drawing_id, drawing),
        Operation::CreateSchematicSymbol {
            sheet_id,
            symbol_id,
            symbol,
        } => apply_schematic_map_create(model, diff, *sheet_id, "symbols", *symbol_id, symbol),
        Operation::SetSchematicSymbol {
            sheet_id,
            symbol_id,
            symbol,
        } => apply_schematic_map_set(model, diff, *sheet_id, "symbols", *symbol_id, symbol),
        Operation::DeleteSchematicSymbol {
            sheet_id,
            symbol_id,
            symbol,
        } => apply_schematic_map_delete(model, diff, *sheet_id, "symbols", *symbol_id, symbol),
        Operation::CreateManufacturingPlan {
            manufacturing_plan_id,
            manufacturing_plan,
        } => apply_production_create(
            model,
            diff,
            *manufacturing_plan_id,
            manufacturing_plan,
            SourceShardKind::ManufacturingPlan,
            "manufacturing",
            "manufacturing_plan",
        ),
        Operation::SetManufacturingPlan {
            manufacturing_plan_id: object_id,
            manufacturing_plan: value,
            ..
        }
        | Operation::SetPanelProjection {
            panel_projection_id: object_id,
            panel_projection: value,
            ..
        }
        | Operation::SetOutputJob {
            output_job_id: object_id,
            output_job: value,
            ..
        } => apply_production_set(model, diff, *object_id, value),
        Operation::DeleteManufacturingPlan {
            manufacturing_plan_id,
            ..
        } => apply_production_delete(model, diff, *manufacturing_plan_id),
        Operation::CreatePanelProjection {
            panel_projection_id,
            panel_projection,
        } => apply_production_create(
            model,
            diff,
            *panel_projection_id,
            panel_projection,
            SourceShardKind::PanelProjection,
            "manufacturing",
            "panel_projection",
        ),
        Operation::DeletePanelProjection {
            panel_projection_id,
            ..
        } => apply_production_delete(model, diff, *panel_projection_id),
        Operation::CreateOutputJob {
            output_job_id,
            output_job,
        } => apply_production_create(
            model,
            diff,
            *output_job_id,
            output_job,
            SourceShardKind::OutputJob,
            "output",
            "output_job",
        ),
        Operation::DeleteOutputJob { output_job_id, .. } => {
            apply_production_delete(model, diff, *output_job_id)
        }
        Operation::SetZoneFill {
            zone_id, zone_fill, ..
        } => {
            let fill = validated_zone_fill_payload(*zone_id, zone_fill)?;
            model.zone_fills.insert(*zone_id, fill);
            Ok(())
        }
        Operation::DeleteZoneFill { zone_id, zone_fill } => {
            validated_zone_fill_payload(*zone_id, zone_fill)?;
            model.zone_fills.remove(zone_id);
            Ok(())
        }
        Operation::CreateRelationship {
            relationship_id,
            relationship,
        } => apply_relationship_create(model, diff, *relationship_id, relationship),
        Operation::DeleteRelationship {
            relationship_id, ..
        } => apply_relationship_delete(model, diff, *relationship_id),
        Operation::SetRelationship {
            relationship_id,
            relationship,
            ..
        } => apply_relationship_set(model, diff, *relationship_id, relationship),
        Operation::CreateVariantOverlay {
            variant_id,
            variant,
        } => apply_variant_create(model, diff, *variant_id, variant),
        Operation::DeleteVariantOverlay { variant_id, .. } => {
            apply_variant_delete(model, diff, *variant_id)
        }
        Operation::SetVariantOverlay {
            variant_id,
            variant,
            ..
        } => apply_variant_set(model, diff, *variant_id, variant),
        _ => apply_operation_to_objects(&mut model.objects, operation, Some(diff)),
    }
}

fn schematic_sheet_payload_objects(
    relative_path: &str,
    sheet: &serde_json::Value,
) -> BTreeMap<uuid::Uuid, DomainObject> {
    let kind = SourceShardKind::SchematicSheet;
    let relative_path = format!("schematic/{relative_path}");
    let shard = SourceShardRef {
        shard_id: uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        authority: source_shard_authority_for_kind(&kind),
        taxon: source_shard_taxon_for_path(&kind, &relative_path),
        kind,
        path: PathBuf::from(&relative_path),
        relative_path,
        dirty_state: SourceShardDirtyState::Clean,
        schema_version: sheet
            .get("schema_version")
            .and_then(serde_json::Value::as_u64),
        content_hash: String::new(),
    };
    let mut objects = BTreeMap::new();
    let mut import_map = BTreeMap::new();
    collect_uuid_objects(sheet, &shard, "schematic", &mut objects, &mut import_map);
    objects
}
