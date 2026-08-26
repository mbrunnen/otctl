#!/usr/bin/env python3
"""Minimal stand-in for an OpenThread Border Router REST API (v0.3.0).

Serves just enough of the endpoints in openapi/ot-rest.yaml, with canned
responses, so `otctl` can be exercised end-to-end without a real device.
No third-party dependencies - stdlib http.server only.

Usage: mock_otbr.py [port]   (default 8123)
Prints "MOCK-OTBR-READY" once listening.
"""
import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8123

NODE_INFO = {
    "state": "leader",
    "rloc16": "0x4400",
    "networkName": "OpenThread",
    "extAddress": "1122334455667788",
    "extPanId": "dead00beef00cafe",
    "numOfRouter": 1,
}

SIMPLE_GET = {
    "/node/state": "\"leader\"",
    "/node/rloc": "\"0x1400\"",
    "/node/rloc16": "\"0x4400\"",
    "/node/ext-address": "\"1122334455667788\"",
    "/node/network-name": "\"OpenThread\"",
    "/node/leader-data": json.dumps({"partitionId": 1, "weighting": 64, "dataVersion": 1,
                                      "stableDataVersion": 1, "leaderRouterId": 0}),
    "/node/ext-panid": "\"dead00beef00cafe\"",
    "/node/num-of-router": "1",
    "/node/ba-id": "\"550e8400-e29b-41d4-a716-446655440000\"",
    "/node/coprocessor/version": "\"OPENTHREAD/thread-reference-20240130\"",
    "/node/dataset/active": json.dumps({"NetworkName": "OpenThread", "PanId": 4660,
                                         "Channel": 15}),
    "/node/dataset/pending": json.dumps({"NetworkName": "OpenThread", "PanId": 4660,
                                          "Channel": 15, "DelayTimer": 30000}),
    "/node/commissioner/state": "\"disabled\"",
    "/node/commissioner/joiner": "[]",
    "/api/devices": json.dumps({"data": []}),
    "/api/diagnostics": json.dumps({"data": []}),
    "/api/actions": json.dumps({"data": []}),
}

WRITE_PATHS = {
    "/node/state", "/node/dataset/active", "/node/dataset/pending",
    "/node/commissioner/state", "/node/commissioner/joiner",
}


class Handler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        sys.stderr.write("mock_otbr: " + (format % args) + "\n")

    def _send(self, status, body=None):
        self.send_response(status)
        if body is not None:
            data = body.encode() if isinstance(body, str) else body
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)
        else:
            self.send_header("content-length", "0")
            self.end_headers()

    def do_GET(self):
        if self.path == "/node":
            return self._send(200, json.dumps(NODE_INFO))
        if self.path in SIMPLE_GET:
            return self._send(200, SIMPLE_GET[self.path])
        if self.path.startswith("/api/actions/"):
            action_id = self.path.rsplit("/", 1)[-1]
            body = json.dumps({"data": {"id": action_id, "type": "getNetworkDiagnosticTask",
                                         "attributes": {"status": "completed"}}})
            return self._send(200, body)
        if self.path.startswith("/api/devices/"):
            return self._send(200, json.dumps({"data": {"id": self.path.rsplit("/", 1)[-1]}}))
        if self.path.startswith("/api/diagnostics/"):
            return self._send(200, json.dumps({"data": {"id": self.path.rsplit("/", 1)[-1]}}))
        self._send(404, json.dumps({"errors": [{"title": "not found"}]}))

    def do_PUT(self):
        if self.path in WRITE_PATHS:
            self._read_body()
            return self._send(200)
        self._send(404, json.dumps({"errors": [{"title": "not found"}]}))

    def do_POST(self):
        if self.path == "/node/commissioner/joiner":
            self._read_body()
            return self._send(200)
        if self.path == "/api/actions":
            body = self._read_body()
            try:
                action_type = json.loads(body)["data"][0]["type"]
            except (KeyError, IndexError, ValueError):
                action_type = "unknown"
            resp = json.dumps({"data": [{"id": "a1", "type": action_type,
                                          "attributes": {"status": "completed"}}]})
            return self._send(200, resp)
        self._send(404, json.dumps({"errors": [{"title": "not found"}]}))

    def do_DELETE(self):
        if self.path == "/node":
            return self._send(200)
        if self.path in ("/api/actions", "/api/devices", "/api/diagnostics"):
            return self._send(200)
        if self.path.startswith(("/api/devices/", "/api/diagnostics/")):
            return self._send(200)
        if self.path == "/node/commissioner/joiner":
            return self._send(200)
        self._send(404, json.dumps({"errors": [{"title": "not found"}]}))

    def _read_body(self):
        length = int(self.headers.get("content-length", 0))
        return self.rfile.read(length) if length else b""


def main():
    server = HTTPServer(("127.0.0.1", PORT), Handler)
    print("MOCK-OTBR-READY", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
