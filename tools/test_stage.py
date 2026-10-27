#!/usr/bin/env python3
"""Exercise staging against a local pinned Horizon checkout; never launch the game."""
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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--horizon', type=Path, default=horizon.ROOT.parent / 'horizon')
    args = parser.parse_args()
    checkout = args.horizon.resolve()
    status_before = subprocess.check_output(['git', '-C', str(checkout), 'status', '--porcelain'])
    original_bytes = {p: (checkout / p).read_bytes() for p in horizon.COMPAT['source_sha256']}
    with tempfile.TemporaryDirectory(prefix='veinminer-stage-test-', dir=horizon.ROOT / 'target') as scratch:
        stage, changed = horizon.stage(checkout, Path(scratch))
        assert changed
        first = snapshot(stage)
        again, changed = horizon.stage(checkout, Path(scratch))
        assert again == stage and not changed
        assert snapshot(stage) == first, 'unchanged stage bytes/mtimes changed'
        source = subprocess.check_output(['git','-C',str(checkout),'show',horizon.PIN+':crates/horizon-client/src/game.rs'], text=True)
        staged = (stage/'crates/horizon-client/src/game.rs').read_text()
        start = '    fn break_block_as_player('
        end = '    /// The hand whose use ended the click'
        assert source.split(start)[1].split(end)[0] == staged.split(start)[1].split(end)[0], 'original player mining implementation changed'
        marker = json.loads((stage/'.horizon-veinminer.json').read_text())
        source_file = stage / horizon.ENGINE
        source_file.write_text(source_file.read_text()+'\n// changed\n')
        try:
            horizon.stage(checkout,Path(scratch))
        except ValueError as error:
            assert 'staged source was edited' in str(error)
        else:
            raise AssertionError('edited stage was reused')
        for bad in ['', 'anchor anchor']:
            try:
                horizon.replace_once(bad, 'anchor', 'replacement', 'example')
            except ValueError:
                pass
            else:
                raise AssertionError('missing or competing adapter anchors accepted')
        assert all((checkout/p).read_bytes() == value for p,value in original_bytes.items())
        assert subprocess.check_output(['git','-C',str(checkout),'status','--porcelain']) == status_before
        print(json.dumps({'status':'ok','source_files_verified':len(marker['files']),
                          'deterministic_bytes_and_mtimes':True,'edited_stage_rejected':True,
                          'original_mining_method_unchanged':True,'original_checkout_unchanged':True},indent=2))


if __name__ == '__main__':
    main()
