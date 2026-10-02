//! One read-only engine adapter for board/schematic selection consumers.
use super::*;
use crate::selection_subject::{AuthoredSelectionClass as Class, RunSelectionIdentity};
use eda_engine::{
    connectivity::{BoardRunMembership, ZoneRegionSuccessor},
    ir::geometry::Point,
    substrate::{
        AuthoredSelectionSource, BusRunMembership, DesignModel, ElectricalMembership,
        ElectricalOccurrence, ElectricalSelectionSnapshot, SelectionSourceBasis,
        SelectionSourceDomain,
    },
};

/// Complete membership and source-qualified owned/related projection evidence.
/// Bus entry geometry is carried by the engine result and never counted as a
/// separately acquired authored member in the S5A Bus subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSelectionProjection {
    pub project: ObjectId,
    pub revision: SelectionModelRevision,
    pub source_basis: Vec<SelectionSourceBasis>,
    pub members: BTreeSet<AuthoredSelectionIdentity>,
    pub related: BTreeSet<AuthoredSelectionIdentity>,
    pub electrical: Option<ElectricalMembership>,
    pub board_run: Option<BoardRunMembership>,
    pub bus_run: Option<BusRunMembership>,
}

/// Immutable per-model-basis authority, shared by panes. It has no selected
/// state, writer, visibility input, scene index or capped context list.
pub struct NativeSelectionResolution {
    snapshot: ElectricalSelectionSnapshot,
    authored: BTreeSet<AuthoredSelectionIdentity>,
}
impl NativeSelectionResolution {
    pub fn capture(model: &DesignModel) -> Result<Self, SelectionResolutionError> {
        Self::from_snapshot(
            ElectricalSelectionSnapshot::capture(model)
                .map_err(SelectionResolutionError::Engine)?,
        )
    }
    pub fn from_snapshot(
        snapshot: ElectricalSelectionSnapshot,
    ) -> Result<Self, SelectionResolutionError> {
        let sources = snapshot
            .authored_sources()
            .map_err(SelectionResolutionError::Engine)?;
        let mut authored = BTreeSet::new();
        for source in sources {
            if source.occurrence.class == "bus_entries" {
                continue;
            }
            authored.insert(source_identity(&source)?);
        }
        Ok(Self { snapshot, authored })
    }
    pub fn snapshot(&self) -> &ElectricalSelectionSnapshot {
        &self.snapshot
    }

    /// Hit acquisition issues an engine qualifier only for the actual Zone
    /// region. Nonpoint ambiguity refuses without altering consumer state.
    pub fn acquire_run(
        &self,
        origin: AuthoredSelectionIdentity,
        hit: Option<Point>,
    ) -> Result<SelectionSubject, SelectionResolutionError> {
        let mut run = RunSelectionIdentity::from(origin);
        let subject = SelectionSubject::Run(run.clone());
        subject
            .validate()
            .map_err(SelectionResolutionError::InvalidSubject)?;
        if matches!(run.origin.class, Class::Track | Class::Via | Class::Zone) {
            let result = self
                .snapshot
                .board_run(self.revision(), &occurrence(&run.origin), hit)
                .map_err(SelectionResolutionError::Engine)?;
            if run.origin.class == Class::Zone {
                let qualifiers: Vec<_> = result
                    .zone_regions
                    .into_iter()
                    .filter(|q| q.source().source_id == run.origin.id)
                    .collect();
                if qualifiers.len() != 1 {
                    return Err(SelectionResolutionError::MissingDerivedAuthority);
                }
                run.zone_region = Some(qualifiers[0].clone());
            }
        } else {
            self.resolve(&subject)?;
        }
        Ok(SelectionSubject::Run(run))
    }

    pub fn region_successor(
        &self,
        previous: &Self,
        run: &RunSelectionIdentity,
    ) -> Result<ZoneRegionSuccessor, SelectionResolutionError> {
        let qualifier = run
            .zone_region
            .as_ref()
            .ok_or(SelectionResolutionError::MissingDerivedAuthority)?;
        self.snapshot
            .zone_region_successor(self.revision(), &previous.snapshot, qualifier)
            .map_err(SelectionResolutionError::Engine)
    }

