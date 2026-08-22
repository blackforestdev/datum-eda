//! Non-focus-stealing accessibility semantics for Datum Console transitions.

use datum_gui_protocol::{ConsoleFeedbackCategory, ConsoleFeedbackDraft, ConsoleFeedbackSeverity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnnouncementPriority {
    Medium,
    High,
}

impl AnnouncementPriority {
    pub(crate) fn atspi_value(self) -> i32 {
        match self {
            Self::Medium => 1,
            Self::High => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccessibilityAnnouncement {
    pub(crate) text: String,
    pub(crate) priority: AnnouncementPriority,
}

pub(crate) fn announcement_for_draft(
    draft: &ConsoleFeedbackDraft,
    critical_consequence: bool,
) -> AccessibilityAnnouncement {
    let category = match draft.category {
        ConsoleFeedbackCategory::ActionEcho => "Action",
        ConsoleFeedbackCategory::ToolPrompt => "Tool prompt",
        ConsoleFeedbackCategory::ActionRefusal => "Action refused",
    };
    let severity = match draft.severity {
        ConsoleFeedbackSeverity::Informational => "Information",
        ConsoleFeedbackSeverity::Success => "Success",
        ConsoleFeedbackSeverity::Warning => "Warning",
        ConsoleFeedbackSeverity::Error => "Error",
    };
    AccessibilityAnnouncement {
        text: format!("{category}. {severity}: {}", draft.message),
        priority: if critical_consequence {
            AnnouncementPriority::High
        } else {
            AnnouncementPriority::Medium
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::ConsoleFeedbackSource;

    #[test]
    fn ordinary_refusal_is_polite_and_severity_is_redundant_in_text() {
        let draft = ConsoleFeedbackDraft::action_refusal(
            ConsoleFeedbackSource::Tool,
            42,
            "no component selected",
        );
        let announcement = announcement_for_draft(&draft, false);
        assert_eq!(announcement.priority, AnnouncementPriority::Medium);
        assert_eq!(
            announcement.text,
            "Action refused. Error: no component selected"
        );
    }

    #[test]
    fn high_priority_requires_an_explicit_critical_consequence() {
        let draft = ConsoleFeedbackDraft::action_refusal(
            ConsoleFeedbackSource::Production,
            42,
            "critical consequence",
        );
        assert_eq!(
            announcement_for_draft(&draft, true).priority,
            AnnouncementPriority::High
        );
        assert_eq!(
            announcement_for_draft(&draft, false).priority,
            AnnouncementPriority::Medium
        );
    }
}
