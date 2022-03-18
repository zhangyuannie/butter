#!/usr/bin/python3
# Butter daemon.
import sys
import os
import json
from typing import Optional, List, Dict
import btrfsutil
from datetime import datetime
import subprocess
import re

BTRFS_FS_TREE_OBJECTID = 5


class Subvolume:
    def __init__(self, path: str, info) -> None:
        self.path = path
        self.info = info
        self.snapshot_source: Optional[Subvolume] = None
        """the subvolume this subvolume is a snapshot of"""
        self.parent: Optional[Subvolume] = None
        """the subvolume which contains this subvolume"""
        self._mounted_path: Optional[str] = None

    @property
    def mounted_path(self) -> Optional[str]:
        """the absolute path this subvolume is mounted to"""
        if self._mounted_path:
            return self._mounted_path
        if self.parent == None:
            return None
        path = self.path.replace(self.parent.path, self.parent.mounted_path, 1)
        normpath = os.path.normpath(path)

        # https://github.com/python/cpython/commit/47abf240365ddd54a91c6ac167900d4bf6806c4f
        return normpath[1:] if normpath.startswith("//") else normpath

    @property
    def is_snapshot(self) -> str:
        return self.snapshot_source != None

    @staticmethod
    def enumerate_all() -> List["Subvolume"]:
        mounted_path_by_subvol_path = find_subvol_mnt()
        subvol_by_uuid: Dict[str, Subvolume] = {}
        subvol_by_id: Dict[int, Subvolume] = {}
        ret: List[Subvolume] = []

        # get partial subvolumes
        with btrfsutil.SubvolumeIterator("/", BTRFS_FS_TREE_OBJECTID, info=True) as it:
            for path, info in it:
                subvol = Subvolume(path, info)
                ret.append(subvol)
                subvol_by_uuid[info.uuid] = subvol
                subvol_by_id[info.id] = subvol

        # complete subvolumes
        for subvol in ret:
            subvol.parent = subvol_by_id.get(subvol.info.parent_id, None)
            if f"/{subvol.path}" in mounted_path_by_subvol_path:
                subvol._mounted_path = mounted_path_by_subvol_path[f"/{subvol.path}"]
            if subvol.info.parent_uuid.count(b"\x00") != len(subvol.info.parent_uuid):
                subvol.snapshot_source = subvol_by_uuid[subvol.info.parent_uuid]

        return ret

    def __str__(self) -> str:
        return f"Subvolume(path={self.path}, mounted_path={self.mounted_path})"


def find_subvol_mnt() -> Dict[str, str]:
    """Get a mapping from subvolume path to mounted path"""
    output = subprocess.check_output(["findmnt", "-t", "btrfs", "--json"])
    filesystems: List[Dict] = json.loads(output)["filesystems"]
    ret = {}
    extract_regex = r"\[([^]]+)"

    while filesystems:
        fs = filesystems.pop()
        subvol_path = re.search(extract_regex, fs["source"]).group(1)
        ret[subvol_path] = fs["target"]
        filesystems.extend(fs.get("children", []))

    return ret


def list_subvolumes(_):
    subvols = Subvolume.enumerate_all()

    return [
        {
            "path": subvol.path,
            "absolute_path": subvol.mounted_path,
            "creation_time": datetime.fromtimestamp(subvol.info.otime).isoformat(
                " ", "minutes"
            ),
            "snapshot_source_path": subvol.snapshot_source.path
            if subvol.snapshot_source
            else None,
        }
        for subvol in subvols
    ]


def rename_snapshot(args) -> Optional[str]:
    try:
        before, after = args
        os.rename(before, after)
        return None
    except Exception as e:
        return str(e)


def delete_snapshot(args):
    try:
        path = args[0]
        btrfsutil.delete_subvolume(path, recursive=True)
        return True
    except:
        return False


def create_snapshot(args) -> Optional[str]:
    try:
        src, dest, read_only = args

        # Make sure target directory exists
        os.makedirs(os.path.dirname(dest), exist_ok=True)

        btrfsutil.create_snapshot(src, dest, read_only=bool(read_only))
        return None
    except Exception as e:
        return str(e)


CMDS = {
    "list_subvolumes": list_subvolumes,
    "rename_snapshot": rename_snapshot,
    "delete_snapshot": delete_snapshot,
    "create_snapshot": create_snapshot,
}


if __name__ == "__main__":
    while True:
        req = sys.stdin.readline()
        if req == "":
            break
        cmd, *args = json.loads(req)
        if cmd in CMDS:
            print(cmd, file=sys.stderr)
            reply = CMDS[cmd](args)
            print(reply, file=sys.stderr)
            sys.stdout.write(json.dumps(reply))
            sys.stdout.write("\n")
            sys.stdout.flush()
        else:
            print(f"butterd: bad Request: {req}", file=sys.stderr)
            sys.exit(1)
