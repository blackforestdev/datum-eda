#!/usr/bin/env python3
from __future__ import annotations
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

    for method in [create_draw_wire_proposal, create_place_label_proposal, create_place_symbol_proposal]:
        setattr(client_cls, method.__name__, method)


def locals_without_self(values: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in values.items() if key != "self"}
