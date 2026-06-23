#!/usr/bin/env python3
from __future__ import annotations


def install_project_query_methods(client_cls: type) -> None:
    def run_project_query(self, method: str, path: str, query: str):
        return self._run_cli_json(
            self.build_request(method, {"path": path}),
            ["project", "query", path, query],
        )

    def get_project_hierarchy(self, path: str | None = None):
        if path is None:
            return self.get_hierarchy()
        return run_project_query(self, "get_project_hierarchy", path, "hierarchy")

    def get_schematic_ports(self, path: str):
        return run_project_query(self, "get_schematic_ports", path, "ports")

    def get_schematic_noconnects(self, path: str):
        return run_project_query(self, "get_schematic_noconnects", path, "noconnects")

    def get_schematic_buses(self, path: str):
        return run_project_query(self, "get_schematic_buses", path, "buses")

    def get_schematic_bus_entries(self, path: str):
        return run_project_query(self, "get_schematic_bus_entries", path, "bus-entries")

    def get_schematic_texts(self, path: str):
        return run_project_query(self, "get_schematic_texts", path, "texts")

    def get_schematic_drawings(self, path: str):
        return run_project_query(self, "get_schematic_drawings", path, "drawings")

    def get_board_zones(self, path: str):
        return run_project_query(self, "get_board_zones", path, "board-zones")
    def get_board_texts(self, path: str):
        return run_project_query(self, "get_board_texts", path, "board-texts")
    def get_board_keepouts(self, path: str):
        return run_project_query(self, "get_board_keepouts", path, "board-keepouts")
    def get_board_outline(self, path: str):
        return run_project_query(self, "get_board_outline", path, "board-outline")
    def get_board_stackup(self, path: str):
        return run_project_query(self, "get_board_stackup", path, "board-stackup")
    def get_board_dimensions(self, path: str):
        return run_project_query(self, "get_board_dimensions", path, "board-dimensions")
    def get_board_nets(self, path: str):
        return run_project_query(self, "get_board_nets", path, "board-nets")
    def get_board_net_classes(self, path: str):
        return run_project_query(self, "get_board_net_classes", path, "board-net-classes")

    setattr(client_cls, "get_project_hierarchy", get_project_hierarchy)
    setattr(client_cls, "get_schematic_ports", get_schematic_ports)
    setattr(client_cls, "get_schematic_noconnects", get_schematic_noconnects)
    setattr(client_cls, "get_schematic_buses", get_schematic_buses)
    setattr(client_cls, "get_schematic_bus_entries", get_schematic_bus_entries)
    setattr(client_cls, "get_schematic_texts", get_schematic_texts)
    setattr(client_cls, "get_schematic_drawings", get_schematic_drawings)
    setattr(client_cls, "get_board_zones", get_board_zones)
    setattr(client_cls, "get_board_texts", get_board_texts)
    setattr(client_cls, "get_board_keepouts", get_board_keepouts)
    setattr(client_cls, "get_board_outline", get_board_outline)
    setattr(client_cls, "get_board_stackup", get_board_stackup)
    setattr(client_cls, "get_board_dimensions", get_board_dimensions)
    setattr(client_cls, "get_board_nets", get_board_nets)
    setattr(client_cls, "get_board_net_classes", get_board_net_classes)
