#!/usr/bin/python3
# Butter daemon.
import sys
import json
from typing import List
import btrfsutil
from datetime import datetime

BTRFS_FS_TREE_OBJECTID = 5


def list_snapshots():
    subvol_by_uuid = {}
    with btrfsutil.SubvolumeIterator("/", BTRFS_FS_TREE_OBJECTID, info=True) as it:
        for path, info in it:
            subvol_by_uuid[info.uuid] = (path, info)

    snapshots = []
    with btrfsutil.SubvolumeIterator("/", info=True) as it:
        for path, info in it:
            if info.parent_uuid.count(b"\x00") != len(info.parent_uuid):
                snapshots.append((path, info))

    return [
        {
            "path": subvol_by_uuid[info.uuid][0],
            "absolute_path": f"/{path}",
            "creation_time": datetime.fromtimestamp(info.otime).isoformat(
                " ", "minutes"
            ),
            "parent_path": subvol_by_uuid[info.parent_uuid][0],
        }
        for path, info in snapshots
    ]


if __name__ == "__main__":
    while True:
        req = sys.stdin.readline()
        if req == "":
            break
        args: List[str] = json.loads(req)
        if args[0] == "list_snapshots":
            reply = list_snapshots()
        else:
            print(f"butterd: bad Request: {req}", file=sys.stderr)
            sys.exit(1)

        sys.stdout.write(json.dumps(reply))
        sys.stdout.write("\n")
        sys.stdout.flush()
else:
    sys.exit(1)
