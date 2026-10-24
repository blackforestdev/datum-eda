//! Occurrence-first read-only navigation in the existing shared Project card.
use super::*;

pub(super) fn render(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let documents = &state.schematic_documents;
    let (Some(project), Some(revision)) = (documents.project, documents.revision.as_ref()) else {
        return;
    };
    let x = rect.x + UI_CARD_PADDING_X;
    let top = rect.y + UI_CARD_CONTENT_TOP;
    draw_text(
        &state.scene.project_name,
        x,
        top,
        design_tokens::typography::BODY_SIZE,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        "Schematic documents",
        x,
        top + 24.0,
        design_tokens::typography::CAPTION_SIZE,
        TEXT_SECONDARY,
        TextFace::Ui,
        text,
    );
    let body = RectPx {
        x: rect.x,
        y: top + 44.0,
        width: rect.width,
        height: (rect.height - UI_CARD_CONTENT_TOP - 44.0).max(0.0),
    };
    let rows = documents.visible_rows();
    let row_height = design_tokens::spacing::SP_06 * 2.0;
    let visible = (body.height / row_height).floor() as usize;
    let start = documents
        .scroll_offset
        .min(rows.len().saturating_sub(visible));
    hits.push(HitRegion {
        rect: body,
        target: HitTarget::SchematicNavigatorScroll {
            project,
            revision: revision.0.clone(),
            total: rows.len(),
            visible,
        },
    });
    for (index, row) in rows.iter().skip(start).take(visible).enumerate() {
        let row_rect = RectPx {
            y: body.y + index as f32 * row_height,
            height: row_height,
            ..body
        };
        let selected = documents.inspected.as_ref() == Some(&row.key);
        if selected {
            quads.push(Quad::from_rect(row_rect, REVIEW_ROW_ACTIVE_BG));
        }
        let indent = row.key.instance_path.len() as f32 * design_tokens::spacing::SP_04;
        let label_x = x + indent + 16.0;
        let has_children = documents
            .rows
            .iter()
            .any(|child| child.parent.as_ref() == Some(&row.key));
        if has_children {
            draw_text(
                if documents.expanded.contains(&row.key) {
                    "−"
                } else {
                    "+"
                },
                x + indent,
                row_rect.y + 2.0,
                12.0,
                TEXT_SECONDARY,
                TextFace::Ui,
                text,
            );
            hits.push(HitRegion {
                rect: RectPx {
                    x: x + indent - 2.0,
                    width: 16.0,
                    ..row_rect
                },
                target: HitTarget::SchematicDocumentExpand {
                    document: row.key.clone(),
                    revision: revision.0.clone(),
                },
            });
        }
        draw_text(
            &format!("{}{}", if selected { "> " } else { "" }, row.name),
            label_x,
            row_rect.y + 2.0,
            design_tokens::typography::BODY_SIZE,
            TEXT_PRIMARY,
            TextFace::Ui,
            text,
        );
        draw_text(
            &format!("Source · {}", row.source_name),
            label_x,
            row_rect.y + 17.0,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_SECONDARY,
            TextFace::Ui,
            text,
        );
        hits.push(HitRegion {
            rect: RectPx {
                x: label_x,
                width: (row_rect.x + row_rect.width - label_x).max(0.0),
                ..row_rect
            },
            target: HitTarget::SchematicDocumentRow {
                document: row.key.clone(),
                revision: revision.0.clone(),
            },
        });
    }
    if rows.is_empty() {
        draw_text(
            "No schematic document source",
            x,
            body.y,
            11.0,
            TEXT_SECONDARY,
            TextFace::Ui,
            text,
        );
    }
}
