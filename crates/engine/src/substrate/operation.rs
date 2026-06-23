use serde::{Deserialize, Serialize};

use super::ObjectId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Operation {
    BumpObjectRevision {
        object_id: ObjectId,
    },
    SetProjectName {
        project_id: ObjectId,
        name: String,
    },
    AddProjectPoolRef {
        path: String,
        priority: u32,
    },
    DeleteProjectPoolRef {
        path: String,
        priority: u32,
    },
    SetProjectRules {
        rules: Vec<serde_json::Value>,
    },
    CreateProjectRule {
        rules_root_id: ObjectId,
        rule_id: ObjectId,
        rule: serde_json::Value,
    },
    SetProjectRule {
        rules_root_id: ObjectId,
        rule_id: ObjectId,
        rule: serde_json::Value,
    },
    DeleteProjectRule {
        rules_root_id: ObjectId,
        rule_id: ObjectId,
        rule: serde_json::Value,
    },
    CreateBoardPackage {
        package_id: ObjectId,
        package: serde_json::Value,
        materialized: serde_json::Value,
    },
    DeleteBoardPackage {
        package_id: ObjectId,
        package: serde_json::Value,
        materialized: serde_json::Value,
    },
    SetBoardPackagePart {
        package_id: ObjectId,
        part_id: ObjectId,
    },
    SetBoardPackagePackage {
        package_id: ObjectId,
        package_ref_id: ObjectId,
        previous_materialized: serde_json::Value,
        materialized: serde_json::Value,
    },
    SetBoardPackageValue {
        package_id: ObjectId,
        value: String,
    },
    SetBoardPackageReference {
        package_id: ObjectId,
        reference: String,
    },
    SetBoardPackagePosition {
        package_id: ObjectId,
        x: i64,
        y: i64,
    },
    SetBoardPackageLayer {
        package_id: ObjectId,
        layer: i32,
    },
    SetComponentSide {
        package_id: ObjectId,
        layer: i32,
    },
    SetBoardPackageRotation {
        package_id: ObjectId,
        rotation: i32,
    },
    SetBoardPackageLocked {
        package_id: ObjectId,
        locked: bool,
    },
    CreateBoardPad {
        pad_id: ObjectId,
        pad: serde_json::Value,
    },
    SetBoardPad {
        pad_id: ObjectId,
        pad: serde_json::Value,
    },
    DeleteBoardPad {
        pad_id: ObjectId,
        pad: serde_json::Value,
    },
    CreateBoardTrack {
        track_id: ObjectId,
        track: serde_json::Value,
    },
    SetBoardTrack {
        track_id: ObjectId,
        track: serde_json::Value,
    },
    DeleteBoardTrack {
        track_id: ObjectId,
        track: serde_json::Value,
    },
    CreateBoardVia {
        via_id: ObjectId,
        via: serde_json::Value,
    },
    SetBoardVia {
        via_id: ObjectId,
        via: serde_json::Value,
    },
    DeleteBoardVia {
        via_id: ObjectId,
        via: serde_json::Value,
    },
    CreateBoardZone {
        zone_id: ObjectId,
        zone: serde_json::Value,
    },
    SetBoardZone {
        zone_id: ObjectId,
        zone: serde_json::Value,
    },
    DeleteBoardZone {
        zone_id: ObjectId,
        zone: serde_json::Value,
    },
    CreateBoardNet {
        net_id: ObjectId,
        net: serde_json::Value,
    },
    SetBoardNet {
        net_id: ObjectId,
        net: serde_json::Value,
    },
    DeleteBoardNet {
        net_id: ObjectId,
        net: serde_json::Value,
    },
    CreateBoardNetClass {
        net_class_id: ObjectId,
        net_class: serde_json::Value,
    },
    SetBoardNetClass {
        net_class_id: ObjectId,
        net_class: serde_json::Value,
    },
    DeleteBoardNetClass {
        net_class_id: ObjectId,
        net_class: serde_json::Value,
    },
    CreateBoardDimension {
        dimension_id: ObjectId,
        dimension: serde_json::Value,
    },
    SetBoardDimension {
        dimension_id: ObjectId,
        dimension: serde_json::Value,
    },
    DeleteBoardDimension {
        dimension_id: ObjectId,
        dimension: serde_json::Value,
    },
    CreateBoardText {
        text_id: ObjectId,
        text: serde_json::Value,
    },
    SetBoardText {
        text_id: ObjectId,
        text: serde_json::Value,
    },
    DeleteBoardText {
        text_id: ObjectId,
        text: serde_json::Value,
    },
    CreateBoardKeepout {
        keepout_id: ObjectId,
        keepout: serde_json::Value,
    },
    SetBoardKeepout {
        keepout_id: ObjectId,
        keepout: serde_json::Value,
    },
    DeleteBoardKeepout {
        keepout_id: ObjectId,
        keepout: serde_json::Value,
    },
    SetBoardOutline {
        board_id: ObjectId,
        outline: serde_json::Value,
    },
    SetBoardStackup {
        board_id: ObjectId,
        stackup: serde_json::Value,
    },
    SetBoardName {
        board_id: ObjectId,
        name: String,
    },
    CreatePoolPackage {
        package_id: ObjectId,
        relative_path: String,
        package: serde_json::Value,
    },
    DeletePoolPackage {
        package_id: ObjectId,
        relative_path: String,
        package: serde_json::Value,
    },
    CreatePoolPadstack {
        padstack_id: ObjectId,
        relative_path: String,
        padstack: serde_json::Value,
    },
    DeletePoolPadstack {
        padstack_id: ObjectId,
        relative_path: String,
        padstack: serde_json::Value,
    },
    CreatePoolLibraryObject {
        object_id: ObjectId,
        relative_path: String,
        object_kind: String,
        object: serde_json::Value,
    },
    SetPoolLibraryObject {
        object_id: ObjectId,
        relative_path: String,
        object_kind: String,
        previous_object: serde_json::Value,
        object: serde_json::Value,
    },
    AttachPoolPartModel {
        part_id: ObjectId,
        relative_path: String,
        previous_attachments: Vec<serde_json::Value>,
        attachments: Vec<serde_json::Value>,
    },
    DetachPoolPartModel {
        part_id: ObjectId,
        relative_path: String,
        previous_attachments: Vec<serde_json::Value>,
        attachments: Vec<serde_json::Value>,
    },
    DeletePoolLibraryObject {
        object_id: ObjectId,
        relative_path: String,
        object_kind: String,
        object: serde_json::Value,
    },
    CreateImportMapShard {
        relative_path: String,
        shard: serde_json::Value,
    },
    DeleteImportMapShard {
        relative_path: String,
        shard: serde_json::Value,
    },
    CreateProposalMetadata {
        proposal_id: ObjectId,
        relative_path: String,
        proposal: serde_json::Value,
    },
    SetProposalMetadata {
        proposal_id: ObjectId,
        relative_path: String,
        previous_proposal: serde_json::Value,
        proposal: serde_json::Value,
    },
    DeleteProposalMetadata {
        proposal_id: ObjectId,
        relative_path: String,
        proposal: serde_json::Value,
    },
    CreateManufacturingPlan {
        manufacturing_plan_id: ObjectId,
        manufacturing_plan: serde_json::Value,
    },
    SetManufacturingPlan {
        manufacturing_plan_id: ObjectId,
        previous_manufacturing_plan: serde_json::Value,
        manufacturing_plan: serde_json::Value,
    },
    DeleteManufacturingPlan {
        manufacturing_plan_id: ObjectId,
        manufacturing_plan: serde_json::Value,
    },
    CreatePanelProjection {
        panel_projection_id: ObjectId,
        panel_projection: serde_json::Value,
    },
    SetPanelProjection {
        panel_projection_id: ObjectId,
        previous_panel_projection: serde_json::Value,
        panel_projection: serde_json::Value,
    },
    DeletePanelProjection {
        panel_projection_id: ObjectId,
        panel_projection: serde_json::Value,
    },
    CreateOutputJob {
        output_job_id: ObjectId,
        output_job: serde_json::Value,
    },
    SetOutputJob {
        output_job_id: ObjectId,
        previous_output_job: serde_json::Value,
        output_job: serde_json::Value,
    },
    DeleteOutputJob {
        output_job_id: ObjectId,
        output_job: serde_json::Value,
    },
    SetZoneFill {
        zone_id: ObjectId,
        previous_zone_fill: Option<serde_json::Value>,
        zone_fill: serde_json::Value,
    },
    DeleteZoneFill {
        zone_id: ObjectId,
        zone_fill: serde_json::Value,
    },
    CreateRelationship {
        relationship_id: ObjectId,
        relationship: serde_json::Value,
    },
    DeleteRelationship {
        relationship_id: ObjectId,
        relationship: serde_json::Value,
    },
    SetRelationship {
        relationship_id: ObjectId,
        previous_relationship: serde_json::Value,
        relationship: serde_json::Value,
    },
    CreateVariantOverlay {
        variant_id: ObjectId,
        variant: serde_json::Value,
    },
    DeleteVariantOverlay {
        variant_id: ObjectId,
        variant: serde_json::Value,
    },
    SetVariantOverlay {
        variant_id: ObjectId,
        previous_variant: serde_json::Value,
        variant: serde_json::Value,
    },
    CreateComponentInstance {
        component_instance_id: ObjectId,
        component_instance: serde_json::Value,
    },
    DeleteComponentInstance {
        component_instance_id: ObjectId,
        component_instance: serde_json::Value,
    },
    SetComponentInstance {
        component_instance_id: ObjectId,
        previous_component_instance: serde_json::Value,
        component_instance: serde_json::Value,
    },
    CreateSchematicWaiver {
        schematic_id: ObjectId,
        waiver_id: ObjectId,
        waiver: serde_json::Value,
    },
    DeleteSchematicWaiver {
        schematic_id: ObjectId,
        waiver_id: ObjectId,
        waiver: serde_json::Value,
    },
    CreateSchematicDeviation {
        schematic_id: ObjectId,
        deviation_id: ObjectId,
        deviation: serde_json::Value,
    },
    DeleteSchematicDeviation {
        schematic_id: ObjectId,
        deviation_id: ObjectId,
        deviation: serde_json::Value,
    },
    CreateSchematicSheet {
        schematic_id: ObjectId,
        sheet_id: ObjectId,
        relative_path: String,
        sheet: serde_json::Value,
    },
    DeleteSchematicSheet {
        schematic_id: ObjectId,
        sheet_id: ObjectId,
        relative_path: String,
        sheet: serde_json::Value,
    },
    CreateSchematicDefinition {
        schematic_id: ObjectId,
        definition_id: ObjectId,
        relative_path: String,
        definition: serde_json::Value,
    },
    DeleteSchematicDefinition {
        schematic_id: ObjectId,
        definition_id: ObjectId,
        relative_path: String,
        definition: serde_json::Value,
    },
    CreateSchematicSheetInstance {
        schematic_id: ObjectId,
        instance_id: ObjectId,
        instance: serde_json::Value,
    },
    DeleteSchematicSheetInstance {
        schematic_id: ObjectId,
        instance_id: ObjectId,
        instance: serde_json::Value,
    },
    SetSchematicSheetInstance {
        schematic_id: ObjectId,
        instance_id: ObjectId,
        previous_instance: serde_json::Value,
        instance: serde_json::Value,
    },
    SetSchematicSheetName {
        sheet_id: ObjectId,
        name: String,
    },
    CreateSchematicWire {
        sheet_id: ObjectId,
        wire_id: ObjectId,
        wire: serde_json::Value,
    },
    DeleteSchematicWire {
        sheet_id: ObjectId,
        wire_id: ObjectId,
        wire: serde_json::Value,
    },
    CreateSchematicJunction {
        sheet_id: ObjectId,
        junction_id: ObjectId,
        junction: serde_json::Value,
    },
    DeleteSchematicJunction {
        sheet_id: ObjectId,
        junction_id: ObjectId,
        junction: serde_json::Value,
    },
    CreateSchematicNoConnect {
        sheet_id: ObjectId,
        noconnect_id: ObjectId,
        noconnect: serde_json::Value,
    },
    DeleteSchematicNoConnect {
        sheet_id: ObjectId,
        noconnect_id: ObjectId,
        noconnect: serde_json::Value,
    },
    CreateSchematicLabel {
        sheet_id: ObjectId,
        label_id: ObjectId,
        label: serde_json::Value,
    },
    SetSchematicLabel {
        sheet_id: ObjectId,
        label_id: ObjectId,
        label: serde_json::Value,
    },
    DeleteSchematicLabel {
        sheet_id: ObjectId,
        label_id: ObjectId,
        label: serde_json::Value,
    },
    CreateSchematicPort {
        sheet_id: ObjectId,
        port_id: ObjectId,
        port: serde_json::Value,
    },
    SetSchematicPort {
        sheet_id: ObjectId,
        port_id: ObjectId,
        port: serde_json::Value,
    },
    DeleteSchematicPort {
        sheet_id: ObjectId,
        port_id: ObjectId,
        port: serde_json::Value,
    },
    CreateSchematicBus {
        sheet_id: ObjectId,
        bus_id: ObjectId,
        bus: serde_json::Value,
    },
    SetSchematicBus {
        sheet_id: ObjectId,
        bus_id: ObjectId,
        bus: serde_json::Value,
    },
    DeleteSchematicBus {
        sheet_id: ObjectId,
        bus_id: ObjectId,
        bus: serde_json::Value,
    },
    CreateSchematicBusEntry {
        sheet_id: ObjectId,
        bus_entry_id: ObjectId,
        bus_entry: serde_json::Value,
    },
    DeleteSchematicBusEntry {
        sheet_id: ObjectId,
        bus_entry_id: ObjectId,
        bus_entry: serde_json::Value,
    },
    CreateSchematicText {
        sheet_id: ObjectId,
        text_id: ObjectId,
        text: serde_json::Value,
    },
    SetSchematicText {
        sheet_id: ObjectId,
        text_id: ObjectId,
        text: serde_json::Value,
    },
    DeleteSchematicText {
        sheet_id: ObjectId,
        text_id: ObjectId,
        text: serde_json::Value,
    },
    CreateSchematicDrawing {
        sheet_id: ObjectId,
        drawing_id: ObjectId,
        drawing: serde_json::Value,
    },
    SetSchematicDrawing {
        sheet_id: ObjectId,
        drawing_id: ObjectId,
        drawing: serde_json::Value,
    },
    DeleteSchematicDrawing {
        sheet_id: ObjectId,
        drawing_id: ObjectId,
        drawing: serde_json::Value,
    },
    CreateSchematicSymbol {
        sheet_id: ObjectId,
        symbol_id: ObjectId,
        symbol: serde_json::Value,
    },
    SetSchematicSymbol {
        sheet_id: ObjectId,
        symbol_id: ObjectId,
        symbol: serde_json::Value,
    },
    DeleteSchematicSymbol {
        sheet_id: ObjectId,
        symbol_id: ObjectId,
        symbol: serde_json::Value,
    },
}

impl Operation {
    pub fn writes_project_shard(&self) -> bool {
        !matches!(self, Operation::BumpObjectRevision { .. })
    }
}
