#!/usr/bin/env python3
"""Bounded public, read-only Milestone 6 research capture. No credentials or writes to RPC."""
import hashlib
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urljoin


OUT = Path("evidence/transition/milestone6") / f"attempt-{sys.argv[1]}"
OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.mkdir(parents=True, exist_ok=False)
RPC = "https://api.mainnet-beta.solana.com"
ADDRESSES = [
    ("source-mint", "PreANxuXjsy2pvisWWMNB6YaJNzr7681wJJr2rHsfTh"),
    ("successor-mint", "Xs3oZwbHvqis4NYcf4YKWmEia2eC84wSiVrcYcTqpH8"),
    ("source-mint-authority", "WV9PJN7XTmTLVwbutCLFxp8TyePee6Xq5mRq6Fti5Wc"),
    ("successor-mint-authority", "7pt9tkctJPK7PPNQJ77GKg8ZffSF6QxoMiCFYHxrtaCj"),
]
records = []


def fetch(name, url, payload=None):
    body_path = OUT / f"{name}.body"
    header_path = OUT / f"{name}.headers"
    command = ["curl", "--max-time", "25", "--silent", "--show-error", "--location",
               "--dump-header", str(header_path), "--output", str(body_path),
               "--user-agent", "Eplyx-read-only-public-research/1.0"]
    if payload is not None:
        command += ["--header", "Content-Type: application/json", "--data-binary", "@-"]
    command += [url]
    run = subprocess.run(command, input=payload, capture_output=True, check=False)
    body = body_path.read_bytes() if body_path.exists() else run.stderr
    raw_headers = header_path.read_text(errors="replace") if header_path.exists() else ""
    response_headers = {}
    status = None
    for block in raw_headers.split("\r\n\r\n"):
        if block.startswith("HTTP/"):
            lines = block.splitlines()
            status = int(lines[0].split()[1])
            response_headers = dict(line.split(":", 1) for line in lines[1:] if ":" in line)
    response_headers = {k.lower(): v.strip() for k, v in response_headers.items()}
    if len(body) > 8 * 1024 * 1024:
        raise RuntimeError(f"response too large: {name}")
    file = body_path.name
    if not body_path.exists():
        body_path.write_bytes(body)
    record = {
        "name": name,
        "url": url,
        "retrieved_at": datetime.now(timezone.utc).isoformat(),
        "http_status": status,
        "content_type": response_headers.get("content-type"),
        "sha256": hashlib.sha256(body).hexdigest(),
        "file": file,
        "request": json.loads(payload) if payload else None,
        "error": body.decode(errors="replace")[:300] if run.returncode != 0 else None,
    }
    records.append(record)
    return body, status


def rpc(name, method, params):
    body, _ = fetch(
        name,
        RPC,
        json.dumps({"jsonrpc": "2.0", "id": name, "method": method, "params": params}).encode(),
    )
    try:
        return json.loads(body)
    except ValueError:
        return {"transport_error": True}


pages = {}
for name, route in (("issuer-spacex", "/spacex"), ("issuer-faq", "/faq")):
    pages[name], _ = fetch(name, urljoin("https://prestocks.com", route))

scripts = set()
for body in pages.values():
    scripts.update(re.findall(rb'<script[^>]+src="([^"]+\.js)"', body))
scripts = [s.decode() for s in scripts if s.startswith(b"/_next/static/")]
scripts.sort(key=lambda s: (0 if "/app/" in s else 1, 0 if "company" in s or "faq" in s else 1, s))
for index, path in enumerate(scripts[:12]):
    fetch(f"issuer-script-{index:02d}", urljoin("https://prestocks.com", path))

searches = []
for role, address in ADDRESSES:
    result = rpc(f"signatures-{role}", "getSignaturesForAddress", [address, {"limit": 20, "commitment": "finalized"}])
    rows = result.get("result") if isinstance(result, dict) else None
    searches.append({"role": role, "address": address, "returned": len(rows) if isinstance(rows, list) else None,
                     "error": result.get("error") if isinstance(result, dict) else "invalid response",
                     "entries": rows if isinstance(rows, list) else []})

selected = []
for search in searches:
    # Four entries per address give a fixed 16-transaction sample without replacement.
    for entry in search["entries"][:4]:
        signature = entry.get("signature")
        if signature and signature not in selected:
            selected.append(signature)
for index, signature in enumerate(selected[:16]):
    rpc(f"transaction-{index:02d}", "getTransaction", [signature, {"encoding": "jsonParsed", "commitment": "finalized", "maxSupportedTransactionVersion": 0}])

manifest = {"schema_version": 1, "purpose": "Milestone 6 bounded public mechanism search", "rpc": RPC,
            "signature_limit_per_address": 20, "transaction_limit": 16, "searches": searches,
            "selected_signatures": selected[:16], "records": records}
(OUT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print(json.dumps({"pages": len(pages), "scripts": min(len(scripts), 12), "searches": [(s["role"], s["returned"], s["error"]) for s in searches],
                  "transactions": len(selected[:16]), "records": len(records)}, indent=2))
