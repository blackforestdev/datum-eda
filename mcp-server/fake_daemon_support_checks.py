#!/usr/bin/env python3
"""Fake daemon client check and standards-repair responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


class FakeDaemonClientChecksMixin:
    def get_check_run(self, path: str, profile: str | None = None) -> JsonRpcResponse:
        self.calls.append(("get_check_run", path, profile))
        return JsonRpcResponse(
            "2.0",
            130,
            {
                "action": "native_project_check_run",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-test",
                "persisted": True,
                "check_run_id": "check-run-test",
                "profile_id": profile or "native-combined",
                "status": "error",
                "summary": {"errors": 1, "warnings": 0, "infos": 0, "waived": 0},
                "finding_count": 1,
                "profile_basis": {
                    "profile_id": profile or "native-combined",
                    "domains": ["relationships", "erc", "drc", "standards", "manufacturing"],
                    "description": "Native combined ERC/DRC/check profile",
                },
                "coverage": [
                    {
                        "domain": "standards",
                        "rule_id": "process_aperture_policy",
                        "status": "evaluated",
                        "target_scope": "board_pads_tracks_vias",
                        "basis_id": "datum.check.coverage.standards.process_aperture_policy.v1",
                        "rule_revision": "v1",
                        "standards_basis": "datum.process_aperture_and_geometry.current",
                    },
                    {
                        "domain": "erc",
                        "rule_id": "schematic_connectivity",
                        "status": "filtered_by_profile" if profile == "standards" else "evaluated",
                        "target_scope": "schematic",
                    },
                ],
                "proposal_refs": ["proposal-test"],
                "proposal_links": [
                    {
                        "proposal_id": "proposal-test",
                        "relationship": "repair_candidate",
                    }
                ],
                "findings": [
                    {
                        "id": "finding-test",
                        "fingerprint": "sha256:finding-test",
                        "domain": "standards",
                        "code": "process_aperture_policy",
                        "standards_basis": "datum.process_aperture_and_geometry.current",
                        "rule_revision": "v1",
                        "import_key": "kicad:board:/pads/0",
                        "severity": "error",
                        "status": "open",
                        "primary_target": {
                            "object_kind": "board_pad",
                            "object_id": "pad-test",
                        },
                        "related_targets": [
                            {
                                "object_kind": "board_footprint",
                                "object_id": "footprint-test",
                            }
                        ],
                        "message": "Pad mask/paste aperture policy is not standards-compliant.",
                        "explanation": (
                            "The pad aperture stack violates the active process "
                            "mask/paste profile."
                        ),
                        "suggested_next_action": (
                            "Generate and review a standards repair proposal."
                        ),
                        "evidence": [{"field": "mask_expansion_nm", "actual": 0}],
                        "proposal_refs": ["proposal-test"],
                    }
                ],
            },
            None,
        )

    def get_check_runs(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_check_runs", path))
        return JsonRpcResponse(
            "2.0",
            135,
            {
                "contract": "check_run_list_v1",
                "project_root": path,
                "check_run_count": 1,
                "check_runs": [
                    {
                        "check_run_id": "check-run-test",
                        "profile_id": "native-combined",
                        "status": "error",
                    }
                ],
            },
            None,
        )

    def show_check_run(self, path: str, check_run: str) -> JsonRpcResponse:
        self.calls.append(("show_check_run", path, check_run))
        return JsonRpcResponse(
            "2.0",
            136,
            {
                "contract": "check_run_record_v1",
                "project_root": path,
                "check_run": {
                    "check_run_id": check_run,
                    "profile_id": "native-combined",
                    "status": "error",
                    "finding_count": 1,
                    "profile_basis": {
                        "profile_id": "native-combined",
                        "domains": ["relationships", "erc", "drc", "standards", "manufacturing"],
                        "description": "Native combined ERC/DRC/check profile",
                    },
                    "coverage": [
                        {
                            "domain": "standards",
                            "rule_id": "process_aperture_policy",
                            "status": "evaluated",
                            "target_scope": "board_pads_tracks_vias",
                        }
                    ],
                    "proposal_refs": ["proposal-test"],
                    "proposal_links": [{"proposal_id": "proposal-test"}],
                    "findings": [
                        {
                            "finding_id": "finding-test",
                            "fingerprint": "sha256:finding-test",
                            "domain": "standards",
                            "rule_id": "process_aperture_policy",
                            "standards_basis": "datum.process_aperture_and_geometry.current",
                            "rule_revision": "v1",
                            "import_key": "kicad:board:/pads/0",
                            "severity": "error",
                            "status": "active",
                            "proposal_refs": ["proposal-test"],
                            "proposal_links": [{"proposal_id": "proposal-test"}],
                        }
                    ],
                },
            },
            None,
        )

    def get_check_profiles(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_check_profiles", path))
        return JsonRpcResponse(
            "2.0",
            137,
            {
                "contract": "check_profiles_v1",
                "project_root": path,
                "default_profile_id": "native-combined",
                "profile_count": 1,
                "profiles": [
                    {
                        "profile_id": "native-combined",
                        "name": "Native combined ERC/DRC/check profile",
                    }
                ],
            },
            None,
        )

    def get_zone_fills(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_zone_fills", path))
        return JsonRpcResponse(
            "2.0",
            132,
            {
                "contract": "zone_fills_query_v1",
                "project_root": path,
                "zone_fill_count": 1,
                "zone_fills": [{"zone_id": "zone-test", "state": "unfilled"}],
            },
            None,
        )

    def fill_zones(
        self,
        path: str,
        zone: str | None = None,
        net: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("fill_zones", path, zone, net))
        return JsonRpcResponse(
            "2.0",
            138,
            {
                "contract": "zone_fill_generate_v1",
                "action": "fill_zones",
                "project_root": path,
                "zone_fill_count": 1,
                "zone_fills": [{"zone_id": zone or "zone-test", "state": "unsupported"}],
            },
            None,
        )

    def generate_standards_repair_proposals(self, path: str) -> JsonRpcResponse:
        self.calls.append(("generate_standards_repair_proposals", path))
        return JsonRpcResponse(
            "2.0",
            131,
            {
                "action": "generate_standards_repair_proposals",
                "project_root": path,
                "proposal_count": 1,
                "proposals": [{"id": "proposal-test", "source": "check"}],
            },
            None,
        )

    def waive_finding(
        self,
        path: str,
        fingerprint: str,
        rationale: str,
        created_by: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("waive_finding", path, fingerprint, rationale, created_by))
        return JsonRpcResponse(
            "2.0",
            133,
            {
                "contract": "project_waive_finding_v1",
                "action": "waive_finding",
                "project_root": path,
                "fingerprint": fingerprint,
                "domain": "standards",
                "status": "applied",
            },
            None,
        )

    def accept_deviation(
        self,
        path: str,
        fingerprint: str,
        rationale: str,
        accepted_by: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("accept_deviation", path, fingerprint, rationale, accepted_by))
        return JsonRpcResponse(
            "2.0",
            134,
            {
                "contract": "project_accept_deviation_v1",
                "action": "accept_deviation",
                "project_root": path,
                "fingerprint": fingerprint,
                "domain": "standards",
                "status": "applied",
            },
            None,
        )
