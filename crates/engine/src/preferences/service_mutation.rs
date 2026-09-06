//! Global Preferences User-partition mutation ownership.

use serde_json::Value;

use super::repository::{GenerationRef, PreferenceMutation, PreferencePartition};
use super::{
    GlobalPreferenceRow, GlobalPreferencesService, PreferenceKey, PreferenceServiceRefusal,
    PreferenceServiceRefusalKind, PreferenceServiceStatus,
};

impl GlobalPreferencesService {
    pub fn set_user(
        &mut self,
        key: PreferenceKey,
        value: Value,
        expected: Option<&GenerationRef>,
    ) -> Result<Vec<GlobalPreferenceRow>, PreferenceServiceRefusal> {
        self.commit_user_mutation(
            PreferenceMutation::Set {
                partition: PreferencePartition::User,
                key,
                value: value.clone(),
            },
            expected,
            Some(value),
            "SetGlobalPreference",
        )
    }

    pub fn reset_user(
        &mut self,
        key: PreferenceKey,
        expected: Option<&GenerationRef>,
    ) -> Result<Vec<GlobalPreferenceRow>, PreferenceServiceRefusal> {
        self.commit_user_mutation(
            PreferenceMutation::Remove {
                partition: PreferencePartition::User,
                key,
            },
            expected,
            None,
            "ResetGlobalPreference",
        )
    }

    fn commit_user_mutation(
        &mut self,
        mutation: PreferenceMutation,
        expected: Option<&GenerationRef>,
        draft: Option<Value>,
        reason: &str,
    ) -> Result<Vec<GlobalPreferenceRow>, PreferenceServiceRefusal> {
        if !self.status.writable() {
            return Err(self.refusal_for_status(draft));
        }
        let key = self.validate_product_mutation(&mutation, draft.clone())?;
        let metadata = self.metadata(reason);
        let actual_expected = match (&self.status, expected) {
            (PreferenceServiceStatus::DefaultsOnly, None) => {
                if matches!(mutation, PreferenceMutation::Remove { .. }) {
                    return Ok(self.rows());
                }
                self.repository
                    .initialize_with_mutations(&[mutation], &metadata)
                    .map_err(|error| self.map_error(error, draft.clone()))?;
                self.refresh();
                return Ok(self.rows());
            }
            (PreferenceServiceStatus::Ready { generation }, Some(expected))
                if generation == expected =>
            {
                expected.clone()
            }
            (PreferenceServiceStatus::Ready { .. }, _) => {
                self.refresh();
                return Err(PreferenceServiceRefusal {
                    kind: PreferenceServiceRefusalKind::StaleGeneration,
                    message:
                        "Preferences changed since this row was displayed; the draft was preserved."
                            .to_owned(),
                    preserved_draft: draft,
                    current_generation: self.status.generation().cloned().map(Box::new),
                });
            }
            _ => return Err(self.refusal_for_status(draft)),
        };
        let current_user = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.value(PreferencePartition::User, key));
        let is_noop = match &mutation {
            PreferenceMutation::Set { value, .. } => {
                current_user.is_some_and(|stored| stored.value == *value)
            }
            PreferenceMutation::Remove { .. } => current_user.is_none(),
            PreferenceMutation::PutUnknown(_) | PreferenceMutation::RemoveUnknown { .. } => false,
        };
        if is_noop {
            return Ok(self.rows());
        }
        match self
            .repository
            .commit_mutations(&actual_expected, &[mutation], &metadata)
        {
            Ok(_) => {
                self.refresh();
                Ok(self.rows())
            }
            Err(error) => {
                self.refresh();
                Err(self.map_error(error, draft))
            }
        }
    }

    fn validate_product_mutation<'a>(
        &self,
        mutation: &'a PreferenceMutation,
        draft: Option<Value>,
    ) -> Result<&'a PreferenceKey, PreferenceServiceRefusal> {
        let (key, value) = match mutation {
            PreferenceMutation::Set { key, value, .. } => (key, Some(value)),
            PreferenceMutation::Remove { key, .. } => (key, None),
            PreferenceMutation::PutUnknown(_) | PreferenceMutation::RemoveUnknown { .. } => {
                return Err(self.mutation_refusal(
                    PreferenceServiceRefusalKind::IneligibleSource,
                    "Product preference mutation cannot edit opaque envelopes".to_owned(),
                    draft,
                ));
            }
        };
        let descriptor = self.registry.get(key).ok_or_else(|| {
            self.mutation_refusal(
                PreferenceServiceRefusalKind::InvalidValue,
                format!("Unknown active preference {}", key.as_str()),
                draft.clone(),
            )
        })?;
        if value.is_some_and(|value| !descriptor.validates(value)) {
            return Err(self.mutation_refusal(
                PreferenceServiceRefusalKind::InvalidValue,
                format!("Invalid value for preference {}", key.as_str()),
                draft,
            ));
        }
        Ok(key)
    }

    fn mutation_refusal(
        &self,
        kind: PreferenceServiceRefusalKind,
        message: String,
        preserved_draft: Option<Value>,
    ) -> PreferenceServiceRefusal {
        PreferenceServiceRefusal {
            kind,
            message,
            preserved_draft,
            current_generation: self.status.generation().cloned().map(Box::new),
        }
    }
}
