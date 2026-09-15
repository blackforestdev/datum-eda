#!/usr/bin/env python3
from __future__ import annotations
from typing import Any


def install_schematic_drawing_methods(client_cls: type, append_optional: Any) -> None:
    def run(self, method: str, params: dict[str, Any], args: list[str]):
        return self._run_cli_json(self.build_request(method, params), args)
    def place_drawing_line(self, path: str, sheet: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int):
        return run(self, "place_drawing_line", locals(), ["project", "place-drawing-line", path, "--sheet", sheet, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm)])
    def place_drawing_rect(self, path: str, sheet: str, min_x_nm: int, min_y_nm: int, max_x_nm: int, max_y_nm: int):
        return run(self, "place_drawing_rect", locals(), ["project", "place-drawing-rect", path, "--sheet", sheet, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm)])
    def place_drawing_circle(self, path: str, sheet: str, center_x_nm: int, center_y_nm: int, radius_nm: int):
        return run(self, "place_drawing_circle", locals(), ["project", "place-drawing-circle", path, "--sheet", sheet, "--center-x-nm", str(center_x_nm), "--center-y-nm", str(center_y_nm), "--radius-nm", str(radius_nm)])
    def place_drawing_arc(self, path: str, sheet: str, center_x_nm: int, center_y_nm: int, radius_nm: int, start_angle_mdeg: int, end_angle_mdeg: int):
        return run(self, "place_drawing_arc", locals(), ["project", "place-drawing-arc", path, "--sheet", sheet, "--center-x-nm", str(center_x_nm), "--center-y-nm", str(center_y_nm), "--radius-nm", str(radius_nm), "--start-angle-mdeg", str(start_angle_mdeg), "--end-angle-mdeg", str(end_angle_mdeg)])
    def edit_drawing_line(self, path: str, drawing: str, from_x_nm: int | None = None, from_y_nm: int | None = None, to_x_nm: int | None = None, to_y_nm: int | None = None):
        args = ["project", "edit-drawing-line", path, "--drawing", drawing]; append_optional(args, "from-x-nm", from_x_nm); append_optional(args, "from-y-nm", from_y_nm); append_optional(args, "to-x-nm", to_x_nm); append_optional(args, "to-y-nm", to_y_nm); return run(self, "edit_drawing_line", locals(), args)
    def edit_drawing_rect(self, path: str, drawing: str, min_x_nm: int | None = None, min_y_nm: int | None = None, max_x_nm: int | None = None, max_y_nm: int | None = None):
        args = ["project", "edit-drawing-rect", path, "--drawing", drawing]; append_optional(args, "min-x-nm", min_x_nm); append_optional(args, "min-y-nm", min_y_nm); append_optional(args, "max-x-nm", max_x_nm); append_optional(args, "max-y-nm", max_y_nm); return run(self, "edit_drawing_rect", locals(), args)
    def edit_drawing_circle(self, path: str, drawing: str, center_x_nm: int | None = None, center_y_nm: int | None = None, radius_nm: int | None = None):
        args = ["project", "edit-drawing-circle", path, "--drawing", drawing]; append_optional(args, "center-x-nm", center_x_nm); append_optional(args, "center-y-nm", center_y_nm); append_optional(args, "radius-nm", radius_nm); return run(self, "edit_drawing_circle", locals(), args)
    def edit_drawing_arc(self, path: str, drawing: str, center_x_nm: int | None = None, center_y_nm: int | None = None, radius_nm: int | None = None, start_angle_mdeg: int | None = None, end_angle_mdeg: int | None = None):
        args = ["project", "edit-drawing-arc", path, "--drawing", drawing]; append_optional(args, "center-x-nm", center_x_nm); append_optional(args, "center-y-nm", center_y_nm); append_optional(args, "radius-nm", radius_nm); append_optional(args, "start-angle-mdeg", start_angle_mdeg); append_optional(args, "end-angle-mdeg", end_angle_mdeg); return run(self, "edit_drawing_arc", locals(), args)
    def delete_drawing(self, path: str, drawing: str):
        return run(self, "delete_drawing", {"path": path, "drawing": drawing}, ["project", "delete-drawing", path, "--drawing", drawing])
    for method in [place_drawing_line, place_drawing_rect, place_drawing_circle, place_drawing_arc, edit_drawing_line, edit_drawing_rect, edit_drawing_circle, edit_drawing_arc, delete_drawing]:
        setattr(client_cls, method.__name__, method)