    pub fn resolve(
        &self,
        subject: &SelectionSubject,
    ) -> Result<NativeSelectionProjection, SelectionResolutionError> {
        subject
            .validate()
            .map_err(SelectionResolutionError::InvalidSubject)?;
        let mut projection = NativeSelectionProjection {
            project: self.project(),
            revision: self.revision().clone(),
            source_basis: self.snapshot.source_basis().to_vec(),
            members: BTreeSet::new(),
            related: BTreeSet::new(),
            electrical: None,
            board_run: None,
            bus_run: None,
        };
        match subject {
            SelectionSubject::None => {}
            SelectionSubject::Object(id) => {
                if self.authored_contains(id) {
                    projection.members.insert(id.clone());
                }
            }
            SelectionSubject::Compound(value) => {
                projection.members = value
                    .members
                    .intersection(&self.authored)
                    .cloned()
                    .collect()
            }
            SelectionSubject::GlobalNet(id) => {
                projection.electrical = Some(
                    self.snapshot
                        .global_net(self.revision(), *id)
                        .map_err(SelectionResolutionError::Engine)?,
                )
            }
            SelectionSubject::Bus(id) => {
                projection.electrical = Some(
                    self.snapshot
                        .bus(self.revision(), *id)
                        .map_err(SelectionResolutionError::Engine)?,
                )
            }
            SelectionSubject::Run(run) => match run.origin.class {
                Class::Wire => {
                    projection.electrical = Some(
                        self.snapshot
                            .schematic_run(self.revision(), &occurrence(&run.origin))
                            .map_err(SelectionResolutionError::Engine)?,
                    )
                }
                Class::BusSection => {
                    let result = self
                        .snapshot
                        .bus_run(self.revision(), &occurrence(&run.origin))
                        .map_err(SelectionResolutionError::Engine)?;
                    projection.electrical = Some(result.membership.clone());
                    projection.bus_run = Some(result);
                }
                Class::Track | Class::Via => {
                    projection.board_run = Some(
                        self.snapshot
                            .board_run(self.revision(), &occurrence(&run.origin), None)
                            .map_err(SelectionResolutionError::Engine)?,
                    )
                }
                Class::Zone => match self.region_successor(self, run)? {
                    ZoneRegionSuccessor::Unique { run, .. } => projection.board_run = Some(*run),
                    _ => return Err(SelectionResolutionError::MissingDerivedAuthority),
                },
                _ => {
                    return Err(SelectionResolutionError::InvalidSubject(
                        SelectionSubjectError::InvalidRunOrigin,
                    ));
                }
            },
            SelectionSubject::Proposal(_)
            | SelectionSubject::Review(_)
            | SelectionSubject::Diagnostic(_) => {
                return Err(SelectionResolutionError::MissingArtifactAuthority);
            }
        }
        if let Some(result) = &projection.electrical {
            projection.members = self.identities(&result.owned)?;
            projection.related = self.identities(&result.related)?;
        }
        if let Some(result) = &projection.board_run {
            projection.members = self.identities(&result.members)?;
        }
        Ok(projection)
    }

