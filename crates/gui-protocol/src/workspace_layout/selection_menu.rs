//! Source-qualified read-only leaves inside the existing marking-menu owner.
use crate::selection_resolution::{NativeSelectionResolution, SelectionAuthority};
use crate::selection_view::inspector_state::class_label;
use crate::{AuthoredSelectionIdentity as Id, PointNm, SelectionSubject};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Object(Id),
    Diagnostic(String),
    Run { origin: Id, point: PointNm },
    Semantic(Id),
    All,
    InspectScope(Option<crate::AuthoredSelectionClass>),
    InventoryToggle,
    FilterToggle(crate::AuthoredSelectionClass),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub label: String,
    pub command: Command,
    pub unavailable: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionMenu {
    pub project: crate::SelectionProjectId,
    pub revision: String,
    pub schematic: bool,
    pub source_scene: String,
    pub filters: crate::WorkspaceFilterState,
    pub inspection_subject: Option<SelectionSubject>,
    pub items: Vec<Item>,
    pub open: bool,
    pub focus: usize,
    pub scroll: usize,
}
pub fn resolve(
    authority: &NativeSelectionResolution,
    command: &Command,
) -> Result<Option<SelectionSubject>, crate::selection_resolution::SelectionResolutionError> {
    let subject = match command {
        Command::All
        | Command::InspectScope(_)
        | Command::InventoryToggle
        | Command::FilterToggle(_) => return Ok(None),
        Command::Diagnostic(_) => {
            return Err(
                crate::selection_resolution::SelectionResolutionError::MissingArtifactAuthority,
            );
        }
        Command::Object(id) => {
            if !authority.authored_contains(id) {
                return Err(
                    crate::selection_resolution::SelectionResolutionError::UnresolvedDerivedMember(
                        id.clone(),
                    ),
                );
            }
            SelectionSubject::Object(id.clone())
        }
        Command::Run { origin, point } => authority.acquire_run(
            origin.clone(),
            Some(eda_engine::ir::geometry::Point::new(point.x, point.y)),
        )?,
        Command::Semantic(id) => authority.semantic_subject_for_origin(id)?,
    };
    authority.resolve(&subject)?;
    Ok(Some(subject))
}
impl SelectionMenu {
    /// Complete eligible local overlap inventory captured before menu opening.
    /// Semantic leaves, findings and inspection controls add no authored members.
    /// This is not the document-wide Ctrl+A eligibility bypass.
    pub fn overlap_candidates(&self) -> impl Iterator<Item = &Id> {
        self.items.iter().filter_map(|item| match &item.command {
            Command::Object(id) if item.unavailable.is_none() => Some(id),
            _ => None,
        })
    }
    /// Ordered eligible acquired/source pairs from the actual shared hit index.
    /// Deduplication is by complete authored identity, never display keys.
    pub fn capture(
        authority: &NativeSelectionResolution,
        origins: Vec<(Id, Id)>,
        point: PointNm,
        filters: crate::WorkspaceFilterState,
    ) -> Self {
        let mut items = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for (acquired, _) in &origins {
            if seen.insert(acquired.clone()) {
                items.push(Item {
                    label: format!("Select {} {}", class_label(acquired.class), acquired.id),
                    command: Command::Object(acquired.clone()),
                    unavailable: None,
                });
            }
        }
        if let Some((acquired, projected)) = origins.first() {
            if acquired.can_originate_run() {
                items.push(Item {
                    label: "Select Run".into(),
                    command: Command::Run {
                        origin: acquired.clone(),
                        point,
                    },
                    unavailable: None,
                });
            }
            use crate::AuthoredSelectionClass as C;
            if matches!(
                projected.class,
                C::Track
                    | C::Via
                    | C::Zone
                    | C::Pad
                    | C::Wire
                    | C::Pin
                    | C::Label
                    | C::Port
                    | C::Junction
                    | C::BusSection
            ) {
                items.push(Item {
                    label: if projected.class == C::BusSection {
                        "Select Bus"
                    } else {
                        "Select Global Net"
                    }
                    .into(),
                    command: Command::Semantic(projected.clone()),
                    unavailable: None,
                });
            }
        }
        items.push(Item {
            label: "Select All".into(),
            command: Command::All,
            unavailable: None,
        });
        for item in &mut items {
            if let Err(error) = resolve(authority, &item.command) {
                item.unavailable = Some(format!("{error:?}"));
            }
        }
        Self {
            project: authority.project(),
            revision: authority.revision().0.clone(),
            schematic: false,
            source_scene: String::new(),
            filters,
            inspection_subject: None,
            items,
            open: false,
            focus: 0,
            scroll: 0,
        }
    }
}

impl SelectionMenu {
    pub fn append_inspection(&mut self, view: &crate::selection_view::SelectionView) {
        self.inspection_subject = Some(view.subject.clone());
        for scope in &view.inspection {
            self.items.push(Item {
                label: format!(
                    "Inspect {}",
                    scope
                        .scope
                        .map_or("All".to_owned(), |c| class_label(c).to_owned())
                ),
                command: Command::InspectScope(scope.scope),
                unavailable: None,
            });
        }
        self.items.push(Item {
            label: "Expand/collapse complete inventory".into(),
            command: Command::InventoryToggle,
            unavailable: None,
        });
        for class in crate::selection_view::inspector_state::ACQUISITION_CLASSES {
            self.items.push(Item {
                label: format!(
                    "{} {} acquisition",
                    if self.filters.excluded_selection_classes.contains(&class) {
                        "Enable"
                    } else {
                        "Exclude"
                    },
                    class_label(class)
                ),
                command: Command::FilterToggle(class),
                unavailable: None,
            });
        }
    }
}

impl SelectionMenu {
    /// Explicit Checks entries use producing fingerprints, not target objects.
    /// The complete list is keyboard reachable and independent of context caps.
    pub fn append_checks(&mut self, checks: &crate::CheckRunReviewState) {
        if checks.project_id.as_deref() != Some(&self.project.to_string()) {
            return;
        }
        let mut counts = std::collections::BTreeMap::new();
        for finding in &checks.findings {
            *counts.entry(&finding.fingerprint).or_insert(0usize) += 1;
        }
        for finding in &checks.findings {
            self.items.push(Item {
                label: format!(
                    "Checks: {} {} — {}",
                    finding.severity, finding.rule_id, finding.fingerprint
                ),
                command: Command::Diagnostic(finding.fingerprint.clone()),
                unavailable: (finding.fingerprint.is_empty()
                    || counts.get(&finding.fingerprint) != Some(&1))
                .then(|| "Finding fingerprint is unavailable or ambiguous".into()),
            });
        }
    }
}
