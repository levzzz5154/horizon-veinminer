#!/usr/bin/env python3
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def payload():
    adapter = (ROOT / "native/horizon.rs").read_text()
    return adapter.replace('include!("veinminer_engine.rs");', (ROOT / "src/lib.rs").read_text()).replace('include!("veinminer_tests.rs");', (ROOT / "tests/mining.rs").read_text())
if __name__ == "__main__":
    import sys
    path = ROOT / "mixins/veinminer/runtime.rs"
    if "--check" in sys.argv:
        if path.read_text() != payload(): raise SystemExit("run python3 tools/bundle.py to refresh the mod payload")
    else:
        path.write_text(payload())
