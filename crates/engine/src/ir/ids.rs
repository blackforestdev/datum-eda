use uuid::Uuid;

/// Fixed namespace UUIDs for deterministic import identity.
/// See specs/IMPORT_SPEC.md §1.
/// Namespace for KiCad-imported objects.
pub fn namespace_kicad() -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"import.kicad.eda-tool")
}

/// Namespace for Eagle-imported objects.
pub fn namespace_eagle() -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"import.eagle.eda-tool")
}

/// Generate a deterministic UUID for an imported object.
/// Same (namespace, object_path) → same UUID, always.
pub fn import_uuid(namespace: &Uuid, object_path: &str) -> Uuid {
    Uuid::new_v5(namespace, object_path.as_bytes())
}

/// Generate a new random UUID for natively-created objects.
pub fn new_uuid() -> Uuid {
    Uuid::new_v4()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_import_uuid() {
        let ns = namespace_kicad();
        let a = import_uuid(&ns, "net:VCC");
        let b = import_uuid(&ns, "net:VCC");
        assert_eq!(a, b, "same input must produce same UUID");
    }

    #[test]
    fn different_paths_different_uuids() {
        let ns = namespace_kicad();
        let a = import_uuid(&ns, "net:VCC");
        let b = import_uuid(&ns, "net:GND");
        assert_ne!(a, b);
    }

    #[test]
    fn different_namespaces_different_uuids() {
        let a = import_uuid(&namespace_kicad(), "net:VCC");
        let b = import_uuid(&namespace_eagle(), "net:VCC");
        assert_ne!(a, b);
    }
}
