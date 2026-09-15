#!/usr/bin/env python3
from __future__ import annotations
import json
from typing import Any


def _component_instance_symbols(symbol: str | None, symbols: list[str] | None) -> list[str]:
    result = [str(value) for value in symbols] if symbols is not None else ([] if symbol is None else [symbol])
    if not result:
        raise ValueError("symbol or symbols is required")
    return result


def _component_role_spec(object_id: str, value: Any) -> str:
    role, label = (value.get("role"), value.get("label")) if isinstance(value, dict) else (value, None)
    return f"{object_id}={role}:{label}" if label is not None else f"{object_id}={role}"


def _append_component_role_args(args: list[str], flag: str, roles: Any | None) -> None:
    if isinstance(roles, dict):
        for object_id, value in roles.items():
            args.extend([f"--{flag}", _component_role_spec(str(object_id), value)])
        return
    for value in [] if roles is None else roles:
        args.extend([f"--{flag}", str(value)])


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

    def bind_component_instance_proposal(
        self,
        path: str,
        symbol: str | None,
        package: str,
        component_instance: str | None = None,
        symbols: list[str] | None = None,
        part: str | None = None,
        symbol_roles: Any | None = None,
        package_roles: Any | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "bind-component-instance", path]
        for value in _component_instance_symbols(symbol, symbols):
            args.extend(["--symbol", value])
        args.extend(["--package", package])
        append_optional(args, "component-instance", component_instance); append_optional(args, "part", part)
        _append_component_role_args(args, "symbol-role", symbol_roles); _append_component_role_args(args, "package-role", package_roles)
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "bind_component_instance_proposal", locals_without_self(locals()), args)

    def set_component_instance_proposal(
        self,
        path: str,
        component_instance: str,
        symbol: str | None,
        package: str,
        symbols: list[str] | None = None,
        part: str | None = None,
        symbol_roles: Any | None = None,
        package_roles: Any | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "set-component-instance", path, "--component-instance", component_instance]
        for value in _component_instance_symbols(symbol, symbols):
            args.extend(["--symbol", value])
        args.extend(["--package", package])
        append_optional(args, "part", part)
        _append_component_role_args(args, "symbol-role", symbol_roles); _append_component_role_args(args, "package-role", package_roles)
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "set_component_instance_proposal", locals_without_self(locals()), args)

    def delete_component_instance_proposal(
        self,
        path: str,
        component_instance: str,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "delete-component-instance", path, "--component-instance", component_instance]
        append_optional(args, "proposal", proposal); append_optional(args, "rationale", rationale)
        return run(self, "delete_component_instance_proposal", locals_without_self(locals()), args)

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
        pad: str | None = None,
        padstack: str | None = None,
        pad_name: str | None = None,
        x_nm: int | None = None,
        y_nm: int | None = None,
        layer: int | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-package", path, "--package", package, "--name", name]
        append_optional(args, "pad", pad); append_optional(args, "padstack", padstack)
        if pad is not None or padstack is not None:
            append_optional(args, "pad-name", pad_name); append_optional(args, "x-nm", x_nm)
            append_optional(args, "y-nm", y_nm); append_optional(args, "layer", layer)
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "create_pool_package_proposal", locals_without_self(locals()), args)

    def create_pool_footprint_proposal(
        self,
        path: str,
        footprint: str,
        package: str,
        name: str,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-footprint", path, "--footprint", footprint, "--package", package, "--name", name]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "create_pool_footprint_proposal", locals_without_self(locals()), args)

    def generate_ipc7351b_soic_proposal(
        self,
        path: str,
        footprint: str,
        package: str,
        padstack: str,
        pads: list[str],
        package_code: str,
        pin_count: int,
        pitch_nm: int,
        body_length_nm: int,
        body_width_nm: int,
        lead_span_nm: int,
        terminal_length_nm: int,
        terminal_width_nm: int,
        density: str | None = None,
        mask_expansion_nm: int | None = None,
        paste_reduction_nm: int | None = None,
        name: str | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "generate-ipc7351b-soic", path, "--footprint", footprint, "--package", package, "--padstack", padstack]
        for pad in pads:
            args.extend(["--pad", pad])
        args.extend([
            "--package-code", package_code,
            "--pin-count", str(pin_count),
            "--pitch-nm", str(pitch_nm),
            "--body-length-nm", str(body_length_nm),
            "--body-width-nm", str(body_width_nm),
            "--lead-span-nm", str(lead_span_nm),
            "--terminal-length-nm", str(terminal_length_nm),
            "--terminal-width-nm", str(terminal_width_nm),
        ])
        append_optional(args, "pool", pool); append_optional(args, "density", density)
        append_optional(args, "mask-expansion-nm", mask_expansion_nm)
        append_optional(args, "paste-reduction-nm", paste_reduction_nm)
        append_optional(args, "name", name); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "generate_ipc7351b_soic_proposal", locals_without_self(locals()), args)

    def create_pool_pin_pad_map_proposal(
        self,
        path: str,
        map: str,
        part: str,
        entries: list[str] | None = None,
        footprint: str | None = None,
        set_default: bool | None = None,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "create-pool-pin-pad-map", path, "--map", map, "--part", part]
        append_optional(args, "footprint", footprint)
        for entry in entries or []:
            args.extend(["--entry", entry])
        args.extend(["--set-default"] if set_default else [])
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "create_pool_pin_pad_map_proposal", locals_without_self(locals()), args)

    def set_pool_pin_pad_map_proposal(
        self,
        path: str,
        map: str,
        mode: str | None,
        entries: list[str],
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "set-pool-pin-pad-map", path, "--map", map]
        append_optional(args, "mode", mode)
        for entry in entries:
            args.extend(["--entry", entry])
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "set_pool_pin_pad_map_proposal", locals_without_self(locals()), args)

    def set_pool_footprint_pad_proposal(
        self,
        path: str,
        footprint: str,
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
        args = ["proposal", "set-pool-footprint-pad", path, "--footprint", footprint, "--pad", pad, "--padstack", padstack]
        append_optional(args, "pad-name", pad_name); append_optional(args, "x-nm", x_nm)
        append_optional(args, "y-nm", y_nm); append_optional(args, "layer", layer)
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "set_pool_footprint_pad_proposal", locals_without_self(locals()), args)

    def set_pool_footprint_courtyard_rect_proposal(
        self,
        path: str,
        footprint: str,
        min_x_nm: int,
        min_y_nm: int,
        max_x_nm: int,
        max_y_nm: int,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "set-pool-footprint-courtyard-rect", path, "--footprint", footprint, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm)]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "set_pool_footprint_courtyard_rect_proposal", locals_without_self(locals()), args)

    def set_pool_footprint_courtyard_polygon_proposal(
        self,
        path: str,
        footprint: str,
        vertices: str,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "set-pool-footprint-courtyard-polygon", path, "--footprint", footprint, "--vertices", vertices]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "set_pool_footprint_courtyard_polygon_proposal", locals_without_self(locals()), args)

    def add_pool_footprint_silkscreen_line_proposal(
        self,
        path: str,
        footprint: str,
        from_x_nm: int,
        from_y_nm: int,
        to_x_nm: int,
        to_y_nm: int,
        width_nm: int,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "add-pool-footprint-silkscreen-line", path, "--footprint", footprint, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm), "--width-nm", str(width_nm)]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "add_pool_footprint_silkscreen_line_proposal", locals_without_self(locals()), args)

    def add_pool_footprint_silkscreen_rect_proposal(
        self,
        path: str,
        footprint: str,
        min_x_nm: int,
        min_y_nm: int,
        max_x_nm: int,
        max_y_nm: int,
        width_nm: int,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "add-pool-footprint-silkscreen-rect", path, "--footprint", footprint, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm), "--width-nm", str(width_nm)]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "add_pool_footprint_silkscreen_rect_proposal", locals_without_self(locals()), args)

    def add_pool_footprint_silkscreen_circle_proposal(
        self,
        path: str,
        footprint: str,
        center_x_nm: int,
        center_y_nm: int,
        radius_nm: int,
        width_nm: int,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "add-pool-footprint-silkscreen-circle", path, "--footprint", footprint, "--center-x-nm", str(center_x_nm), "--center-y-nm", str(center_y_nm), "--radius-nm", str(radius_nm), "--width-nm", str(width_nm)]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "add_pool_footprint_silkscreen_circle_proposal", locals_without_self(locals()), args)

    def add_pool_footprint_silkscreen_polygon_proposal(
        self,
        path: str,
        footprint: str,
        vertices: str,
        closed: bool,
        width_nm: int,
        pool: str | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
    ):
        args = ["proposal", "add-pool-footprint-silkscreen-polygon", path, "--footprint", footprint, "--vertices", vertices, "--closed", str(closed).lower(), "--width-nm", str(width_nm)]
        append_optional(args, "pool", pool); append_optional(args, "proposal", proposal)
        append_optional(args, "rationale", rationale)
        return run(self, "add_pool_footprint_silkscreen_polygon_proposal", locals_without_self(locals()), args)

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

    for method in [create_draw_wire_proposal, create_place_label_proposal, create_place_symbol_proposal, create_board_component_replacement_proposal, create_board_component_replacements_proposal, create_board_component_replacement_plan_proposal, bind_component_instance_proposal, set_component_instance_proposal, delete_component_instance_proposal, create_pool_library_object_proposal, create_pool_unit_proposal, create_pool_symbol_proposal, create_pool_entity_proposal, create_pool_padstack_proposal, create_pool_package_proposal, create_pool_footprint_proposal, generate_ipc7351b_soic_proposal, create_pool_pin_pad_map_proposal, set_pool_pin_pad_map_proposal, set_pool_footprint_pad_proposal, set_pool_footprint_courtyard_rect_proposal, set_pool_footprint_courtyard_polygon_proposal, add_pool_footprint_silkscreen_line_proposal, add_pool_footprint_silkscreen_rect_proposal, add_pool_footprint_silkscreen_circle_proposal, add_pool_footprint_silkscreen_polygon_proposal, set_pool_package_pad_proposal, set_pool_package_courtyard_rect_proposal, set_pool_package_courtyard_polygon_proposal]:
        setattr(client_cls, method.__name__, method)


def locals_without_self(values: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in values.items() if key != "self"}
