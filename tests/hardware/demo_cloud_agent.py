"""Demo cloud agent for Alive hardware tests.

Calls a public HTTPS endpoint every 60 seconds and logs the result, so a
closed lid run proves whether WiFi stayed connected (spec test TC36).

Run:  python demo_cloud_agent.py cloud.log
"""
import sys
import time
import urllib.request
from datetime import datetime

path = sys.argv[1] if len(sys.argv) > 1 else "cloud.log"
url = "https://example.com/"
print(f"demo_cloud_agent: calling {url} every 60 seconds, logging to {path}", flush=True)
while True:
    stamp = datetime.now().isoformat(timespec="seconds")
    try:
        with urllib.request.urlopen(url, timeout=15) as response:
            result = f"{stamp} OK status {response.status}"
    except Exception as exc:
        result = f"{stamp} FAILED {exc}"
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(result + "\n")
    print(result, flush=True)
    time.sleep(60)
