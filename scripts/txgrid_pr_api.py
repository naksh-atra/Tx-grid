#!/usr/bin/env python3
"""
Tx-grid GitHub API part: PR creation/check and auto-merge.
Handles: check existing PR dev->main, create PR, auto-merge, sync main back to dev.
"""
import subprocess, json, os, time, urllib.request, urllib.error

PAT_FILE = os.environ.get("PAT_FILE", "/mnt/c/Users/Acer/Desktop/Code/tmux_work/scripts/txgrid_pat.txt")
REPO = "naksh-atra/Tx-grid"
LOG_FILE = os.environ.get("LOG_FILE", "/mnt/c/Users/Acer/Desktop/Code/tmux_work/scripts/txgrid_cron.log")
PROJECT_DIR = "/mnt/c/Users/Acer/Desktop/Code/tmux_work"

def log(msg):
    ts = subprocess.check_output(["date", "+%Y-%m-%d %H:%M:%S"]).decode().strip()
    with open(LOG_FILE, "a") as f:
        f.write(f"{ts}: [API] {msg}\n")

def run(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=PROJECT_DIR)
    return r.stdout.strip(), r.returncode

def gh(method, path, data=None):
    url = f"https://api.github.com{path}"
    body = json.dumps(data).encode() if data else None
    headers = {
        "Authorization": f"token {TOKEN}",
        "Accept": "application/vnd.github.v3+json",
        "Content-Type": "application/json",
    }
    req = urllib.request.Request(url, data=body, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read()), resp.status
    except urllib.error.HTTPError as e:
        err_body = ""
        try: err_body = e.read().decode()
        except: pass
        try: return json.loads(err_body), e.code
        except: return {}, e.code

def sync_main_to_dev():
    run(["git", "fetch", "origin", "--quiet"])
    run(["git", "merge", "origin/main", "--quiet", "-m", "chore: sync main into dev"])
    run(["git", "push", "origin", "dev", "--quiet"])

def main():
    # Read PAT
    TOKEN = None
    try:
        with open(PAT_FILE) as f:
            TOKEN = f.read().strip()
    except Exception as e:
        log(f"Cannot read PAT: {e}")
        return

    if not TOKEN:
        log("PAT is empty, skipping API calls")
        return

    os.chdir(PROJECT_DIR)

    # Get latest dev commit info
    latest_msg, _ = run(["git", "log", "-1", "--pretty=%s", "dev"])
    latest_sha, _ = run(["git", "rev-parse", "--short", "dev"])

    # Get new commits since main
    new_commits_raw, _ = run(["git", "log", "origin/main..dev", "--oneline"])
    if new_commits_raw:
        commit_lines = "\n".join(f"- {l}" for l in new_commits_raw.split("\n") if l)
    else:
        commit_lines = f"- {latest_sha} {latest_msg}"

    # Check existing PR
    prs, code = gh("GET", f"/repos/{REPO}/pulls?head=naksh-atra:dev&base=main&state=open")
    existing_pr = prs[0]["number"] if isinstance(prs, list) and prs else None

    if existing_pr:
        log(f"PR #{existing_pr} exists, updating...")
        gh("POST", f"/repos/{REPO}/issues/{existing_pr}/comments",
           {"body": f"New commits:\n{commit_lines}"})

        pr_data, _ = gh("GET", f"/repos/{REPO}/pulls/{existing_pr}")
        mergeable = pr_data.get("mergeable")
        if mergeable is True:
            _, mc = gh("PUT", f"/repos/{REPO}/pulls/{existing_pr}/merge",
                        {"merge_method": "squash", "commit_title": latest_msg})
            if mc in (200, 201):
                log(f"Merged PR #{existing_pr}")
                sync_main_to_dev()
            else:
                log(f"Merge failed (HTTP {mc})")
        else:
            log(f"PR #{existing_pr} not mergeable: {mergeable}")
    else:
        # Create new PR
        new_pr_data, hc = gh("POST", f"/repos/{REPO}/pulls", {
            "title": latest_msg,
            "head": "dev",
            "base": "main",
            "body": f"Auto PR: dev to main\n\nCommits:\n{commit_lines}"
        })
        if hc == 201:
            pr_num = new_pr_data.get("number", "?")
            log(f"Created PR #{pr_num}")
            time.sleep(3)
            pr_data, _ = gh("GET", f"/repos/{REPO}/pulls/{pr_num}")
            if pr_data.get("mergeable") is True:
                _, mc = gh("PUT", f"/repos/{REPO}/pulls/{pr_num}/merge",
                            {"merge_method": "squash", "commit_title": latest_msg})
                if mc in (200, 201):
                    log(f"Auto-merged PR #{pr_num}")
                    sync_main_to_dev()
        else:
            # Try to get error message
            try: err_msg = new_pr_data.get("message", "unknown")
            except: err_msg = "unknown"
            log(f"PR creation failed (HTTP {hc}): {err_msg}")

    log("API section done.")

if __name__ == "__main__":
    main()
