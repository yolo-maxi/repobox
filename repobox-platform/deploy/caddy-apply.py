#!/usr/bin/env python3
"""Apply (or roll back) the repobox-platform managed block in the main Caddyfile.

Runs ON THE WEB HOST as root (via sudo). It never prints file contents.

apply:
  1. back up /etc/caddy/Caddyfile to /etc/caddy/backups/Caddyfile.pre-repobox-platform-<stamp>
  2. replace either the existing managed block (between the BEGIN/END markers)
     or the exact legacy `auth.repo.box { ... reverse_proxy 127.0.0.1:3005 }`
     block with the managed block file; refuse if neither matches exactly
  3. `caddy validate` the candidate; only then move it into place and reload
  4. print the backup path (the rollback route)

rollback:
  restore the given backup and reload.
"""
import argparse, os, re, shutil, subprocess, sys, tempfile, time

CADDYFILE = "/etc/caddy/Caddyfile"
BACKUP_DIR = "/etc/caddy/backups"
BEGIN = "# BEGIN repobox-platform managed\n"
END = "# END repobox-platform managed\n"

LEGACY_BLOCK = (
    "auth.repo.box {\n"
    "\theader {\n"
    "\t\tX-Content-Type-Options \"nosniff\"\n"
    "\t\tX-Frame-Options \"DENY\"\n"
    "\t\tReferrer-Policy \"strict-origin-when-cross-origin\"\n"
    "\t\tStrict-Transport-Security \"max-age=31536000; includeSubDomains; preload\"\n"
    "\t}\n"
    "\treverse_proxy 127.0.0.1:3005\n"
    "}\n"
)


# The live Caddyfile must stay world-readable: `caddy reload` runs as the
# caddy user, while backups are root-only (0640). Copying a backup's mode onto
# the live file breaks every subsequent reload with "permission denied".
LIVE_MODE = 0o644


def install_live(src):
    tmp = CADDYFILE + ".restore"
    shutil.copyfile(src, tmp)
    os.chmod(tmp, LIVE_MODE)
    os.replace(tmp, CADDYFILE)


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True)


def validate(path):
    r = run(["caddy", "validate", "--config", path, "--adapter", "caddyfile"])
    if r.returncode != 0:
        sys.stderr.write("caddy validate FAILED:\n" + r.stderr[-2000:] + "\n")
    else:
        print("caddy validate: OK")
    return r.returncode == 0


def reload():
    r = run(["systemctl", "reload", "caddy"])
    if r.returncode != 0:
        sys.stderr.write("caddy reload FAILED:\n" + r.stderr[-2000:] + "\n")
        return False
    print("caddy reload: OK")
    return True


def apply(block_path):
    with open(CADDYFILE) as f:
        current = f.read()
    with open(block_path) as f:
        block = f.read()
    if not (block.startswith(BEGIN) and block.endswith(END)):
        sys.exit("managed block file must start/end with the markers")

    if BEGIN in current:
        start = current.index(BEGIN)
        end = current.index(END, start) + len(END)
        candidate = current[:start] + block + current[end:]
        mode = "updated existing managed block"
    elif current.count(LEGACY_BLOCK) == 1:
        candidate = current.replace(LEGACY_BLOCK, block, 1)
        mode = "replaced legacy auth.repo.box block (3005) with managed block"
    else:
        n = len(re.findall(r"^auth\.repo\.box\b", current, re.M))
        sys.exit(
            f"CONFLICT: no managed block and the legacy auth.repo.box block does not match exactly "
            f"({n} auth.repo.box site definitions found). Nothing changed; inspect the Caddyfile by hand."
        )

    if candidate == current:
        print("no change needed")
        return
    os.makedirs(BACKUP_DIR, exist_ok=True)
    stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    backup = os.path.join(BACKUP_DIR, f"Caddyfile.pre-repobox-platform-{stamp}")
    shutil.copy2(CADDYFILE, backup)
    os.chmod(backup, 0o640)
    fd, tmp = tempfile.mkstemp(prefix="Caddyfile.candidate.", dir="/etc/caddy")
    with os.fdopen(fd, "w") as f:
        f.write(candidate)
    if not validate(tmp):
        os.unlink(tmp)
        sys.exit(f"candidate rejected; live Caddyfile untouched (backup at {backup})")
    os.chmod(tmp, LIVE_MODE)
    os.replace(tmp, CADDYFILE)
    print(mode)
    if not reload():
        install_live(backup)
        reload()
        sys.exit("reload failed; restored previous Caddyfile")
    print(f"backup: {backup}")
    print(f"rollback: sudo python3 {os.path.abspath(__file__)} rollback {backup}")


def rollback(backup):
    if not os.path.isfile(backup):
        sys.exit(f"no such backup {backup}")
    if not validate(backup):
        sys.exit("backup does not validate; not applying")
    install_live(backup)
    if not reload():
        sys.exit("reload failed after restore")
    print(f"restored {backup}")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("apply")
    a.add_argument("block")
    r = sub.add_parser("rollback")
    r.add_argument("backup")
    args = ap.parse_args()
    if os.geteuid() != 0:
        sys.exit("run with sudo")
    if args.cmd == "apply":
        apply(args.block)
    else:
        rollback(args.backup)
