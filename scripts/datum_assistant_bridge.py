#!/usr/bin/env python3
import json
import os
import sys
import urllib.error
import urllib.request


SYSTEM_PROMPT = """You are Datum's embedded collaborative EDA assistant.

You work inside a shared human+AI PCB editing session.
You do not invent hidden state. You act only on the provided live context.

Return JSON only with this schema:
{
  "message": "short human-readable response",
  "actions": [
    { "type": "open_dock", "tab": "terminal" | "assistant" },
    { "type": "set_tool", "tool": "select" | "move" | "route" },
    { "type": "select_component_reference", "reference": "<refdes>" },
    { "type": "select_review_action", "action_id": "<action-id>" },
    { "type": "move_selected_by", "dx_nm": 0, "dy_nm": 0 },
    { "type": "begin_route_selected_proposal" },
    { "type": "apply_selected_route" },
    { "type": "cancel_active_edit" },
    { "type": "run_terminal_command", "command": "<shell command>" }
  ]
}

Rules:
- Be conservative and explicit.
- Prefer app-native actions over shell commands for board edits.
- Use shell commands for coding/development tasks when appropriate.
- If context is insufficient, ask a brief question in message and emit no actions.
- Never emit commands outside this schema.
"""


class AssistantBridge:
    def __init__(self):
        self.context = {}
        self.history = []
        self.model = os.environ.get("DATUM_ASSISTANT_MODEL", "gpt-5.4-mini")
        self.base_url = os.environ.get(
            "DATUM_ASSISTANT_BASE_URL", "https://api.openai.com/v1/chat/completions"
        )
        self.api_key = os.environ.get("OPENAI_API_KEY") or os.environ.get(
            "DATUM_ASSISTANT_API_KEY"
        )

    def emit(self, payload):
        sys.stdout.write(json.dumps(payload) + "\n")
        sys.stdout.flush()

    def emit_ready(self):
        configured = bool(self.api_key)
        message = (
            f"assistant bridge ready; backend model {self.model}"
            if configured
            else "assistant bridge ready; backend configuration is required"
        )
        self.emit({"type": "ready", "message": message, "configured": configured})

    def handle(self, msg):
        kind = msg.get("type")
        if kind == "context":
            self.context = msg.get("context") or {}
            return
        if kind != "user_message":
            self.emit({"type": "error", "message": f"unknown message type: {kind}"})
            return
        text = (msg.get("text") or "").strip()
        context = msg.get("context") or self.context
        self.context = context
        if not self.api_key:
            self.emit(
                {
                    "type": "response",
                    "message": (
                        "assistant backend is not configured in the host. "
                        "Set it from Datum assistant config and try again."
                    ),
                    "actions": [],
                }
            )
            return
        response = self.query_model(text, context)
        self.history.append({"role": "user", "content": text})
        self.history.append({"role": "assistant", "content": response.get("message", "")})
        self.emit(
            {
                "type": "response",
                "message": response.get("message", ""),
                "actions": response.get("actions", []),
            }
        )

    def query_model(self, text, context):
        messages = [{"role": "system", "content": SYSTEM_PROMPT}]
        for entry in self.history[-8:]:
            messages.append(entry)
        messages.append(
            {
                "role": "user",
                "content": (
                    "Live Datum context JSON:\n"
                    + json.dumps(context, indent=2)
                    + "\n\nUser request:\n"
                    + text
                    + "\n\nReturn JSON only."
                ),
            }
        )
        payload = {
            "model": self.model,
            "temperature": 0.2,
            "messages": messages,
            "response_format": {"type": "json_object"},
        }
        request = urllib.request.Request(
            self.base_url,
            data=json.dumps(payload).encode("utf-8"),
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {self.api_key}",
            },
            method="POST",
        )
        try:
            with urllib.request.urlopen(request, timeout=90) as response:
                body = json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as err:
            detail = err.read().decode("utf-8", errors="replace")
            return {
                "message": f"assistant backend HTTP {err.code}: {detail[:240]}",
                "actions": [],
            }
        except Exception as err:  # pragma: no cover - runtime error path
            return {"message": f"assistant backend failed: {err}", "actions": []}
        content = ""
        choices = body.get("choices") or []
        if choices:
            message = choices[0].get("message") or {}
            content = message.get("content") or ""
        try:
            parsed = json.loads(content)
        except Exception:
            return {
                "message": f"assistant backend returned invalid JSON: {content[:240]}",
                "actions": [],
            }
        if not isinstance(parsed, dict):
            return {"message": "assistant backend returned non-object JSON", "actions": []}
        actions = parsed.get("actions")
        if not isinstance(actions, list):
            actions = []
        message = parsed.get("message")
        if not isinstance(message, str):
            message = ""
        return {"message": message, "actions": actions}


def main():
    bridge = AssistantBridge()
    bridge.emit_ready()
    for raw in sys.stdin:
        line = raw.strip()
        if not line:
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError as err:
            bridge.emit({"type": "error", "message": f"invalid JSON input: {err}"})
            continue
        bridge.handle(message)


if __name__ == "__main__":
    main()
