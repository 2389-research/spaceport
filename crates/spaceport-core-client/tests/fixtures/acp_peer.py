# ABOUTME: A fixed ACP child for the private Core and host adapter process test.
# ABOUTME: It asks one permission and reports the selected answer without a provider.
import json
import sys


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


prompt_id = None
for line in sys.stdin:
    message = json.loads(line)
    method = message.get("method")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": message["id"], "result":
              {"protocolVersion": 1, "agentCapabilities": {}}})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"sessionId": "s1"}})
    elif method == "session/prompt":
        prompt_id = message["id"]
        send({"jsonrpc": "2.0", "id": 77, "method": "session/request_permission",
              "params": {"sessionId": "s1", "toolCall":
                         {"toolCallId": "fixture-call", "title": "Run fixture command", "kind": "execute"},
                         "options": [{"optionId": "allow", "name": "Allow", "kind": "allow_once"},
                                     {"optionId": "deny", "name": "Deny", "kind": "reject_once"}]}})
    elif message.get("id") == 77 and prompt_id is not None:
        chosen = message["result"]["outcome"].get("optionId")
        send({"jsonrpc": "2.0", "method": "session/update", "params":
              {"sessionId": "s1", "update": {"sessionUpdate": "agent_message_chunk",
                                               "content": {"type": "text", "text": f"chosen {chosen}"}}}})
        send({"jsonrpc": "2.0", "id": prompt_id, "result": {"stopReason": "end_turn"}})
