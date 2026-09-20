#!/usr/bin/env python3
"""One bounded retry of the six selected public RPC samples that returned 429."""
import hashlib
import json
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path("evidence/transition/milestone6")
previous = json.loads((ROOT / "attempt-2/manifest.json").read_text())
out = ROOT / "attempt-3-retry"
out.mkdir(exist_ok=False)
records = []
for index in range(10, 16):
    signature = previous["selected_signatures"][index]
    request = {"jsonrpc": "2.0", "id": f"retry-{index:02d}", "method": "getTransaction",
               "params": [signature, {"encoding": "jsonParsed", "commitment": "finalized", "maxSupportedTransactionVersion": 0}]}
    name = f"retry-transaction-{index:02d}"
    body = out / f"{name}.body"
    headers = out / f"{name}.headers"
    run = subprocess.run(["curl", "--max-time", "25", "--silent", "--show-error", "--location",
                          "--dump-header", str(headers), "--output", str(body),
                          "--header", "Content-Type: application/json", "--data-binary", "@-",
                          "https://api.mainnet-beta.solana.com"], input=json.dumps(request).encode(),
                         capture_output=True, check=False)
    data = body.read_bytes() if body.exists() else run.stderr
    status = None
    for line in headers.read_text(errors="replace").splitlines() if headers.exists() else []:
        if line.startswith("HTTP/"):
            status = int(line.split()[1])
    records.append({"index": index, "signature": signature, "request": request,
                    "retrieved_at": datetime.now(timezone.utc).isoformat(), "http_status": status,
                    "curl_exit": run.returncode, "sha256": hashlib.sha256(data).hexdigest(),
                    "file": body.name if body.exists() else None})
    time.sleep(2)
(out / "manifest.json").write_text(json.dumps({"schema_version": 1,
    "original_manifest_sha256": hashlib.sha256((ROOT / "attempt-2/manifest.json").read_bytes()).hexdigest(),
    "records": records}, indent=2) + "\n")
print(json.dumps({"outcomes": [(r["index"], r["http_status"]) for r in records]}, indent=2))
