//! Side-effect-free Global Preferences product queries.

use std::collections::BTreeMap;

use serde_json::json;

use super::product_service::{control_view, effect_timing, field_details};
use super::{
    DescribeResultV1, ExplainResultV1, GetResultV1, GlobalPreferenceRow,
    GlobalPreferencesProductService, ListResultV1, PreferenceControlDescriptorV1,
    PreferenceErrorCodeV1, PreferenceErrorV1, PreferenceMatchKindV1, PreferenceQueryResultV1,
    PreferenceQueryV1, PreferenceSearchMatchV1, PreferenceSectionViewV1, PreferenceValueViewV1,
    SearchResultV1,
};

impl GlobalPreferencesProductService {
    pub fn rows(&self) -> Vec<GlobalPreferenceRow> {
        self.service.rows()
    }

    pub fn surface(&self) -> &super::PreferenceSurfaceCatalog {
        self.service.surface()
    }

    pub fn registry(&self) -> &super::DescriptorRegistry {
        self.service.registry()
    }

    pub fn status(&self) -> &super::PreferenceServiceStatus {
        self.service.status()
    }

    pub fn legacy_migration(&self) -> &super::LegacyConsoleMigrationState {
        self.service.legacy_migration()
    }

    pub fn query(
        &self,
        query: PreferenceQueryV1,
    ) -> Result<PreferenceQueryResultV1, PreferenceErrorV1> {
        match query {
            PreferenceQueryV1::Describe => Ok(PreferenceQueryResultV1::Describe(self.describe()?)),
            PreferenceQueryV1::List { section } => Ok(PreferenceQueryResultV1::List(
                self.list(section.as_deref())?,
            )),
            PreferenceQueryV1::Get { key } => Ok(PreferenceQueryResultV1::Get(GetResultV1 {
                value: self.value_for_key(&key)?,
            })),
            PreferenceQueryV1::Search { query } => {
                Ok(PreferenceQueryResultV1::Search(self.search(&query)?))
            }
            PreferenceQueryV1::Explain { key } => {
                let row = self.row_for_key(&key)?;
                Ok(PreferenceQueryResultV1::Explain(ExplainResultV1 {
                    explanation: self.explanation(&row)?,
                }))
            }
            PreferenceQueryV1::PreviewProjectUnitsSeed { source } => Ok(
                PreferenceQueryResultV1::PreviewProjectUnitsSeed(self.preview_units_seed(source)?),
            ),
        }
    }

    fn describe(&self) -> Result<DescribeResultV1, PreferenceErrorV1> {
        let sections = self
            .service
            .surface()
            .sections()
            .iter()
            .map(|section| PreferenceSectionViewV1 {
                id: section.id.as_str().to_owned(),
                label: section.label.clone(),
                order: section.order,
            })
            .collect();
        let controls = self
            .service
            .surface()
            .entries()
            .iter()
            .map(|entry| {
                let descriptor = self
                    .service
                    .registry()
                    .get(&entry.key)
                    .expect("catalog key");
                PreferenceControlDescriptorV1 {
                    key: entry.key.as_str().to_owned(),
                    label: descriptor.presentation.label.clone(),
                    description: descriptor.presentation.description.clone(),
                    section_id: entry.section.as_str().to_owned(),
                    row_order: entry.row_order,
                    control: control_view(&entry.control),
                    default_value: descriptor.default_value.literal().cloned(),
                    effect_timing: effect_timing(descriptor.apply_behavior).to_owned(),
                }
            })
            .collect();
        Ok(DescribeResultV1 {
            sections,
            controls,
            active_catalog_digest: self.active_catalog_digest.clone(),
        })
    }

    fn list(&self, section: Option<&str>) -> Result<ListResultV1, PreferenceErrorV1> {
        if let Some(section) = section
            && !self
                .service
                .surface()
                .sections()
                .iter()
                .any(|item| item.id.as_str() == section)
        {
            return Err(self.error(
                PreferenceErrorCodeV1::UnknownSection,
                "Section is not active",
                BTreeMap::from([("section".to_owned(), json!(section))]),
                None,
            ));
        }
        let values = self
            .service
            .rows()
            .iter()
            .filter(|row| {
                section.is_none_or(|section| self.entry(&row.key).section.as_str() == section)
            })
            .map(|row| self.value_view(row))
            .collect::<Result<_, _>>()?;
        Ok(ListResultV1 { values })
    }

    fn search(&self, query: &str) -> Result<SearchResultV1, PreferenceErrorV1> {
        let needle = query.trim().to_ascii_lowercase();
        if needle.is_empty() {
            return Err(self.error(
                PreferenceErrorCodeV1::InvalidQuery,
                "Search query must not be empty",
                field_details("query", "empty"),
                None,
            ));
        }
        let rows = self.service.rows();
        let mut matches = Vec::new();
        for entry in self.service.surface().entries() {
            let descriptor = self
                .service
                .registry()
                .get(&entry.key)
                .expect("catalog key");
            let (match_kind, matched_vocabulary) = if descriptor
                .presentation
                .label
                .to_ascii_lowercase()
                .contains(&needle)
            {
                (PreferenceMatchKindV1::Label, None)
            } else if descriptor
                .presentation
                .description
                .to_ascii_lowercase()
                .contains(&needle)
            {
                (PreferenceMatchKindV1::Description, None)
            } else if entry.key.as_str().to_ascii_lowercase().contains(&needle) {
                (
                    PreferenceMatchKindV1::StableKey,
                    Some(entry.key.as_str().to_owned()),
                )
            } else if let Some(alias) = descriptor
                .retired_aliases
                .iter()
                .find(|alias| alias.to_ascii_lowercase().contains(&needle))
            {
                (PreferenceMatchKindV1::RegisteredAlias, Some(alias.clone()))
            } else {
                continue;
            };
            let row = rows
                .iter()
                .find(|row| row.key == entry.key)
                .expect("catalog row");
            matches.push(PreferenceSearchMatchV1 {
                value: self.value_view(row)?,
                match_kind,
                matched_vocabulary,
            });
        }
        Ok(SearchResultV1 {
            query: query.to_owned(),
            matches,
        })
    }

    pub(super) fn value_for_key(
        &self,
        key: &str,
    ) -> Result<PreferenceValueViewV1, PreferenceErrorV1> {
        let row = self.row_for_key(key)?;
        self.value_view(&row)
    }

    pub(super) fn row_for_key(&self, key: &str) -> Result<GlobalPreferenceRow, PreferenceErrorV1> {
        let parsed = self.active_key(key, None)?;
        self.service
            .rows()
            .into_iter()
            .find(|row| row.key == parsed)
            .ok_or_else(|| self.unknown_key(key, None))
    }
}
