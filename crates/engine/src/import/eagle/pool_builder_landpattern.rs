use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::ir::geometry::Polygon;
use crate::pool::{Footprint, Package, Pad};

pub(super) fn eagle_body_package(package_uuid: Uuid, name: String) -> Package {
    Package {
        uuid: package_uuid,
        name,
        package_family: None,
        package_code: None,
        mounting_type: None,
        body_dimensions: None,
        terminals: HashMap::new(),
        pads: HashMap::new(),
        courtyard: Polygon {
            vertices: Vec::new(),
            closed: true,
        },
        silkscreen: Vec::new(),
        models_3d: Vec::new(),
        body_height_nm: None,
        body_height_mounted_nm: None,
        tags: HashSet::new(),
    }
}

pub(super) fn eagle_footprint(
    footprint_uuid: Uuid,
    package_uuid: Uuid,
    name: String,
    pads: HashMap<Uuid, Pad>,
    silkscreen: Vec<crate::pool::Primitive>,
) -> Footprint {
    Footprint {
        uuid: footprint_uuid,
        name,
        package: package_uuid,
        pads,
        courtyard: Polygon {
            vertices: Vec::new(),
            closed: true,
        },
        silkscreen,
        fab: Vec::new(),
        assembly: Vec::new(),
        mechanical: Vec::new(),
        models_3d: Vec::new(),
        standards_basis: None,
        process_aperture_policy: Some("import_preserved".to_string()),
        tags: HashSet::new(),
    }
}
