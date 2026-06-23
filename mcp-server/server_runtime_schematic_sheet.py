#!/usr/bin/env python3
from __future__ import annotations
from typing import Any


def install_schematic_sheet_methods(client_cls: type, append_optional: Any) -> None:
    def run(self, method: str, params: dict[str, Any], args: list[str]):
        return self._run_cli_json(self.build_request(method, params), args)

    def create_sheet(self, path: str, name: str, sheet: str | None = None):
        args = ["project", "create-sheet", path, "--name", name]
        append_optional(args, "sheet", sheet)
        return run(self, "create_sheet", {"path": path, "name": name, "sheet": sheet}, args)

    def delete_sheet(self, path: str, sheet: str):
        return run(
            self,
            "delete_sheet",
            {"path": path, "sheet": sheet},
            ["project", "delete-sheet", path, "--sheet", sheet],
        )

    def rename_sheet(self, path: str, sheet: str, name: str):
        return run(
            self,
            "rename_sheet",
            {"path": path, "sheet": sheet, "name": name},
            ["project", "rename-sheet", path, "--sheet", sheet, "--name", name],
        )

    def create_sheet_definition(
        self,
        path: str,
        root_sheet: str,
        name: str,
        definition: str | None = None,
    ):
        args = [
            "project",
            "create-sheet-definition",
            path,
            "--root-sheet",
            root_sheet,
            "--name",
            name,
        ]
        append_optional(args, "definition", definition)
        return run(
            self,
            "create_sheet_definition",
            {
                "path": path,
                "root_sheet": root_sheet,
                "name": name,
                "definition": definition,
            },
            args,
        )

    def create_sheet_instance(
        self,
        path: str,
        definition: str,
        name: str,
        x_nm: int,
        y_nm: int,
        parent_sheet: str | None = None,
        instance: str | None = None,
    ):
        args = [
            "project",
            "create-sheet-instance",
            path,
            "--definition",
            definition,
            "--name",
            name,
            "--x-nm",
            str(x_nm),
            "--y-nm",
            str(y_nm),
        ]
        append_optional(args, "parent-sheet", parent_sheet)
        append_optional(args, "instance", instance)
        return run(
            self,
            "create_sheet_instance",
            {
                "path": path,
                "definition": definition,
                "parent_sheet": parent_sheet,
                "name": name,
                "x_nm": x_nm,
                "y_nm": y_nm,
                "instance": instance,
            },
            args,
        )

    def delete_sheet_instance(self, path: str, instance: str):
        return run(
            self,
            "delete_sheet_instance",
            {"path": path, "instance": instance},
            ["project", "delete-sheet-instance", path, "--instance", instance],
        )

    def move_sheet_instance(self, path: str, instance: str, x_nm: int, y_nm: int):
        return run(
            self,
            "move_sheet_instance",
            {"path": path, "instance": instance, "x_nm": x_nm, "y_nm": y_nm},
            [
                "project",
                "move-sheet-instance",
                path,
                "--instance",
                instance,
                "--x-nm",
                str(x_nm),
                "--y-nm",
                str(y_nm),
            ],
        )

    def bind_sheet_instance_port(self, path: str, instance: str, port: str):
        return run(
            self,
            "bind_sheet_instance_port",
            {"path": path, "instance": instance, "port": port},
            ["project", "bind-sheet-instance-port", path, "--instance", instance, "--port", port],
        )

    def unbind_sheet_instance_port(self, path: str, instance: str, port: str):
        return run(
            self,
            "unbind_sheet_instance_port",
            {"path": path, "instance": instance, "port": port},
            ["project", "unbind-sheet-instance-port", path, "--instance", instance, "--port", port],
        )

    for method in [
        create_sheet,
        delete_sheet,
        rename_sheet,
        create_sheet_definition,
        create_sheet_instance,
        delete_sheet_instance,
        move_sheet_instance,
        bind_sheet_instance_port,
        unbind_sheet_instance_port,
    ]:
        setattr(client_cls, method.__name__, method)
