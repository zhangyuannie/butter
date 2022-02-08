#!/usr/bin/python3
# Butter daemon.
import sys
import json
from typing import List
import btrfsutil
from datetime import datetime


def list_snapshots():
    parent_subvol_by_uuid = {}
    snapshots = []
    with btrfsutil.SubvolumeIterator("/", 5, info=True) as it:
        for path, info in it:
            print(info, file=sys.stderr)
            if info.parent_uuid.count(b"\x00") == len(info.parent_uuid):
                parent_subvol_by_uuid[info.uuid] = (path, info)
            else:
                snapshots.append((path, info))

    return [
        {
            "path": path,
            "creation_time": datetime.fromtimestamp(info.otime).isoformat(
                " ", "minutes"
            ),
            "parent_path": parent_subvol_by_uuid[info.parent_uuid][0],
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
            print(f"Bad Request: {req}", file=sys.stderr)
            sys.exit(1)

        sys.stdout.write(json.dumps(reply))
        sys.stdout.write("\n")
        sys.stdout.flush()
else:
    sys.exit(1)
