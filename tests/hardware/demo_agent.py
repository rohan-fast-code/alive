"""Demo local agent for Alive hardware tests.

Writes a timestamp line to a heartbeat file every 10 seconds. After a
closed lid run, a gap larger than 15 seconds in this file is proof that
the machine slept or the process was suspended (spec test TC31 and TC32).

Run:  python demo_agent.py heartbeat.log
"""
import sys
import time
from datetime import datetime

path = sys.argv[1] if len(sys.argv) > 1 else "heartbeat.log"
print(f"demo_agent: writing heartbeat lines to {path}", flush=True)
while True:
    line = datetime.now().isoformat(timespec="seconds")
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(line + "\n")
    print(line, flush=True)
    time.sleep(10)
