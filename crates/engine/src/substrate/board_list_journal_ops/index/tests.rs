use super::*;
use serde_json::json;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn payload(n: u128, label: &str) -> Value {
    json!({"uuid":id(n),"text":label})
}
#[test]
fn board_list_pass_preserves_payload_identity_duplicates_and_errors() {
    let mut board = json!({"texts":[{"uuid":"invalid"},payload(1,"first"),payload(1,"second"),payload(2,"last")],"dimensions":[],"keepouts":[]});
    let mut pass = BoardPass::new(&mut board);
    assert!(
        pass.apply(&Operation::CreateBoardText {
            text_id: id(1),
            text: payload(9, "ignored")
        })
        .unwrap()
    );
    assert_eq!(pass.value("texts", id(1)).unwrap()["text"], "first");
    pass.apply(&Operation::SetBoardText {
        text_id: id(1),
        text: payload(3, "renamed"),
    })
    .unwrap();
    assert_eq!(pass.value("texts", id(1)).unwrap()["text"], "second");
    pass.apply(&Operation::DeleteBoardText {
        text_id: id(3),
        text: Value::Null,
    })
    .unwrap();
    assert_eq!(pass.value("texts", id(1)).unwrap()["text"], "second");
    pass.apply(&Operation::CreateBoardText {
        text_id: id(8),
        text: payload(1, "third"),
    })
    .unwrap();
    assert_eq!(pass.value("texts", id(1)).unwrap()["text"], "second");
    pass.apply(&Operation::SetBoardText {
        text_id: id(1),
        text: json!({"uuid":"invalid","text":"unaddressable"}),
    })
    .unwrap();
    assert_eq!(pass.value("texts", id(1)).unwrap()["text"], "third");
    let mut inverse = Vec::new();
    pass.inverse(
        &Operation::CreateBoardText {
            text_id: id(1),
            text: payload(1, "duplicate"),
        },
        &mut inverse,
    )
    .unwrap();
    assert_eq!(
        inverse,
        vec![Operation::DeleteBoardText {
            text_id: id(1),
            text: payload(1, "duplicate")
        }]
    );
    assert_eq!(pass.board()["texts"].as_array().unwrap().len(), 4);
    assert!(
        matches!(pass.value("texts",id(99)),Err(EngineError::NotFound{object_type:"board_array_object",uuid}) if uuid==id(99))
    );
    let mut malformed = json!({"texts":{}});
    let mut pass = BoardPass::new(&mut malformed);
    for op in [
        Operation::CreateBoardText {
            text_id: id(1),
            text: payload(1, "a"),
        },
        Operation::SetBoardText {
            text_id: id(1),
            text: payload(1, "a"),
        },
        Operation::DeleteBoardText {
            text_id: id(1),
            text: Value::Null,
        },
    ] {
        assert!(
            matches!(pass.apply(&op),Err(EngineError::Validation(s)) if s=="board shard missing texts array")
        );
    }
}
#[test]
fn board_list_pass_indexes_full_dense_append_and_reverse_tail_without_rescans() {
    let mut board = json!({"texts":[],"dimensions":[],"keepouts":[]});
    let mut pass = BoardPass::new(&mut board);
    let mut inverse = Vec::new();
    for n in 1..=100000 {
        pass.inverse(
            &Operation::CreateBoardText {
                text_id: id(n),
                text: payload(n, "I"),
            },
            &mut inverse,
        )
        .unwrap();
    }
    assert_eq!(pass.board()["texts"].as_array().unwrap().len(), 100000);
    assert_eq!(pass.arrays["texts"].len(), 100000);
    for op in inverse.iter().rev() {
        pass.apply(op).unwrap();
    }
    assert!(pass.board()["texts"].as_array().unwrap().is_empty());
    assert!(
        pass.arrays.contains_key("texts"),
        "tail Undo retains the pass index"
    );
    assert!(pass.arrays["texts"].is_empty());
    for name in ["dimensions", "keepouts"] {
        pass.insert(name, id(1), payload(1, name)).unwrap();
        assert_eq!(pass.value(name, id(1)).unwrap()["text"], name);
        pass.remove(name, id(1)).unwrap();
    }
}
