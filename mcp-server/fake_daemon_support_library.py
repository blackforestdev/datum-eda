#!/usr/bin/env python3
"""Fake daemon client native pool-library responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


def _package_silkscreen_footprint_uuid(package: str) -> str:
    return f"footprint-for-{package}"


class FakeDaemonClientLibraryMixin:
    def get_pool_library_objects(
        self,
        path: str,
        pool: str | None = None,
        kind: str | None = None,
        object: str | None = None,
        include_payload: bool | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("get_pool_library_objects", path, pool, kind, object, include_payload))
        payload = {
            "uuid": "symbol-test",
            "schema_version": 1,
            "name": "Symbol Test",
        } if include_payload else None
        result = {
            "contract": "native_project_library_objects_query_v1",
            "project_id": "project-test",
            "model_revision": "model-rev-test",
            "object_count": 1,
            "objects": [{
                "object_uuid": object or "symbol-test",
                "object_kind": kind or "symbols",
                "pool_path": pool or "pool",
                "relative_path": f"{pool or 'pool'}/{kind or 'symbols'}/{object or 'symbol-test'}.json",
                "object_revision": 0,
                "schema_version": 1,
                "source_hash": "sha256:library-object",
                **({"payload": payload} if payload is not None else {}),
            }],
        }
        return JsonRpcResponse("2.0", 178, result, None)

    def show_pool_library_object(
        self,
        path: str,
        object: str,
        pool: str | None = None,
        kind: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("show_pool_library_object", path, object, pool, kind))
        return JsonRpcResponse(
            "2.0",
            179,
            {
                "contract": "native_project_library_objects_query_v1",
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "object_count": 1,
                "objects": [{
                    "object_uuid": object,
                    "object_kind": kind or "symbols",
                    "pool_path": pool or "pool",
                    "relative_path": f"{pool or 'pool'}/{kind or 'symbols'}/{object}.json",
                    "object_revision": 0,
                    "schema_version": 1,
                    "source_hash": "sha256:library-object",
                    "payload": {"uuid": object, "schema_version": 1, "name": "Symbol Test"},
                }],
            },
            None,
        )

    def get_pool_model_blobs(
        self,
        path: str,
        pool: str | None = None,
        role: str | None = None,
        sha256: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("get_pool_model_blobs", path, pool, role, sha256))
        result = {
            "contract": "native_project_pool_models_query_v1",
            "project_id": "project-test",
            "model_revision": "model-rev-test",
            "model_count": 1,
            "models": [{
                "pool_path": pool or "pool",
                "role": role or "spice",
                "sha256": sha256 or "abc123",
                "relative_path": f"{pool or 'pool'}/models/{role or 'spice'}/{sha256 or 'abc123'}.lib",
                "computed_sha256": sha256 or "abc123",
                "hash_matches": True,
                "model_uuid": "model-uuid-test",
                "attachments": [],
            }],
        }
        return JsonRpcResponse("2.0", 182, result, None)

    def gc_pool_model_blobs(
        self,
        path: str,
        pool: str | None = None,
        role: str | None = None,
        sha256: str | None = None,
        apply: bool | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("gc_pool_model_blobs", path, pool, role, sha256, apply))
        result = {
            "contract": "native_project_pool_model_gc_v1",
            "action": "gc_pool_models",
            "pool_path": pool or "pool",
            "applied": bool(apply),
            "planned_count": 1,
            "deleted_count": 1 if apply else 0,
            "skipped_count": 0,
            "entries": [{
                "role": role or "spice",
                "sha256": sha256 or "abc123",
                "relative_path": f"{pool or 'pool'}/models/{role or 'spice'}/{sha256 or 'abc123'}.lib",
                "size_bytes": 12,
                "deleted": bool(apply),
                "skipped_reason": None,
            }],
        }
        return JsonRpcResponse("2.0", 203, result, None)

    def create_pool_library_object(
        self,
        path: str,
        pool: str,
        kind: str,
        object: str,
        from_json: str,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_library_object", path, pool, kind, object, from_json))
        return JsonRpcResponse(
            "2.0",
            180,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_pool_library_object",
                "project_root": path,
                "pool": pool,
                "object_kind": kind,
                "object_uuid": object,
                "from_json": from_json,
            },
            None,
        )

    def create_pool_unit(
        self,
        path: str,
        pool: str,
        unit: str,
        name: str,
        manufacturer: str,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_unit", path, pool, unit, name, manufacturer))
        return JsonRpcResponse(
            "2.0",
            183,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_unit",
                "project_root": path,
                "pool": pool,
                "object_kind": "units",
                "object_uuid": unit,
                "name": name,
                "manufacturer": manufacturer,
            },
            None,
        )

    def create_pool_symbol(
        self,
        path: str,
        pool: str,
        symbol: str,
        unit: str,
        name: str,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_symbol", path, pool, symbol, unit, name))
        return JsonRpcResponse(
            "2.0",
            184,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_symbol",
                "project_root": path,
                "pool": pool,
                "object_kind": "symbols",
                "object_uuid": symbol,
                "unit": unit,
                "name": name,
            },
            None,
        )

    def create_pool_entity(
        self,
        path: str,
        pool: str,
        entity: str,
        gate: str,
        unit: str,
        symbol: str,
        name: str,
        prefix: str,
        manufacturer: str,
        gate_name: str,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_entity", path, pool, entity, gate, unit, symbol, name, prefix, manufacturer, gate_name))
        return JsonRpcResponse(
            "2.0",
            185,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_entity",
                "project_root": path,
                "pool": pool,
                "object_kind": "entities",
                "object_uuid": entity,
                "gate": gate,
                "unit": unit,
                "symbol": symbol,
                "name": name,
                "prefix": prefix,
                "manufacturer": manufacturer,
                "gate_name": gate_name,
            },
            None,
        )

    def create_pool_padstack(
        self,
        path: str,
        pool: str,
        padstack: str,
        name: str,
        aperture: str | None = None,
        diameter_nm: int | None = None,
        width_nm: int | None = None,
        height_nm: int | None = None,
        drill_nm: int | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_padstack", path, pool, padstack, name, aperture, diameter_nm, width_nm, height_nm, drill_nm))
        return JsonRpcResponse(
            "2.0",
            186,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_padstack",
                "project_root": path,
                "pool": pool,
                "object_kind": "padstacks",
                "object_uuid": padstack,
                "name": name,
                "aperture": aperture,
                "diameter_nm": diameter_nm,
                "width_nm": width_nm,
                "height_nm": height_nm,
                "drill_nm": drill_nm,
            },
            None,
        )

    def create_pool_package(
        self,
        path: str,
        pool: str,
        package: str,
        name: str,
        pad: str | None = None,
        padstack: str | None = None,
        pad_name: str = "1",
        x_nm: int = 0,
        y_nm: int = 0,
        layer: int = 1,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_package", path, pool, package, name, pad, padstack, pad_name, x_nm, y_nm, layer))
        return JsonRpcResponse(
            "2.0",
            187,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_package",
                "project_root": path,
                "pool": pool,
                "object_kind": "packages",
                "object_uuid": package,
                "name": name,
                "pad": pad,
                "padstack": padstack,
                "pad_name": pad_name,
                "x_nm": x_nm,
                "y_nm": y_nm,
                "layer": layer,
            },
            None,
        )

    def create_pool_footprint(
        self,
        path: str,
        pool: str,
        footprint: str,
        package: str,
        name: str,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_footprint", path, pool, footprint, package, name))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_footprint",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": footprint,
                "package": package,
                "name": name,
            },
            None,
        )

    def generate_ipc7351b_soic(
        self,
        path: str,
        pool: str,
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
        density: str = "nominal",
        mask_expansion_nm: int = 50000,
        paste_reduction_nm: int = 50000,
        name: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "generate_ipc7351b_soic",
                path,
                pool,
                footprint,
                package,
                padstack,
                pads,
                package_code,
                pin_count,
                pitch_nm,
                body_length_nm,
                body_width_nm,
                lead_span_nm,
                terminal_length_nm,
                terminal_width_nm,
                density,
                mask_expansion_nm,
                paste_reduction_nm,
                name,
            )
        )
        return JsonRpcResponse(
            "2.0",
            189,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "generate_ipc7351b_soic",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": footprint,
                "package": package,
                "padstack": padstack,
                "pads": pads,
                "package_code": package_code,
                "pin_count": pin_count,
            },
            None,
        )

    def create_pool_part(
        self,
        path: str,
        pool: str,
        part: str,
        entity: str,
        package: str,
        mpn: str,
        manufacturer: str,
        value: str,
        description: str,
        datasheet: str,
        lifecycle: str,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_part", path, pool, part, entity, package, mpn, manufacturer, value, description, datasheet, lifecycle))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_part",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "entity": entity,
                "package": package,
                "mpn": mpn,
                "manufacturer": manufacturer,
                "value": value,
                "description": description,
                "datasheet": datasheet,
                "lifecycle": lifecycle,
            },
            None,
        )

    def set_pool_part_metadata(
        self,
        path: str,
        pool: str,
        part: str,
        mpn: str | None = None,
        manufacturer: str | None = None,
        manufacturer_jep106: int | None = None,
        value: str | None = None,
        description: str | None = None,
        datasheet: str | None = None,
        lifecycle: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_part_metadata", path, pool, part, mpn, manufacturer, manufacturer_jep106, value, description, datasheet, lifecycle))
        result = {
            "contract": "native_project_pool_library_object_mutation_v1",
            "action": "set_part_metadata",
            "project_root": path,
            "pool": pool,
            "object_kind": "parts",
            "object_uuid": part,
        }
        result.update({k: v for k, v in {"mpn": mpn, "manufacturer": manufacturer, "manufacturer_jep106": manufacturer_jep106, "value": value, "description": description, "datasheet": datasheet, "lifecycle": lifecycle}.items() if v is not None})
        return JsonRpcResponse("2.0", 188, result, None)

    def set_pool_part_parametric(
        self,
        path: str,
        pool: str,
        part: str,
        mode: str,
        params: dict[str, str],
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_part_parametric", path, pool, part, mode, params))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_part_parametric",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "mode": mode,
                "params": params,
            },
            None,
        )

    def set_pool_part_orderable_mpns(
        self,
        path: str,
        pool: str,
        part: str,
        mode: str,
        mpns: list[str],
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_part_orderable_mpns", path, pool, part, mode, mpns))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_part_orderable_mpns",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "mode": mode,
                "mpns": mpns,
            },
            None,
        )

    def set_pool_part_tags(
        self,
        path: str,
        pool: str,
        part: str,
        mode: str,
        tags: list[str],
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_part_tags", path, pool, part, mode, tags))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_part_tags",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "mode": mode,
                "tags": tags,
            },
            None,
        )

    def set_pool_part_packaging_options(
        self,
        path: str,
        pool: str,
        part: str,
        mode: str,
        options: list[str],
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_part_packaging_options", path, pool, part, mode, options))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_part_packaging_options",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "mode": mode,
                "options": options,
            },
            None,
        )

    def set_pool_part_behavioural_models(
        self,
        path: str,
        pool: str,
        part: str,
        mode: str,
        models: list[str],
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_part_behavioural_models", path, pool, part, mode, models))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_part_behavioural_models",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "mode": mode,
                "models": models,
            },
            None,
        )

    def set_pool_part_supply_chain(
        self,
        path: str,
        pool: str,
        part: str,
        clear: bool = False,
        checked_at: str | None = None,
        offers: list[str] | None = None,
    ) -> JsonRpcResponse:
        offer_payloads = offers or []
        self.calls.append(("set_pool_part_supply_chain", path, pool, part, clear, checked_at, offer_payloads))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_part_supply_chain",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "clear": clear,
                "checked_at": checked_at,
                "offers": offer_payloads,
            },
            None,
        )

    def attach_pool_part_model(
        self,
        path: str,
        pool: str,
        part: str,
        source: str,
        role: str,
        dialect: str | None = None,
        model_names: list[str] | None = None,
        encrypted: bool = False,
        encryption_scheme: str | None = None,
        vendor: str | None = None,
        fetched_at: str | None = None,
        format_metadata_json: str | None = None,
    ) -> JsonRpcResponse:
        names = model_names or []
        self.calls.append(("attach_pool_part_model", path, pool, part, source, role, dialect, names, encrypted, encryption_scheme, vendor, fetched_at, format_metadata_json))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "attach_part_model",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "source": source,
                "role": role,
                "dialect": dialect,
                "model_names": names,
                "encrypted": encrypted,
                "encryption_scheme": encryption_scheme,
                "vendor": vendor,
                "fetched_at": fetched_at,
                "format_metadata_json": format_metadata_json,
            },
            None,
        )

    def detach_pool_part_model(
        self,
        path: str,
        pool: str,
        part: str,
        attachment: str | None = None,
        model: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("detach_pool_part_model", path, pool, part, attachment, model))
        return JsonRpcResponse(
            "2.0",
            188,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "detach_part_model",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "attachment": attachment,
                "model": model,
            },
            None,
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
    ) -> JsonRpcResponse:
        self.calls.append((
            "set_pool_part_thermal",
            path,
            pool,
            part,
            theta_ja_c_per_w,
            theta_jc_top_c_per_w,
            theta_jc_bot_c_per_w,
            theta_jb_c_per_w,
            max_junction_c,
            thermal_reference,
            clear,
        ))
        result = {
            "contract": "native_project_pool_library_object_mutation_v1",
            "action": "set_part_thermal",
            "project_root": path,
            "pool": pool,
            "object_kind": "parts",
            "object_uuid": part,
            "clear": clear,
        }
        result.update({k: v for k, v in {
            "theta_ja_c_per_w": theta_ja_c_per_w,
            "theta_jc_top_c_per_w": theta_jc_top_c_per_w,
            "theta_jc_bot_c_per_w": theta_jc_bot_c_per_w,
            "theta_jb_c_per_w": theta_jb_c_per_w,
            "max_junction_c": max_junction_c,
            "thermal_reference": thermal_reference,
        }.items() if v is not None})
        return JsonRpcResponse("2.0", 188, result, None)

    def set_pool_unit_pin(
        self,
        path: str,
        pool: str,
        unit: str,
        pin: str,
        name: str,
        direction: str,
        swap_group: int,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_unit_pin", path, pool, unit, pin, name, direction, swap_group))
        return JsonRpcResponse(
            "2.0",
            190,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_unit_pin",
                "project_root": path,
                "pool": pool,
                "object_kind": "units",
                "object_uuid": unit,
                "pin": pin,
                "name": name,
                "direction": direction,
                "swap_group": swap_group,
            },
            None,
        )

    def set_pool_part_pad_map_entry(
        self,
        path: str,
        pool: str,
        part: str,
        pad: str,
        gate: str,
        pin: str,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_part_pad_map_entry", path, pool, part, pad, gate, pin))
        return JsonRpcResponse(
            "2.0",
            189,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_part_pad_map_entry",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "pad": pad,
                "gate": gate,
                "pin": pin,
            },
            None,
        )

    def set_pool_package_pad(
        self,
        path: str,
        pool: str,
        package: str,
        pad: str,
        padstack: str,
        pad_name: str,
        x_nm: int,
        y_nm: int,
        layer: int,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_package_pad", path, pool, package, pad, padstack, pad_name, x_nm, y_nm, layer))
        return JsonRpcResponse(
            "2.0",
            191,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_package_pad",
                "project_root": path,
                "pool": pool,
                "object_kind": "packages",
                "object_uuid": package,
                "pad": pad,
                "padstack": padstack,
                "pad_name": pad_name,
                "x_nm": x_nm,
                "y_nm": y_nm,
                "layer": layer,
            },
            None,
        )

    def set_pool_footprint_pad(
        self,
        path: str,
        pool: str,
        footprint: str,
        pad: str,
        padstack: str,
        pad_name: str,
        x_nm: int,
        y_nm: int,
        layer: int,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_footprint_pad", path, pool, footprint, pad, padstack, pad_name, x_nm, y_nm, layer))
        return JsonRpcResponse(
            "2.0",
            192,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_footprint_pad",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": footprint,
                "pad": pad,
                "padstack": padstack,
                "pad_name": pad_name,
                "x_nm": x_nm,
                "y_nm": y_nm,
                "layer": layer,
            },
            None,
        )

    def set_pool_package_courtyard_rect(
        self,
        path: str,
        pool: str,
        package: str,
        min_x_nm: int,
        min_y_nm: int,
        max_x_nm: int,
        max_y_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_package_courtyard_rect", path, pool, package, min_x_nm, min_y_nm, max_x_nm, max_y_nm))
        return JsonRpcResponse(
            "2.0",
            193,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_package_courtyard_rect",
                "project_root": path,
                "pool": pool,
                "object_kind": "packages",
                "object_uuid": package,
                "min_x_nm": min_x_nm,
                "min_y_nm": min_y_nm,
                "max_x_nm": max_x_nm,
                "max_y_nm": max_y_nm,
            },
            None,
        )

    def set_pool_footprint_courtyard_rect(
        self,
        path: str,
        pool: str,
        footprint: str,
        min_x_nm: int,
        min_y_nm: int,
        max_x_nm: int,
        max_y_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_footprint_courtyard_rect", path, pool, footprint, min_x_nm, min_y_nm, max_x_nm, max_y_nm))
        return JsonRpcResponse(
            "2.0",
            195,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_footprint_courtyard_rect",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": footprint,
                "min_x_nm": min_x_nm,
                "min_y_nm": min_y_nm,
                "max_x_nm": max_x_nm,
                "max_y_nm": max_y_nm,
            },
            None,
        )

    def set_pool_footprint_courtyard_polygon(
        self,
        path: str,
        pool: str,
        footprint: str,
        vertices: str,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_footprint_courtyard_polygon", path, pool, footprint, vertices))
        return JsonRpcResponse(
            "2.0",
            196,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_footprint_courtyard_polygon",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": footprint,
                "vertices": vertices,
            },
            None,
        )

    def add_pool_footprint_silkscreen_line(
        self,
        path: str,
        pool: str,
        footprint: str,
        from_x_nm: int,
        from_y_nm: int,
        to_x_nm: int,
        to_y_nm: int,
        width_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("add_pool_footprint_silkscreen_line", path, pool, footprint, from_x_nm, from_y_nm, to_x_nm, to_y_nm, width_nm))
        return JsonRpcResponse(
            "2.0",
            197,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "add_footprint_silkscreen_line",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": footprint,
                "from_x_nm": from_x_nm,
                "from_y_nm": from_y_nm,
                "to_x_nm": to_x_nm,
                "to_y_nm": to_y_nm,
                "width_nm": width_nm,
            },
            None,
        )

    def add_pool_footprint_silkscreen_rect(
        self,
        path: str,
        pool: str,
        footprint: str,
        min_x_nm: int,
        min_y_nm: int,
        max_x_nm: int,
        max_y_nm: int,
        width_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("add_pool_footprint_silkscreen_rect", path, pool, footprint, min_x_nm, min_y_nm, max_x_nm, max_y_nm, width_nm))
        return JsonRpcResponse(
            "2.0",
            198,
            {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_footprint_silkscreen_rect", "project_root": path, "pool": pool, "object_kind": "footprints", "object_uuid": footprint, "width_nm": width_nm},
            None,
        )

    def add_pool_footprint_silkscreen_circle(
        self,
        path: str,
        pool: str,
        footprint: str,
        center_x_nm: int,
        center_y_nm: int,
        radius_nm: int,
        width_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("add_pool_footprint_silkscreen_circle", path, pool, footprint, center_x_nm, center_y_nm, radius_nm, width_nm))
        return JsonRpcResponse(
            "2.0",
            199,
            {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_footprint_silkscreen_circle", "project_root": path, "pool": pool, "object_kind": "footprints", "object_uuid": footprint, "radius_nm": radius_nm, "width_nm": width_nm},
            None,
        )

    def add_pool_footprint_silkscreen_polygon(
        self,
        path: str,
        pool: str,
        footprint: str,
        vertices: str,
        closed: bool,
        width_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("add_pool_footprint_silkscreen_polygon", path, pool, footprint, vertices, closed, width_nm))
        return JsonRpcResponse(
            "2.0",
            200,
            {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_footprint_silkscreen_polygon", "project_root": path, "pool": pool, "object_kind": "footprints", "object_uuid": footprint, "vertices": vertices, "closed": closed, "width_nm": width_nm},
            None,
        )

    def set_pool_package_courtyard_polygon(
        self,
        path: str,
        pool: str,
        package: str,
        vertices: str,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_package_courtyard_polygon", path, pool, package, vertices))
        return JsonRpcResponse(
            "2.0",
            194,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_package_courtyard_polygon",
                "project_root": path,
                "pool": pool,
                "object_kind": "packages",
                "object_uuid": package,
                "vertices": vertices,
            },
            None,
        )

    def add_pool_package_silkscreen_line(
        self,
        path: str,
        pool: str,
        package: str,
        from_x_nm: int,
        from_y_nm: int,
        to_x_nm: int,
        to_y_nm: int,
        width_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("add_pool_package_silkscreen_line", path, pool, package, from_x_nm, from_y_nm, to_x_nm, to_y_nm, width_nm))
        return JsonRpcResponse(
            "2.0",
            194,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "add_package_silkscreen_line",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": _package_silkscreen_footprint_uuid(package),
                "from_x_nm": from_x_nm,
                "from_y_nm": from_y_nm,
                "to_x_nm": to_x_nm,
                "to_y_nm": to_y_nm,
                "width_nm": width_nm,
            },
            None,
        )

    def add_pool_symbol_line(self, path: str, pool: str, symbol: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int, width_nm: int) -> JsonRpcResponse:
        self.calls.append(("add_pool_symbol_line", path, pool, symbol, from_x_nm, from_y_nm, to_x_nm, to_y_nm, width_nm))
        return JsonRpcResponse("2.0", 201, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_symbol_line", "project_root": path, "pool": pool, "object_kind": "symbols", "object_uuid": symbol, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm, "width_nm": width_nm}, None)

    def add_pool_symbol_rect(self, path: str, pool: str, symbol: str, min_x_nm: int, min_y_nm: int, max_x_nm: int, max_y_nm: int, width_nm: int) -> JsonRpcResponse:
        self.calls.append(("add_pool_symbol_rect", path, pool, symbol, min_x_nm, min_y_nm, max_x_nm, max_y_nm, width_nm))
        return JsonRpcResponse("2.0", 203, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_symbol_rect", "project_root": path, "pool": pool, "object_kind": "symbols", "object_uuid": symbol, "min_x_nm": min_x_nm, "min_y_nm": min_y_nm, "max_x_nm": max_x_nm, "max_y_nm": max_y_nm, "width_nm": width_nm}, None)

    def add_pool_symbol_circle(self, path: str, pool: str, symbol: str, center_x_nm: int, center_y_nm: int, radius_nm: int, width_nm: int) -> JsonRpcResponse:
        self.calls.append(("add_pool_symbol_circle", path, pool, symbol, center_x_nm, center_y_nm, radius_nm, width_nm))
        return JsonRpcResponse("2.0", 204, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_symbol_circle", "project_root": path, "pool": pool, "object_kind": "symbols", "object_uuid": symbol, "center_x_nm": center_x_nm, "center_y_nm": center_y_nm, "radius_nm": radius_nm, "width_nm": width_nm}, None)

    def add_pool_symbol_arc(self, path: str, pool: str, symbol: str, x_nm: int, y_nm: int, radius_nm: int, start_angle: int, end_angle: int, width_nm: int) -> JsonRpcResponse:
        self.calls.append(("add_pool_symbol_arc", path, pool, symbol, x_nm, y_nm, radius_nm, start_angle, end_angle, width_nm))
        return JsonRpcResponse("2.0", 206, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_symbol_arc", "project_root": path, "pool": pool, "object_kind": "symbols", "object_uuid": symbol, "x_nm": x_nm, "y_nm": y_nm, "radius_nm": radius_nm, "start_angle": start_angle, "end_angle": end_angle, "width_nm": width_nm}, None)

    def add_pool_symbol_polygon(self, path: str, pool: str, symbol: str, vertices: str, closed: bool, width_nm: int) -> JsonRpcResponse:
        self.calls.append(("add_pool_symbol_polygon", path, pool, symbol, vertices, closed, width_nm))
        return JsonRpcResponse("2.0", 207, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_symbol_polygon", "project_root": path, "pool": pool, "object_kind": "symbols", "object_uuid": symbol, "vertices": vertices, "closed": closed, "width_nm": width_nm}, None)

    def add_pool_symbol_text(self, path: str, pool: str, symbol: str, text: str, x_nm: int, y_nm: int, rotation: int) -> JsonRpcResponse:
        self.calls.append(("add_pool_symbol_text", path, pool, symbol, text, x_nm, y_nm, rotation))
        return JsonRpcResponse("2.0", 205, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_symbol_text", "project_root": path, "pool": pool, "object_kind": "symbols", "object_uuid": symbol, "text": text, "x_nm": x_nm, "y_nm": y_nm, "rotation": rotation}, None)

    def set_pool_symbol_pin_anchor(self, path: str, pool: str, symbol: str, pin: str, x_nm: int, y_nm: int) -> JsonRpcResponse:
        self.calls.append(("set_pool_symbol_pin_anchor", path, pool, symbol, pin, x_nm, y_nm))
        return JsonRpcResponse("2.0", 202, {"contract": "native_project_pool_library_object_mutation_v1", "action": "set_symbol_pin_anchor", "project_root": path, "pool": pool, "object_kind": "symbols", "object_uuid": symbol, "pin": pin, "x_nm": x_nm, "y_nm": y_nm}, None)

    def add_pool_package_silkscreen_rect(
        self,
        path: str,
        pool: str,
        package: str,
        min_x_nm: int,
        min_y_nm: int,
        max_x_nm: int,
        max_y_nm: int,
        width_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("add_pool_package_silkscreen_rect", path, pool, package, min_x_nm, min_y_nm, max_x_nm, max_y_nm, width_nm))
        return JsonRpcResponse(
            "2.0",
            195,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "add_package_silkscreen_rect",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": _package_silkscreen_footprint_uuid(package),
                "min_x_nm": min_x_nm,
                "min_y_nm": min_y_nm,
                "max_x_nm": max_x_nm,
                "max_y_nm": max_y_nm,
                "width_nm": width_nm,
            },
            None,
        )

    def add_pool_package_silkscreen_polygon(self, path: str, pool: str, package: str, vertices: str, closed: bool, width_nm: int) -> JsonRpcResponse:
        self.calls.append(("add_pool_package_silkscreen_polygon", path, pool, package, vertices, closed, width_nm))
        return JsonRpcResponse("2.0", 199, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_package_silkscreen_polygon", "project_root": path, "pool": pool, "object_kind": "footprints", "object_uuid": _package_silkscreen_footprint_uuid(package), "vertices": vertices, "closed": closed, "width_nm": width_nm}, None)

    def add_pool_package_silkscreen_circle(
        self,
        path: str,
        pool: str,
        package: str,
        center_x_nm: int,
        center_y_nm: int,
        radius_nm: int,
        width_nm: int,
    ) -> JsonRpcResponse:
        self.calls.append(("add_pool_package_silkscreen_circle", path, pool, package, center_x_nm, center_y_nm, radius_nm, width_nm))
        return JsonRpcResponse(
            "2.0",
            196,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "add_package_silkscreen_circle",
                "project_root": path,
                "pool": pool,
                "object_kind": "footprints",
                "object_uuid": _package_silkscreen_footprint_uuid(package),
                "center_x_nm": center_x_nm,
                "center_y_nm": center_y_nm,
                "radius_nm": radius_nm,
                "width_nm": width_nm,
            },
            None,
        )

    def add_pool_package_silkscreen_text(self, path: str, pool: str, package: str, text: str, x_nm: int, y_nm: int, rotation: float) -> JsonRpcResponse:
        self.calls.append(("add_pool_package_silkscreen_text", path, pool, package, text, x_nm, y_nm, rotation))
        return JsonRpcResponse("2.0", 197, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_package_silkscreen_text", "project_root": path, "pool": pool, "object_kind": "footprints", "object_uuid": _package_silkscreen_footprint_uuid(package), "text": text, "x_nm": x_nm, "y_nm": y_nm, "rotation": rotation}, None)

    def add_pool_package_silkscreen_arc(self, path: str, pool: str, package: str, x_nm: int, y_nm: int, radius_nm: int, start_angle: int, end_angle: int, width_nm: int) -> JsonRpcResponse:
        self.calls.append(("add_pool_package_silkscreen_arc", path, pool, package, x_nm, y_nm, radius_nm, start_angle, end_angle, width_nm))
        return JsonRpcResponse("2.0", 198, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_package_silkscreen_arc", "project_root": path, "pool": pool, "object_kind": "footprints", "object_uuid": _package_silkscreen_footprint_uuid(package), "x_nm": x_nm, "y_nm": y_nm, "radius_nm": radius_nm, "start_angle": start_angle, "end_angle": end_angle, "width_nm": width_nm}, None)

    def add_pool_package_model_3d(self, path: str, pool: str, package: str, model_path: str, transform_json: str | None = None, format: str | None = None, tx_nm: int | None = None, ty_nm: int | None = None, tz_nm: int | None = None, roll_tenths_deg: int | None = None, pitch_tenths_deg: int | None = None, yaw_tenths_deg: int | None = None, scale: object | None = None) -> JsonRpcResponse:
        options = {"format": format, "tx_nm": tx_nm, "ty_nm": ty_nm, "tz_nm": tz_nm, "roll_tenths_deg": roll_tenths_deg, "pitch_tenths_deg": pitch_tenths_deg, "yaw_tenths_deg": yaw_tenths_deg, "scale": scale}
        self.calls.append(("add_pool_package_model_3d", path, pool, package, model_path, transform_json, format, tx_nm, ty_nm, tz_nm, roll_tenths_deg, pitch_tenths_deg, yaw_tenths_deg, scale))
        return JsonRpcResponse("2.0", 200, {"contract": "native_project_pool_library_object_mutation_v1", "action": "add_package_model_3d", "project_root": path, "pool": pool, "object_kind": "packages", "object_uuid": package, "model_path": model_path, "transform_json": transform_json, **options}, None)

    def set_pool_package_body_heights(self, path: str, pool: str, package: str, body_height_nm: int | None = None, body_height_mounted_nm: int | None = None, clear: bool | None = None) -> JsonRpcResponse:
        self.calls.append(("set_pool_package_body_heights", path, pool, package, body_height_nm, body_height_mounted_nm, clear))
        return JsonRpcResponse("2.0", 201, {"contract": "native_project_pool_library_object_mutation_v1", "action": "set_package_body_heights", "project_root": path, "pool": pool, "object_kind": "packages", "object_uuid": package, "body_height_nm": body_height_nm, "body_height_mounted_nm": body_height_mounted_nm, "clear": clear}, None)

    def set_pool_part_pad_map(
        self,
        path: str,
        pool: str,
        part: str,
        mode: str,
        entries: list[dict[str, str]],
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_part_pad_map", path, pool, part, mode, entries))
        return JsonRpcResponse(
            "2.0",
            192,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_part_pad_map",
                "project_root": path,
                "pool": pool,
                "object_kind": "parts",
                "object_uuid": part,
                "mode": mode,
                "entries": entries,
            },
            None,
        )

    def create_pool_pin_pad_map(
        self,
        path: str,
        pool: str,
        map: str,
        part: str,
        entries: list[dict[str, str]],
        footprint: str | None = None,
        set_default: bool = False,
    ) -> JsonRpcResponse:
        self.calls.append(("create_pool_pin_pad_map", path, pool, map, part, footprint, entries, set_default))
        return JsonRpcResponse(
            "2.0",
            202,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "create_pin_pad_map",
                "project_root": path,
                "pool": pool,
                "object_kind": "pin_pad_maps",
                "object_uuid": map,
                "part": part,
                "footprint": footprint,
                "entries": entries,
                "set_default": set_default,
            },
            None,
        )

    def set_pool_pin_pad_map(
        self,
        path: str,
        pool: str,
        map: str,
        mode: str,
        entries: list[dict[str, str]],
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_pin_pad_map", path, pool, map, mode, entries))
        return JsonRpcResponse(
            "2.0",
            203,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_pin_pad_map",
                "project_root": path,
                "pool": pool,
                "object_kind": "pin_pad_maps",
                "object_uuid": map,
                "mode": mode,
                "entries": entries,
            },
            None,
        )

    def delete_pool_library_object(
        self,
        path: str,
        pool: str,
        kind: str,
        object: str,
    ) -> JsonRpcResponse:
        self.calls.append(("delete_pool_library_object", path, pool, kind, object))
        return JsonRpcResponse(
            "2.0",
            181,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "delete_pool_library_object",
                "project_root": path,
                "pool": pool,
                "object_kind": kind,
                "object_uuid": object,
            },
            None,
        )

    def set_pool_library_object(
        self,
        path: str,
        pool: str,
        kind: str,
        object: str,
        from_json: str,
    ) -> JsonRpcResponse:
        self.calls.append(("set_pool_library_object", path, pool, kind, object, from_json))
        return JsonRpcResponse(
            "2.0",
            182,
            {
                "contract": "native_project_pool_library_object_mutation_v1",
                "action": "set_pool_library_object",
                "project_root": path,
                "pool": pool,
                "object_kind": kind,
                "object_uuid": object,
                "from_json": from_json,
            },
            None,
        )
