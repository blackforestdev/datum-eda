#!/usr/bin/env python3
from __future__ import annotations
import json
from typing import Any


def install_proposal_authoring_methods(client_cls: type, append_optional: Any) -> None:
    def run(self, method: str, params: dict[str, Any], args: list[str]):
        return self._run_cli_json(self.build_request(method, params), args)

    def create_draw_wire_proposal(
        self,
        path: str,
        sheet: str,
        from_x_nm: int,
        from_y_nm: int,
        to_x_nm: int,
        to_y_nm: int,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-draw-wire", path, "--sheet", sheet, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm)]
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_draw_wire_proposal", locals_without_self(locals()), args)

    def create_place_label_proposal(
        self,
        path: str,
        sheet: str,
        name: str,
        x_nm: int,
        y_nm: int,
        kind: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-place-label", path, "--sheet", sheet, "--name", name, "--x-nm", str(x_nm), "--y-nm", str(y_nm)]
        append_optional(args, "kind", kind); append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_place_label_proposal", locals_without_self(locals()), args)

    def create_place_symbol_proposal(
        self,
        path: str,
        sheet: str,
        reference: str,
        value: str,
        x_nm: int,
        y_nm: int,
        lib_id: str | None = None,
        rotation_deg: int | None = None,
        mirrored: bool | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-place-symbol", path, "--sheet", sheet, "--reference", reference, "--value", value, "--x-nm", str(x_nm), "--y-nm", str(y_nm)]
        append_optional(args, "lib-id", lib_id); append_optional(args, "rotation-deg", rotation_deg)
        args.extend(["--mirrored"] if mirrored else [])
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_place_symbol_proposal", locals_without_self(locals()), args)

    def create_board_component_replacement_proposal(
        self,
        path: str,
        component: str,
        package: str | None = None,
        part: str | None = None,
        value: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-board-component-replacement", path, "--component", component]
        append_optional(args, "package", package); append_optional(args, "part", part); append_optional(args, "value", value)
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_board_component_replacement_proposal", locals_without_self(locals()), args)

    def create_board_component_replacements_proposal(
        self,
        path: str,
        replacements: list[dict[str, Any]],
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-board-component-replacements", path]
        for replacement in replacements:
            args.extend(["--replacement", json.dumps(replacement, separators=(",", ":"))])
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_board_component_replacements_proposal", locals_without_self(locals()), args)

    def create_board_component_replacement_plan_proposal(
        self,
        path: str,
        selections: list[dict[str, Any]],
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-board-component-replacement-plan", path]
        for selection in selections:
            args.extend(["--selection", json.dumps(selection, separators=(",", ":"))])
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_board_component_replacement_plan_proposal", locals_without_self(locals()), args)

    def create_pool_library_object_proposal(
        self,
        path: str,
        kind: str,
        object: str,
        from_json: str,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-library-object", path, "--kind", kind, "--object", object, "--from-json", from_json]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_pool_library_object_proposal", locals_without_self(locals()), args)

    def create_pool_unit_proposal(
        self,
        path: str,
        unit: str,
        name: str,
        manufacturer: str | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-unit", path, "--unit", unit, "--name", name]
        append_optional(args, "manufacturer", manufacturer); append_optional(args, "pool", pool)
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_pool_unit_proposal", locals_without_self(locals()), args)

    def create_pool_symbol_proposal(
        self,
        path: str,
        symbol: str,
        unit: str,
        name: str,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-symbol", path, "--symbol", symbol, "--unit", unit, "--name", name]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_pool_symbol_proposal", locals_without_self(locals()), args)

    def create_pool_entity_proposal(
        self,
        path: str,
        entity: str,
        gate: str,
        unit: str,
        symbol: str,
        name: str,
        prefix: str,
        manufacturer: str | None = None,
        gate_name: str | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-entity", path, "--entity", entity, "--gate", gate, "--unit", unit, "--symbol", symbol, "--name", name, "--prefix", prefix]
        append_optional(args, "manufacturer", manufacturer); append_optional(args, "gate-name", gate_name); append_optional(args, "pool", pool)
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_pool_entity_proposal", locals_without_self(locals()), args)

    def create_pool_padstack_proposal(
        self,
        path: str,
        padstack: str,
        name: str,
        aperture: str | None = None,
        diameter_nm: int | None = None,
        width_nm: int | None = None,
        height_nm: int | None = None,
        drill_nm: int | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-padstack", path, "--padstack", padstack, "--name", name]
        append_optional(args, "aperture", aperture); append_optional(args, "diameter-nm", diameter_nm)
        append_optional(args, "width-nm", width_nm); append_optional(args, "height-nm", height_nm)
        append_optional(args, "drill-nm", drill_nm); append_optional(args, "pool", pool)
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "create_pool_padstack_proposal", locals_without_self(locals()), args)

    def create_pool_package_proposal(
        self,
        path: str,
        package: str,
        name: str,
        pad: str,
        padstack: str,
        pad_name: str | None = None,
        x_nm: int | None = None,
        y_nm: int | None = None,
        layer: int | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-package", path, "--package", package, "--name", name, "--pad", pad, "--padstack", padstack]
        append_optional(args, "pad-name", pad_name); append_optional(args, "x-nm", x_nm)
        append_optional(args, "y-nm", y_nm); append_optional(args, "layer", layer)
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "create_pool_package_proposal", locals_without_self(locals()), args)

    def set_pool_package_pad_proposal(
        self,
        path: str,
        package: str,
        pad: str,
        padstack: str,
        pad_name: str | None = None,
        x_nm: int | None = None,
        y_nm: int | None = None,
        layer: int | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "set-pool-package-pad", path, "--package", package, "--pad", pad, "--padstack", padstack]
        append_optional(args, "pad-name", pad_name); append_optional(args, "x-nm", x_nm)
        append_optional(args, "y-nm", y_nm); append_optional(args, "layer", layer)
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "set_pool_package_pad_proposal", locals_without_self(locals()), args)

    def set_pool_package_courtyard_rect_proposal(
        self,
        path: str,
        package: str,
        min_x_nm: int,
        min_y_nm: int,
        max_x_nm: int,
        max_y_nm: int,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "set-pool-package-courtyard-rect", path, "--package", package, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm)]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "set_pool_package_courtyard_rect_proposal", locals_without_self(locals()), args)

    def set_pool_package_courtyard_polygon_proposal(
        self,
        path: str,
        package: str,
        vertices: str,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "set-pool-package-courtyard-polygon", path, "--package", package, "--vertices", vertices]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "set_pool_package_courtyard_polygon_proposal", locals_without_self(locals()), args)

    for method in [create_draw_wire_proposal, create_place_label_proposal, create_place_symbol_proposal, create_board_component_replacement_proposal, create_board_component_replacements_proposal, create_board_component_replacement_plan_proposal, create_pool_library_object_proposal, create_pool_unit_proposal, create_pool_symbol_proposal, create_pool_entity_proposal, create_pool_padstack_proposal, create_pool_package_proposal, set_pool_package_pad_proposal, set_pool_package_courtyard_rect_proposal, set_pool_package_courtyard_polygon_proposal]:
        setattr(client_cls, method.__name__, method)


def locals_without_self(values: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in values.items() if key != "self"}
