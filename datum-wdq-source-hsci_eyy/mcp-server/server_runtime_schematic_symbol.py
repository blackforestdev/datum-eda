#!/usr/bin/env python3
from __future__ import annotations
from typing import Any


def install_schematic_symbol_methods(client_cls: type, append_optional: Any) -> None:
    def run(self, method: str, params: dict[str, Any], args: list[str]):
        return self._run_cli_json(self.build_request(method, params), args)
    def run_symbol(self, method: str, command: str, path: str, symbol: str, params: dict[str, Any], *tail: str):
        return run(self, method, {"path": path, "symbol": symbol, **params}, ["project", command, path, "--symbol", symbol, *tail])
    def run_field(self, method: str, command: str, path: str, field: str, params: dict[str, Any], *tail: str):
        return run(self, method, {"path": path, "field": field, **params}, ["project", command, path, "--field", field, *tail])
    def place_symbol(self, path: str, sheet: str, reference: str, value: str, x_nm: int, y_nm: int, lib_id: str | None = None, rotation_deg: int | None = None, mirrored: bool | None = None):
        args = ["project", "place-symbol", path, "--sheet", sheet, "--reference", reference, "--value", value, "--x-nm", str(x_nm), "--y-nm", str(y_nm)]; append_optional(args, "lib-id", lib_id); append_optional(args, "rotation-deg", rotation_deg); args.extend(["--mirrored"] if mirrored else []); return run(self, "place_symbol", {"path": path, "sheet": sheet, "reference": reference, "value": value, "x_nm": x_nm, "y_nm": y_nm, "lib_id": lib_id, "rotation_deg": rotation_deg, "mirrored": mirrored}, args)
    def move_symbol(self, path: str, symbol: str, x_nm: int, y_nm: int):
        return run(self, "move_symbol", {"path": path, "symbol": symbol, "x_nm": x_nm, "y_nm": y_nm}, ["project", "move-symbol", path, "--symbol", symbol, "--x-nm", str(x_nm), "--y-nm", str(y_nm)])
    def rotate_symbol(self, path: str, symbol: str, rotation_deg: int):
        return run(self, "rotate_symbol", {"path": path, "symbol": symbol, "rotation_deg": rotation_deg}, ["project", "rotate-symbol", path, "--symbol", symbol, "--rotation-deg", str(rotation_deg)])
    def mirror_symbol(self, path: str, symbol: str):
        return run(self, "mirror_symbol", {"path": path, "symbol": symbol}, ["project", "mirror-symbol", path, "--symbol", symbol])
    def delete_symbol(self, path: str, symbol: str):
        return run(self, "delete_symbol", {"path": path, "symbol": symbol}, ["project", "delete-symbol", path, "--symbol", symbol])
    def set_symbol_reference(self, path: str, symbol: str, reference: str):
        return run(self, "set_symbol_reference", {"path": path, "symbol": symbol, "reference": reference}, ["project", "set-symbol-reference", path, "--symbol", symbol, "--reference", reference])
    def set_symbol_value(self, path: str, symbol: str, value: str):
        return run(self, "set_symbol_value", {"path": path, "symbol": symbol, "value": value}, ["project", "set-symbol-value", path, "--symbol", symbol, "--value", value])
    def set_symbol_display_mode(self, path: str, symbol: str, mode: str):
        return run_symbol(self, "set_symbol_display_mode", "set-symbol-display-mode", path, symbol, {"mode": mode}, "--mode", mode)
    def set_symbol_hidden_power_behavior(self, path: str, symbol: str, behavior: str):
        return run_symbol(self, "set_symbol_hidden_power_behavior", "set-symbol-hidden-power-behavior", path, symbol, {"behavior": behavior}, "--behavior", behavior)
    def set_symbol_unit(self, path: str, symbol: str, unit: str):
        return run_symbol(self, "set_symbol_unit", "set-symbol-unit", path, symbol, {"unit": unit}, "--unit", unit)
    def clear_symbol_unit(self, path: str, symbol: str):
        return run_symbol(self, "clear_symbol_unit", "clear-symbol-unit", path, symbol, {})
    def set_symbol_gate(self, path: str, symbol: str, gate: str):
        return run_symbol(self, "set_symbol_gate", "set-symbol-gate", path, symbol, {"gate": gate}, "--gate", gate)
    def clear_symbol_gate(self, path: str, symbol: str):
        return run_symbol(self, "clear_symbol_gate", "clear-symbol-gate", path, symbol, {})
    def set_symbol_entity(self, path: str, symbol: str, entity: str):
        return run_symbol(self, "set_symbol_entity", "set-symbol-entity", path, symbol, {"entity": entity}, "--entity", entity)
    def clear_symbol_entity(self, path: str, symbol: str):
        return run_symbol(self, "clear_symbol_entity", "clear-symbol-entity", path, symbol, {})
    def set_symbol_part(self, path: str, symbol: str, part: str):
        return run_symbol(self, "set_symbol_part", "set-symbol-part", path, symbol, {"part": part}, "--part", part)
    def clear_symbol_part(self, path: str, symbol: str):
        return run_symbol(self, "clear_symbol_part", "clear-symbol-part", path, symbol, {})
    def set_symbol_lib_id(self, path: str, symbol: str, lib_id: str):
        return run_symbol(self, "set_symbol_lib_id", "set-symbol-lib-id", path, symbol, {"lib_id": lib_id}, "--lib-id", lib_id)
    def clear_symbol_lib_id(self, path: str, symbol: str):
        return run_symbol(self, "clear_symbol_lib_id", "clear-symbol-lib-id", path, symbol, {})
    def set_pin_override(self, path: str, symbol: str, pin: str, visible: bool, x_nm: int | None = None, y_nm: int | None = None):
        args = ["project", "set-pin-override", path, "--symbol", symbol, "--pin", pin, "--visible", str(visible).lower()]; append_optional(args, "x-nm", x_nm); append_optional(args, "y-nm", y_nm); return run(self, "set_pin_override", {"path": path, "symbol": symbol, "pin": pin, "visible": visible, "x_nm": x_nm, "y_nm": y_nm}, args)
    def clear_pin_override(self, path: str, symbol: str, pin: str):
        return run_symbol(self, "clear_pin_override", "clear-pin-override", path, symbol, {"pin": pin}, "--pin", pin)
    def add_symbol_field(self, path: str, symbol: str, key: str, value: str, hidden: bool | None = None, x_nm: int | None = None, y_nm: int | None = None):
        args = ["project", "add-symbol-field", path, "--symbol", symbol, "--key", key, "--value", value]; args.extend(["--hidden"] if hidden else []); append_optional(args, "x-nm", x_nm); append_optional(args, "y-nm", y_nm); return run(self, "add_symbol_field", {"path": path, "symbol": symbol, "key": key, "value": value, "hidden": hidden, "x_nm": x_nm, "y_nm": y_nm}, args)
    def edit_symbol_field(self, path: str, field: str, key: str | None = None, value: str | None = None, visible: bool | None = None, x_nm: int | None = None, y_nm: int | None = None):
        args = ["project", "edit-symbol-field", path, "--field", field]; append_optional(args, "key", key); append_optional(args, "value", value); append_optional(args, "visible", None if visible is None else str(visible).lower()); append_optional(args, "x-nm", x_nm); append_optional(args, "y-nm", y_nm); return run(self, "edit_symbol_field", {"path": path, "field": field, "key": key, "value": value, "visible": visible, "x_nm": x_nm, "y_nm": y_nm}, args)
    def delete_symbol_field(self, path: str, field: str):
        return run_field(self, "delete_symbol_field", "delete-symbol-field", path, field, {})
    for method in [place_symbol, move_symbol, rotate_symbol, mirror_symbol, delete_symbol, set_symbol_reference, set_symbol_value, set_symbol_display_mode, set_symbol_hidden_power_behavior, set_symbol_unit, clear_symbol_unit, set_symbol_gate, clear_symbol_gate, set_symbol_entity, clear_symbol_entity, set_symbol_part, clear_symbol_part, set_symbol_lib_id, clear_symbol_lib_id, set_pin_override, clear_pin_override, add_symbol_field, edit_symbol_field, delete_symbol_field]:
        setattr(client_cls, method.__name__, method)