    fn identities(
        &self,
        occurrences: &BTreeSet<ElectricalOccurrence>,
    ) -> Result<BTreeSet<AuthoredSelectionIdentity>, SelectionResolutionError> {
        let mut result = BTreeSet::new();
        for source in occurrences {
            if source.class == "bus_entries" {
                continue;
            } // owned projection retained above
            let domain = match source.class.as_str() {
                "packages" | "pads" | "tracks" | "vias" | "zones" => SelectionSourceDomain::Board,
                "symbols" | "pins" | "wires" | "buses" | "labels" | "ports" | "junctions" => {
                    SelectionSourceDomain::Schematic
                }
                _ => {
                    return Err(SelectionResolutionError::UnsupportedEngineClass(
                        source.class.clone(),
                    ));
                }
            };
            let id = source_identity(&AuthoredSelectionSource {
                domain,
                occurrence: source.clone(),
            })?;
            if !self.authored_contains(&id) {
                return Err(SelectionResolutionError::UnresolvedDerivedMember(id));
            }
            result.insert(id);
        }
        Ok(result)
    }
}
impl SelectionAuthority for NativeSelectionResolution {
    fn project(&self) -> ObjectId {
        self.snapshot.project_id()
    }
    fn revision(&self) -> &SelectionModelRevision {
        self.snapshot.revision()
    }
    fn authored_contains(&self, id: &AuthoredSelectionIdentity) -> bool {
        self.authored.contains(id)
    }
    fn derived_exists(&self, subject: &SelectionSubject) -> Result<bool, SelectionResolutionError> {
        Ok(match subject {
            SelectionSubject::GlobalNet(id) => self.snapshot.net_ids().contains(id),
            SelectionSubject::Bus(id) => self.snapshot.bus_ids().contains(id),
            _ => true,
        })
    }
    fn artifact_exists(
        &self,
        subject: &SelectionSubject,
    ) -> Result<bool, SelectionResolutionError> {
        match subject {
            SelectionSubject::Proposal(_)
            | SelectionSubject::Review(_)
            | SelectionSubject::Diagnostic(_) => {
                Err(SelectionResolutionError::MissingArtifactAuthority)
            }
            _ => Ok(true),
        }
    }
    fn members(
        &self,
        subject: &SelectionSubject,
    ) -> Result<BTreeSet<AuthoredSelectionIdentity>, SelectionResolutionError> {
        Ok(self.resolve(subject)?.members)
    }
}

fn source_identity(
    source: &AuthoredSelectionSource,
) -> Result<AuthoredSelectionIdentity, SelectionResolutionError> {
    let class = match (source.domain, source.occurrence.class.as_str()) {
        (SelectionSourceDomain::Board, "packages") => Class::Footprint,
        (SelectionSourceDomain::Board, "pads") => Class::Pad,
        (SelectionSourceDomain::Board, "tracks") => Class::Track,
        (SelectionSourceDomain::Board, "vias") => Class::Via,
        (SelectionSourceDomain::Board, "zones") => Class::Zone,
        (SelectionSourceDomain::Board, "texts") => Class::BoardText,
        (SelectionSourceDomain::Board, "outline") => Class::BoardOutline,
        (SelectionSourceDomain::Schematic, "symbols") => Class::Symbol,
        (SelectionSourceDomain::Schematic, "pins") => Class::Pin,
        (SelectionSourceDomain::Schematic, "wires") => Class::Wire,
        (SelectionSourceDomain::Schematic, "buses") => Class::BusSection,
        (SelectionSourceDomain::Schematic, "labels") => Class::Label,
        (SelectionSourceDomain::Schematic, "ports") => Class::Port,
        (SelectionSourceDomain::Schematic, "junctions") => Class::Junction,
        (SelectionSourceDomain::Schematic, "noconnects") => Class::NoConnect,
        (SelectionSourceDomain::Schematic, "texts") => Class::SchematicText,
        (SelectionSourceDomain::Schematic, "drawings") => Class::SchematicGraphic,
        _ => {
            return Err(SelectionResolutionError::UnsupportedEngineClass(
                source.occurrence.class.clone(),
            ));
        }
    };
    Ok(AuthoredSelectionIdentity {
        class,
        id: source.occurrence.source_id,
        instance_path: source.occurrence.instance_path.clone(),
    })
}
fn occurrence(id: &AuthoredSelectionIdentity) -> ElectricalOccurrence {
    let class = match id.class {
        Class::Track => "tracks",
        Class::Via => "vias",
        Class::Zone => "zones",
        Class::Wire => "wires",
        Class::BusSection => "buses",
        _ => "invalid_origin",
    };
    ElectricalOccurrence {
        class: class.into(),
        source_id: id.id,
        instance_path: id.instance_path.clone(),
    }
}
