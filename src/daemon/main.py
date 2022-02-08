#!/usr/bin/python3
# Butter daemon.
import sys
import os
import json
import btrfsutil
from datetime import datetime

BTRFS_FS_TREE_OBJECTID = 5


def list_snapshots(_):
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


def rename_snapshot(args):
    try:
        before, after = args
        os.rename(before, after)
        return True
    except:
        return False


def delete_snapshot(args):
    try:
        path = args[0]
        btrfsutil.delete_subvolume(path, recursive=True)
        return True
    except:
        return False


CMDS = {
    "list_snapshots": list_snapshots,
    "rename_snapshot": rename_snapshot,
    "delete_snapshot": delete_snapshot,
}


if __name__ == "__main__":
    while True:
        req = sys.stdin.readline()
        if req == "":
            break
        cmd, *args = json.loads(req)
        if cmd in CMDS:
            reply = CMDS[cmd](args)
            sys.stdout.write(json.dumps(reply))
            sys.stdout.write("\n")
            sys.stdout.flush()
        else:
            print(f"butterd: bad Request: {req}", file=sys.stderr)
            sys.exit(1)

else:
    sys.exit(1)
