#!/usr/bin/env python3
from __future__ import annotations


def install_library_methods(client_cls: type) -> None:
    def get_pool_model_blobs(
        self,
        path: str,
        pool: str | None = None,
        role: str | None = None,
        sha256: str | None = None,
    ):
        args = ["project", "query", path, "pool-models"]
        for key, value in {"pool": pool, "role": role, "sha256": sha256}.items():
            if value is not None:
                args.extend([f"--{key}", value])
        return self._run_cli_json(
            self.build_request("get_pool_model_blobs", {"path": path, "pool": pool, "role": role, "sha256": sha256}),
            args,
        )

    def gc_pool_model_blobs(
        self,
        path: str,
        pool: str | None = None,
        role: str | None = None,
        sha256: str | None = None,
        apply: bool | None = None,
    ):
        args = ["project", "gc-pool-models", path]
        for key, value in {"pool": pool, "role": role, "sha256": sha256}.items():
            if value is not None:
                args.extend([f"--{key}", value])
        if apply:
            args.append("--apply")
        return self._run_cli_json(
            self.build_request("gc_pool_model_blobs", {"path": path, "pool": pool, "role": role, "sha256": sha256, "apply": apply}),
            args,
        )

    def set_pool_unit_pin(self, path: str, pool: str, unit: str, pin: str, name: str, direction: str, swap_group: int):
        return self._run_cli_json(
            self.build_request("set_pool_unit_pin", {"path": path, "pool": pool, "unit": unit, "pin": pin, "name": name, "direction": direction, "swap_group": swap_group}),
            ["project", "set-pool-unit-pin", path, "--pool", pool, "--unit", unit, "--pin", pin, "--name", name, "--direction", direction, "--swap-group", str(swap_group)],
        )

    def set_pool_package_pad(self, path: str, pool: str, package: str, pad: str, padstack: str, pad_name: str, x_nm: int, y_nm: int, layer: int):
        return self._run_cli_json(
            self.build_request("set_pool_package_pad", {"path": path, "pool": pool, "package": package, "pad": pad, "padstack": padstack, "pad_name": pad_name, "x_nm": x_nm, "y_nm": y_nm, "layer": layer}),
            ["project", "set-pool-package-pad", path, "--pool", pool, "--package", package, "--pad", pad, "--padstack", padstack, "--pad-name", pad_name, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--layer", str(layer)],
        )

    def create_pool_footprint(self, path: str, pool: str, footprint: str, package: str, name: str):
        return self._run_cli_json(
            self.build_request("create_pool_footprint", {"path": path, "pool": pool, "footprint": footprint, "package": package, "name": name}),
            ["project", "create-pool-footprint", path, "--pool", pool, "--footprint", footprint, "--package", package, "--name", name],
        )

    def generate_ipc7351b_soic(
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
        pool: str = "pool",
        density: str = "nominal",
        mask_expansion_nm: int = 50000,
        paste_reduction_nm: int = 50000,
        name: str | None = None,
    ):
        args = [
            "project",
            "generate-ipc7351b-soic",
            path,
            "--pool",
            pool,
            "--footprint",
            footprint,
            "--package",
            package,
            "--padstack",
            padstack,
        ]
        for pad in pads:
            args.extend(["--pad", pad])
        args.extend([
            "--package-code",
            package_code,
            "--pin-count",
            str(pin_count),
            "--pitch-nm",
            str(pitch_nm),
            "--body-length-nm",
            str(body_length_nm),
            "--body-width-nm",
            str(body_width_nm),
            "--lead-span-nm",
            str(lead_span_nm),
            "--terminal-length-nm",
            str(terminal_length_nm),
            "--terminal-width-nm",
            str(terminal_width_nm),
            "--density",
            density,
            "--mask-expansion-nm",
            str(mask_expansion_nm),
            "--paste-reduction-nm",
            str(paste_reduction_nm),
        ])
        if name is not None:
            args.extend(["--name", name])
        return self._run_cli_json(
            self.build_request(
                "generate_ipc7351b_soic",
                {
                    "path": path,
                    "pool": pool,
                    "footprint": footprint,
                    "package": package,
                    "padstack": padstack,
                    "pads": pads,
                    "package_code": package_code,
                    "pin_count": pin_count,
                    "pitch_nm": pitch_nm,
                    "body_length_nm": body_length_nm,
                    "body_width_nm": body_width_nm,
                    "lead_span_nm": lead_span_nm,
                    "terminal_length_nm": terminal_length_nm,
                    "terminal_width_nm": terminal_width_nm,
                    "density": density,
                    "mask_expansion_nm": mask_expansion_nm,
                    "paste_reduction_nm": paste_reduction_nm,
                    "name": name,
                },
            ),
            args,
        )

    def set_pool_footprint_pad(self, path: str, pool: str, footprint: str, pad: str, padstack: str, pad_name: str, x_nm: int, y_nm: int, layer: int):
        return self._run_cli_json(
            self.build_request("set_pool_footprint_pad", {"path": path, "pool": pool, "footprint": footprint, "pad": pad, "padstack": padstack, "pad_name": pad_name, "x_nm": x_nm, "y_nm": y_nm, "layer": layer}),
            ["project", "set-pool-footprint-pad", path, "--pool", pool, "--footprint", footprint, "--pad", pad, "--padstack", padstack, "--pad-name", pad_name, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--layer", str(layer)],
        )

    def set_pool_footprint_courtyard_rect(self, path: str, pool: str, footprint: str, min_x_nm: int, min_y_nm: int, max_x_nm: int, max_y_nm: int):
        return self._run_cli_json(
            self.build_request("set_pool_footprint_courtyard_rect", {"path": path, "pool": pool, "footprint": footprint, "min_x_nm": min_x_nm, "min_y_nm": min_y_nm, "max_x_nm": max_x_nm, "max_y_nm": max_y_nm}),
            ["project", "set-pool-footprint-courtyard-rect", path, "--pool", pool, "--footprint", footprint, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm)],
        )

    def set_pool_footprint_courtyard_polygon(self, path: str, pool: str, footprint: str, vertices: str):
        return self._run_cli_json(
            self.build_request("set_pool_footprint_courtyard_polygon", {"path": path, "pool": pool, "footprint": footprint, "vertices": vertices}),
            ["project", "set-pool-footprint-courtyard-polygon", path, "--pool", pool, "--footprint", footprint, "--vertices", vertices],
        )

    def add_pool_footprint_silkscreen_line(self, path: str, pool: str, footprint: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_footprint_silkscreen_line", {"path": path, "pool": pool, "footprint": footprint, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm, "width_nm": width_nm}),
            ["project", "add-pool-footprint-silkscreen-line", path, "--pool", pool, "--footprint", footprint, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_footprint_silkscreen_rect(self, path: str, pool: str, footprint: str, min_x_nm: int, min_y_nm: int, max_x_nm: int, max_y_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_footprint_silkscreen_rect", {"path": path, "pool": pool, "footprint": footprint, "min_x_nm": min_x_nm, "min_y_nm": min_y_nm, "max_x_nm": max_x_nm, "max_y_nm": max_y_nm, "width_nm": width_nm}),
            ["project", "add-pool-footprint-silkscreen-rect", path, "--pool", pool, "--footprint", footprint, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_footprint_silkscreen_circle(self, path: str, pool: str, footprint: str, center_x_nm: int, center_y_nm: int, radius_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_footprint_silkscreen_circle", {"path": path, "pool": pool, "footprint": footprint, "center_x_nm": center_x_nm, "center_y_nm": center_y_nm, "radius_nm": radius_nm, "width_nm": width_nm}),
            ["project", "add-pool-footprint-silkscreen-circle", path, "--pool", pool, "--footprint", footprint, "--center-x-nm", str(center_x_nm), "--center-y-nm", str(center_y_nm), "--radius-nm", str(radius_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_footprint_silkscreen_polygon(self, path: str, pool: str, footprint: str, vertices: str, closed: bool, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_footprint_silkscreen_polygon", {"path": path, "pool": pool, "footprint": footprint, "vertices": vertices, "closed": closed, "width_nm": width_nm}),
            ["project", "add-pool-footprint-silkscreen-polygon", path, "--pool", pool, "--footprint", footprint, "--vertices", vertices, "--closed", str(closed).lower(), "--width-nm", str(width_nm)],
        )

    def set_pool_package_courtyard_rect(self, path: str, pool: str, package: str, min_x_nm: int, min_y_nm: int, max_x_nm: int, max_y_nm: int):
        return self._run_cli_json(
            self.build_request("set_pool_package_courtyard_rect", {"path": path, "pool": pool, "package": package, "min_x_nm": min_x_nm, "min_y_nm": min_y_nm, "max_x_nm": max_x_nm, "max_y_nm": max_y_nm}),
            ["project", "set-pool-package-courtyard-rect", path, "--pool", pool, "--package", package, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm)],
        )

    def set_pool_package_courtyard_polygon(self, path: str, pool: str, package: str, vertices: str):
        return self._run_cli_json(
            self.build_request("set_pool_package_courtyard_polygon", {"path": path, "pool": pool, "package": package, "vertices": vertices}),
            ["project", "set-pool-package-courtyard-polygon", path, "--pool", pool, "--package", package, "--vertices", vertices],
        )

    def add_pool_package_silkscreen_line(self, path: str, pool: str, package: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_package_silkscreen_line", {"path": path, "pool": pool, "package": package, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm, "width_nm": width_nm}),
            ["project", "add-pool-package-silkscreen-line", path, "--pool", pool, "--package", package, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_symbol_line(self, path: str, pool: str, symbol: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_symbol_line", {"path": path, "pool": pool, "symbol": symbol, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm, "width_nm": width_nm}),
            ["project", "add-pool-symbol-line", path, "--pool", pool, "--symbol", symbol, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_symbol_rect(self, path: str, pool: str, symbol: str, min_x_nm: int, min_y_nm: int, max_x_nm: int, max_y_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_symbol_rect", {"path": path, "pool": pool, "symbol": symbol, "min_x_nm": min_x_nm, "min_y_nm": min_y_nm, "max_x_nm": max_x_nm, "max_y_nm": max_y_nm, "width_nm": width_nm}),
            ["project", "add-pool-symbol-rect", path, "--pool", pool, "--symbol", symbol, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_symbol_circle(self, path: str, pool: str, symbol: str, center_x_nm: int, center_y_nm: int, radius_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_symbol_circle", {"path": path, "pool": pool, "symbol": symbol, "center_x_nm": center_x_nm, "center_y_nm": center_y_nm, "radius_nm": radius_nm, "width_nm": width_nm}),
            ["project", "add-pool-symbol-circle", path, "--pool", pool, "--symbol", symbol, "--center-x-nm", str(center_x_nm), "--center-y-nm", str(center_y_nm), "--radius-nm", str(radius_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_symbol_arc(self, path: str, pool: str, symbol: str, x_nm: int, y_nm: int, radius_nm: int, start_angle: int, end_angle: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_symbol_arc", {"path": path, "pool": pool, "symbol": symbol, "x_nm": x_nm, "y_nm": y_nm, "radius_nm": radius_nm, "start_angle": start_angle, "end_angle": end_angle, "width_nm": width_nm}),
            ["project", "add-pool-symbol-arc", path, "--pool", pool, "--symbol", symbol, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--radius-nm", str(radius_nm), "--start-angle", str(start_angle), "--end-angle", str(end_angle), "--width-nm", str(width_nm)],
        )

    def add_pool_symbol_polygon(self, path: str, pool: str, symbol: str, vertices: str, closed: bool, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_symbol_polygon", {"path": path, "pool": pool, "symbol": symbol, "vertices": vertices, "closed": closed, "width_nm": width_nm}),
            ["project", "add-pool-symbol-polygon", path, "--pool", pool, "--symbol", symbol, "--vertices", vertices, "--closed", str(closed).lower(), "--width-nm", str(width_nm)],
        )

    def add_pool_symbol_text(self, path: str, pool: str, symbol: str, text: str, x_nm: int, y_nm: int, rotation: int):
        return self._run_cli_json(
            self.build_request("add_pool_symbol_text", {"path": path, "pool": pool, "symbol": symbol, "text": text, "x_nm": x_nm, "y_nm": y_nm, "rotation": rotation}),
            ["project", "add-pool-symbol-text", path, "--pool", pool, "--symbol", symbol, "--text", text, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--rotation", str(rotation)],
        )

    def set_pool_symbol_pin_anchor(self, path: str, pool: str, symbol: str, pin: str, x_nm: int, y_nm: int):
        return self._run_cli_json(
            self.build_request("set_pool_symbol_pin_anchor", {"path": path, "pool": pool, "symbol": symbol, "pin": pin, "x_nm": x_nm, "y_nm": y_nm}),
            ["project", "set-pool-symbol-pin-anchor", path, "--pool", pool, "--symbol", symbol, "--pin", pin, "--x-nm", str(x_nm), "--y-nm", str(y_nm)],
        )

    def add_pool_package_silkscreen_rect(self, path: str, pool: str, package: str, min_x_nm: int, min_y_nm: int, max_x_nm: int, max_y_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_package_silkscreen_rect", {"path": path, "pool": pool, "package": package, "min_x_nm": min_x_nm, "min_y_nm": min_y_nm, "max_x_nm": max_x_nm, "max_y_nm": max_y_nm, "width_nm": width_nm}),
            ["project", "add-pool-package-silkscreen-rect", path, "--pool", pool, "--package", package, "--min-x-nm", str(min_x_nm), "--min-y-nm", str(min_y_nm), "--max-x-nm", str(max_x_nm), "--max-y-nm", str(max_y_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_package_silkscreen_polygon(self, path: str, pool: str, package: str, vertices: str, closed: bool, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_package_silkscreen_polygon", {"path": path, "pool": pool, "package": package, "vertices": vertices, "closed": closed, "width_nm": width_nm}),
            ["project", "add-pool-package-silkscreen-polygon", path, "--pool", pool, "--package", package, "--vertices", vertices, "--closed", str(closed).lower(), "--width-nm", str(width_nm)],
        )

    def add_pool_package_silkscreen_circle(self, path: str, pool: str, package: str, center_x_nm: int, center_y_nm: int, radius_nm: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_package_silkscreen_circle", {"path": path, "pool": pool, "package": package, "center_x_nm": center_x_nm, "center_y_nm": center_y_nm, "radius_nm": radius_nm, "width_nm": width_nm}),
            ["project", "add-pool-package-silkscreen-circle", path, "--pool", pool, "--package", package, "--center-x-nm", str(center_x_nm), "--center-y-nm", str(center_y_nm), "--radius-nm", str(radius_nm), "--width-nm", str(width_nm)],
        )

    def add_pool_package_silkscreen_arc(self, path: str, pool: str, package: str, x_nm: int, y_nm: int, radius_nm: int, start_angle: int, end_angle: int, width_nm: int):
        return self._run_cli_json(
            self.build_request("add_pool_package_silkscreen_arc", {"path": path, "pool": pool, "package": package, "x_nm": x_nm, "y_nm": y_nm, "radius_nm": radius_nm, "start_angle": start_angle, "end_angle": end_angle, "width_nm": width_nm}),
            ["project", "add-pool-package-silkscreen-arc", path, "--pool", pool, "--package", package, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--radius-nm", str(radius_nm), "--start-angle", str(start_angle), "--end-angle", str(end_angle), "--width-nm", str(width_nm)],
        )

    def add_pool_package_silkscreen_text(self, path: str, pool: str, package: str, text: str, x_nm: int, y_nm: int, rotation: float):
        return self._run_cli_json(
            self.build_request("add_pool_package_silkscreen_text", {"path": path, "pool": pool, "package": package, "text": text, "x_nm": x_nm, "y_nm": y_nm, "rotation": rotation}),
            ["project", "add-pool-package-silkscreen-text", path, "--pool", pool, "--package", package, "--text", text, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--rotation", str(rotation)],
        )

    def add_pool_package_model_3d(self, path: str, pool: str, package: str, model_path: str, transform_json: str | None = None, format: str | None = None, tx_nm: int | None = None, ty_nm: int | None = None, tz_nm: int | None = None, roll_tenths_deg: int | None = None, pitch_tenths_deg: int | None = None, yaw_tenths_deg: int | None = None, scale: object | None = None):
        options = {"format": format, "tx_nm": tx_nm, "ty_nm": ty_nm, "tz_nm": tz_nm, "roll_tenths_deg": roll_tenths_deg, "pitch_tenths_deg": pitch_tenths_deg, "yaw_tenths_deg": yaw_tenths_deg, "scale": scale}
        args = ["project", "add-pool-package-model-3d", path, "--pool", pool, "--package", package, "--model-path", model_path]
        if transform_json is not None:
            args.extend(["--transform-json", transform_json])
        for key, value in options.items():
            if value is not None:
                args.extend([f"--{key.replace('_', '-')}", str(value)])
        return self._run_cli_json(
            self.build_request("add_pool_package_model_3d", {"path": path, "pool": pool, "package": package, "model_path": model_path, "transform_json": transform_json, **options}),
            args,
        )

    def set_pool_package_body_heights(self, path: str, pool: str, package: str, body_height_nm: int | None = None, body_height_mounted_nm: int | None = None, clear: bool | None = None):
        args = ["project", "set-pool-package-body-heights", path, "--pool", pool, "--package", package]
        if body_height_nm is not None:
            args.extend(["--body-height-nm", str(body_height_nm)])
        if body_height_mounted_nm is not None:
            args.extend(["--body-height-mounted-nm", str(body_height_mounted_nm)])
        if clear:
            args.append("--clear")
        return self._run_cli_json(
            self.build_request("set_pool_package_body_heights", {"path": path, "pool": pool, "package": package, "body_height_nm": body_height_nm, "body_height_mounted_nm": body_height_mounted_nm, "clear": clear}),
            args,
        )

    def set_pool_part_metadata(self, path: str, pool: str, part: str, mpn: str | None = None, manufacturer: str | None = None, manufacturer_jep106: int | None = None, value: str | None = None, description: str | None = None, datasheet: str | None = None, lifecycle: str | None = None):
        metadata = {"mpn": mpn, "manufacturer": manufacturer, "manufacturer_jep106": manufacturer_jep106, "value": value, "description": description, "datasheet": datasheet, "lifecycle": lifecycle}
        args = ["project", "set-pool-part-metadata", path, "--pool", pool, "--part", part]
        for key, value_ in metadata.items():
            if value_ is not None:
                args.extend([f"--{key.replace('_', '-')}", str(value_)])
        return self._run_cli_json(
            self.build_request("set_pool_part_metadata", {"path": path, "pool": pool, "part": part, **metadata}),
            args,
        )

    def set_pool_part_parametric(self, path: str, pool: str, part: str, mode: str, params: dict[str, str]):
        args = ["project", "set-pool-part-parametric", path, "--pool", pool, "--part", part, "--mode", mode]
        for key, value in params.items():
            args.extend(["--param", f"{key}={value}"])
        return self._run_cli_json(
            self.build_request("set_pool_part_parametric", {"path": path, "pool": pool, "part": part, "mode": mode, "params": params}),
            args,
        )

    def set_pool_part_orderable_mpns(self, path: str, pool: str, part: str, mode: str, mpns: list[str]):
        args = ["project", "set-pool-part-orderable-mpns", path, "--pool", pool, "--part", part, "--mode", mode]
        for mpn in mpns:
            args.extend(["--mpn", mpn])
        return self._run_cli_json(
            self.build_request("set_pool_part_orderable_mpns", {"path": path, "pool": pool, "part": part, "mode": mode, "mpns": mpns}),
            args,
        )

    def set_pool_part_tags(self, path: str, pool: str, part: str, mode: str, tags: list[str]):
        args = ["project", "set-pool-part-tags", path, "--pool", pool, "--part", part, "--mode", mode]
        for tag in tags:
            args.extend(["--tag", tag])
        return self._run_cli_json(
            self.build_request("set_pool_part_tags", {"path": path, "pool": pool, "part": part, "mode": mode, "tags": tags}),
            args,
        )

    def set_pool_part_packaging_options(self, path: str, pool: str, part: str, mode: str, options: list[str]):
        args = ["project", "set-pool-part-packaging-options", path, "--pool", pool, "--part", part, "--mode", mode]
        for option in options:
            args.extend(["--option", option])
        return self._run_cli_json(
            self.build_request("set_pool_part_packaging_options", {"path": path, "pool": pool, "part": part, "mode": mode, "options": options}),
            args,
        )

    def set_pool_part_supply_chain(self, path: str, pool: str, part: str, clear: bool = False, checked_at: str | None = None, offers: list[str] | None = None):
        offer_payloads = offers or []
        args = ["project", "set-pool-part-supply-chain", path, "--pool", pool, "--part", part]
        if clear:
            args.append("--clear")
        if checked_at is not None:
            args.extend(["--checked-at", checked_at])
        for offer in offer_payloads:
            args.extend(["--offer", offer])
        return self._run_cli_json(
            self.build_request("set_pool_part_supply_chain", {"path": path, "pool": pool, "part": part, "clear": clear, "checked_at": checked_at, "offers": offer_payloads}),
            args,
        )

    def set_pool_part_behavioural_models(self, path: str, pool: str, part: str, mode: str, models: list[str]):
        args = ["project", "set-pool-part-behavioural-models", path, "--pool", pool, "--part", part, "--mode", mode]
        for model in models:
            args.extend(["--model", model])
        return self._run_cli_json(
            self.build_request("set_pool_part_behavioural_models", {"path": path, "pool": pool, "part": part, "mode": mode, "models": models}),
            args,
        )

    def attach_pool_part_model(self, path: str, pool: str, part: str, source: str, role: str, dialect: str | None = None, model_names: list[str] | None = None, encrypted: bool = False, encryption_scheme: str | None = None, vendor: str | None = None, fetched_at: str | None = None, format_metadata_json: str | None = None):
        names = model_names or []
        args = ["project", "attach-pool-part-model", path, "--pool", pool, "--part", part, "--source", source, "--role", role]
        if dialect is not None:
            args.extend(["--dialect", dialect])
        for model_name in names:
            args.extend(["--model-name", model_name])
        if encrypted:
            args.append("--encrypted")
        if encryption_scheme is not None:
            args.extend(["--encryption-scheme", encryption_scheme])
        if vendor is not None:
            args.extend(["--vendor", vendor])
        if fetched_at is not None:
            args.extend(["--fetched-at", fetched_at])
        if format_metadata_json is not None:
            args.extend(["--format-metadata-json", format_metadata_json])
        return self._run_cli_json(
            self.build_request("attach_pool_part_model", {"path": path, "pool": pool, "part": part, "source": source, "role": role, "dialect": dialect, "model_names": names, "encrypted": encrypted, "encryption_scheme": encryption_scheme, "vendor": vendor, "fetched_at": fetched_at, "format_metadata_json": format_metadata_json}),
            args,
        )

    def detach_pool_part_model(self, path: str, pool: str, part: str, attachment: str | None = None, model: str | None = None):
        args = ["project", "detach-pool-part-model", path, "--pool", pool, "--part", part]
        if attachment is not None:
            args.extend(["--attachment", attachment])
        if model is not None:
            args.extend(["--model", model])
        return self._run_cli_json(
            self.build_request("detach_pool_part_model", {"path": path, "pool": pool, "part": part, "attachment": attachment, "model": model}),
            args,
        )

    def set_pool_part_thermal(
        self,
        path: str,
        pool: str,
        part: str,
        theta_ja_c_per_w=None,
        theta_jc_top_c_per_w=None,
        theta_jc_bot_c_per_w=None,
        theta_jb_c_per_w=None,
        max_junction_c=None,
        thermal_reference: str | None = None,
        clear: bool = False,
    ):
        args = ["project", "set-pool-part-thermal", path, "--pool", pool, "--part", part]
        fields = {
            "theta_ja_c_per_w": theta_ja_c_per_w,
            "theta_jc_top_c_per_w": theta_jc_top_c_per_w,
            "theta_jc_bot_c_per_w": theta_jc_bot_c_per_w,
            "theta_jb_c_per_w": theta_jb_c_per_w,
            "max_junction_c": max_junction_c,
            "thermal_reference": thermal_reference,
        }
        for key, value in fields.items():
            if value is not None:
                args.extend([f"--{key.replace('_', '-')}", str(value)])
        if clear:
            args.append("--clear")
        return self._run_cli_json(
            self.build_request("set_pool_part_thermal", {"path": path, "pool": pool, "part": part, **fields, "clear": clear}),
            args,
        )

    def set_pool_part_pad_map(self, path: str, pool: str, part: str, mode: str, entries: list[dict[str, str]]):
        args = ["project", "set-pool-part-pad-map", path, "--pool", pool, "--part", part, "--mode", mode]
        for entry in entries:
            args.extend(["--entry", f"{entry['pad']}:{entry['gate']}:{entry['pin']}"])
        return self._run_cli_json(
            self.build_request("set_pool_part_pad_map", {"path": path, "pool": pool, "part": part, "mode": mode, "entries": entries}),
            args,
        )

    def create_pool_pin_pad_map(self, path: str, pool: str, map: str, part: str, entries: list[dict[str, str]], footprint: str | None = None, set_default: bool = False):
        args = ["project", "create-pool-pin-pad-map", path, "--pool", pool, "--map", map, "--part", part]
        if footprint is not None:
            args.extend(["--footprint", footprint])
        if set_default:
            args.append("--set-default")
        for entry in entries:
            value = f"{entry['pad']}:{entry['gate']}:{entry['pin']}" if entry.get("gate") else f"{entry['pin']}:{entry['pad']}"
            args.extend(["--entry", value])
        return self._run_cli_json(
            self.build_request("create_pool_pin_pad_map", {"path": path, "pool": pool, "map": map, "part": part, "footprint": footprint, "entries": entries, "set_default": set_default}),
            args,
        )

    def set_pool_pin_pad_map(self, path: str, pool: str, map: str, mode: str, entries: list[dict[str, str]]):
        args = ["project", "set-pool-pin-pad-map", path, "--pool", pool, "--map", map, "--mode", mode]
        for entry in entries:
            value = f"{entry['pad']}:{entry['gate']}:{entry['pin']}" if entry.get("gate") else f"{entry['pin']}:{entry['pad']}"
            args.extend(["--entry", value])
        return self._run_cli_json(
            self.build_request("set_pool_pin_pad_map", {"path": path, "pool": pool, "map": map, "mode": mode, "entries": entries}),
            args,
        )

    setattr(client_cls, "get_pool_model_blobs", get_pool_model_blobs)
    setattr(client_cls, "gc_pool_model_blobs", gc_pool_model_blobs)
    setattr(client_cls, "set_pool_unit_pin", set_pool_unit_pin)
    setattr(client_cls, "set_pool_package_pad", set_pool_package_pad)
    setattr(client_cls, "create_pool_footprint", create_pool_footprint)
    setattr(client_cls, "generate_ipc7351b_soic", generate_ipc7351b_soic)
    setattr(client_cls, "set_pool_footprint_pad", set_pool_footprint_pad)
    setattr(client_cls, "set_pool_footprint_courtyard_rect", set_pool_footprint_courtyard_rect)
    setattr(client_cls, "set_pool_footprint_courtyard_polygon", set_pool_footprint_courtyard_polygon)
    setattr(client_cls, "add_pool_footprint_silkscreen_line", add_pool_footprint_silkscreen_line)
    setattr(client_cls, "add_pool_footprint_silkscreen_rect", add_pool_footprint_silkscreen_rect)
    setattr(client_cls, "add_pool_footprint_silkscreen_circle", add_pool_footprint_silkscreen_circle)
    setattr(client_cls, "add_pool_footprint_silkscreen_polygon", add_pool_footprint_silkscreen_polygon)
    setattr(client_cls, "set_pool_package_courtyard_rect", set_pool_package_courtyard_rect)
    setattr(client_cls, "set_pool_package_courtyard_polygon", set_pool_package_courtyard_polygon)
    setattr(client_cls, "add_pool_symbol_line", add_pool_symbol_line)
    setattr(client_cls, "add_pool_symbol_rect", add_pool_symbol_rect)
    setattr(client_cls, "add_pool_symbol_circle", add_pool_symbol_circle)
    setattr(client_cls, "add_pool_symbol_arc", add_pool_symbol_arc)
    setattr(client_cls, "add_pool_symbol_polygon", add_pool_symbol_polygon)
    setattr(client_cls, "add_pool_symbol_text", add_pool_symbol_text)
    setattr(client_cls, "set_pool_symbol_pin_anchor", set_pool_symbol_pin_anchor)
    setattr(client_cls, "add_pool_package_silkscreen_line", add_pool_package_silkscreen_line)
    setattr(client_cls, "add_pool_package_silkscreen_rect", add_pool_package_silkscreen_rect)
    setattr(client_cls, "add_pool_package_silkscreen_polygon", add_pool_package_silkscreen_polygon)
    setattr(client_cls, "add_pool_package_silkscreen_circle", add_pool_package_silkscreen_circle)
    setattr(client_cls, "add_pool_package_silkscreen_arc", add_pool_package_silkscreen_arc)
    setattr(client_cls, "add_pool_package_silkscreen_text", add_pool_package_silkscreen_text)
    setattr(client_cls, "add_pool_package_model_3d", add_pool_package_model_3d)
    setattr(client_cls, "set_pool_package_body_heights", set_pool_package_body_heights)
    setattr(client_cls, "set_pool_part_metadata", set_pool_part_metadata)
    setattr(client_cls, "set_pool_part_parametric", set_pool_part_parametric)
    setattr(client_cls, "set_pool_part_orderable_mpns", set_pool_part_orderable_mpns)
    setattr(client_cls, "set_pool_part_tags", set_pool_part_tags)
    setattr(client_cls, "set_pool_part_packaging_options", set_pool_part_packaging_options)
    setattr(client_cls, "set_pool_part_supply_chain", set_pool_part_supply_chain)
    setattr(client_cls, "set_pool_part_behavioural_models", set_pool_part_behavioural_models)
    setattr(client_cls, "attach_pool_part_model", attach_pool_part_model)
    setattr(client_cls, "detach_pool_part_model", detach_pool_part_model)
    setattr(client_cls, "set_pool_part_thermal", set_pool_part_thermal)
    setattr(client_cls, "set_pool_part_pad_map", set_pool_part_pad_map)
    setattr(client_cls, "create_pool_pin_pad_map", create_pool_pin_pad_map)
    setattr(client_cls, "set_pool_pin_pad_map", set_pool_pin_pad_map)
