#!/usr/bin/env python3
"""Verify the real TOML mixin plan, repeatability and pristine Horizon source."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import horizon


def snapshot(root):
    return {str(p.relative_to(root)): (hashlib.sha256(p.read_bytes()).hexdigest(), p.stat().st_mtime_ns)
            for p in root.rglob('*') if p.is_file()}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--horizon',type=Path,default=horizon.ROOT.parent/'horizon')
    args=parser.parse_args()
    checkout=args.horizon.resolve()
    status=subprocess.check_output(['git','-C',str(checkout),'status','--porcelain'])
    original={p:(checkout/p).read_bytes() for p in horizon.COMPAT['source_sha256']}
    with tempfile.TemporaryDirectory(prefix='veinminer-mixins-test-',dir=horizon.ROOT/'target') as scratch:
        root=Path(scratch)
        output,changed=horizon.mixin_stage(checkout,root)
        assert changed
        first=snapshot(output)
        second,changed=horizon.mixin_stage(checkout,root)
        assert second==output and not changed
        assert first==snapshot(output)
        plan=json.loads((root/'plan.json').read_text())
        assert plan['schema']==4
        assert len(plan['injections'])==3 and len(plan['modules'])==1
        assert {p['mod_id'] for p in plan['injections']}=={'veinminer'}
        pristine=list((root/'original').glob('*/.horizon-veinminer.json'))[0].parent
        subprocess.run(['cargo','run','--quiet','--locked','--offline','--manifest-path',str(horizon.ROOT/'tools/audit/Cargo.toml'),
                        '--target-dir',str(horizon.ROOT/'target/audit'),'--',str(pristine),str(output),str(root/'plan.json')],check=True)
        assert all((checkout/p).read_bytes()==b for p,b in original.items())
        assert subprocess.check_output(['git','-C',str(checkout),'status','--porcelain'])==status
        print(json.dumps({'status':'ok','injections':3,'modules':1,'source_rewriter':'horizon-mixins',
                          'deterministic_bytes_and_mtimes':True,'original_asts_preserved':True,
                          'original_checkout_unchanged':True},indent=2))

if __name__=='__main__': main()
