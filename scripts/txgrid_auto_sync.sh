#!/bin/bash
# Tx-grid auto-commit, push, and PR script
# Detects uncommitted changes, commits to dev via SSH, pushes, and manages PR dev->main via API

set -uo pipefail

PROJECT_DIR="/mnt/c/Users/Acer/Desktop/Code/tmux_work"
PAT_FILE="$PROJECT_DIR/scripts/txgrid_pat.txt"
LOG_FILE="$PROJECT_DIR/scripts/txgrid_cron.log"
REPO="naksh-atra/Tx-grid"
REMOTE="git@github.com:${REPO}.git"

# Ensure SSH agent has the key
export SSH_AUTH_SOCK=""
if [ -z "$SSH_AUTH_SOCK" ]; then
    eval "$(ssh-agent -s)" > /dev/null 2>&1
    ssh-add ~/.ssh/id_ed25519 > /dev/null 2>&1 || true
fi

log() {
    echo "$(date): $1" >> "$LOG_FILE"
}

cd "$PROJECT_DIR"

# Ensure we're on dev, fetch latest
git checkout dev 2>/dev/null
git fetch origin --quiet 2>/dev/null
git pull origin dev --quiet 2>/dev/null || true

# Stage any previously committed changes in staging area
git diff --cached --name-only | wc -l | tr -d ' ' > /dev/null 2>&1 || true

# Check for any uncommitted changes
DIFF_COUNT=$(git diff --name-only | wc -l | tr -d ' ')
UNTRACKED_RAW=$(git ls-files --others --exclude-standard 2>/dev/null | grep -v '^target/' | grep -v '^\.hermes/' | grep -v '^\.cache/' | grep -v '^scripts/txgrid_pat\.txt$' | grep -v '^scripts/txgrid_cron\.log$')
UNTRACKED_COUNT=$(echo "$UNTRACKED_RAW" | grep -c . 2>/dev/null || echo "0")

if [ "$DIFF_COUNT" -eq 0 ] && [ "$UNTRACKED_COUNT" -eq 0 ]; then
    log "No changes detected."
    exit 0
fi

log "Changes detected: modified=$DIFF_COUNT, untracked=$UNTRACKED_COUNT"

COMMITS_MADE=0

# Stage and commit each logical group separately for maximal commit count

# First: untracked files (one commit per file for max commits)
if [ "$UNTRACKED_COUNT" -gt 0 ]; then
    while IFS= read -r f; do
        [ -z "$f" ] && continue
        git add "$f" 2>/dev/null
        BASENAME=$(basename "$f")
        git -c user.name="naksh-atra" -c user.email="nakshatra.rajput@outlook.com" \
            commit -m "feat: add ${BASENAME}" --quiet 2>/dev/null
        COMMITS_MADE=$((COMMITS_MADE + 1))
        log "Committed untracked: $BASENAME"
    done <<< "$UNTRACKED_RAW"
fi

# Then: modified files grouped by module/directory
if [ "$DIFF_COUNT" -gt 0 ]; then
    # Group modified files by directory
    declare -A DIR_FILES
    while IFS= read -r f; do
        [ -z "$f" ] && continue
        DIR=$(dirname "$f")
        if [ -z "${DIR_FILES[$DIR]+x}" ]; then
            DIR_FILES[$DIR]="$f"
        else
            DIR_FILES[$DIR]="${DIR_FILES[$DIR]}"$'\n'"$f"
        fi
    done <<< "$(git diff --name-only)"

    # Commit each directory group separately
    for DIR in "${!DIR_FILES[@]}"; do
        FILES="${DIR_FILES[$DIR]}"
        
        # Stage files for this directory
        while IFS= read -r f; do
            [ -z "$f" ] && continue
            git add "$f" 2>/dev/null
        done <<< "$FILES"
        
        FILE_COUNT=$(echo "$FILES" | grep -c . || true)
        DIR_NAME=$(basename "$DIR")
        
        if [ "$FILE_COUNT" -gt 1 ]; then
            COMMIT_MSG="feat: update ${DIR_NAME}/ (${FILE_COUNT} files)"
        else
            BASENAME=$(basename "$(echo "$FILES" | head -1)")
            COMMIT_MSG="feat: update ${BASENAME}"
        fi
        
        git -c user.name="naksh-atra" -c user.email="nakshatra.rajput@outlook.com" \
            commit -m "$COMMIT_MSG" --quiet 2>/dev/null
        COMMITS_MADE=$((COMMITS_MADE + 1))
        log "Committed: $COMMIT_MSG"
    done

    unset DIR_FILES
fi

# Catch any remaining staged changes
REMAINING=$(git diff --cached --name-only | wc -l | tr -d ' ')
if [ "$REMAINING" -gt 0 ]; then
    git -c user.name="naksh-atra" -c user.email="nakshatra.rajput@outlook.com" \
        commit -m "chore: commit ${REMAINING} remaining file(s)" --quiet 2>/dev/null
    COMMITS_MADE=$((COMMITS_MADE + 1))
    log "Committed remaining: $REMAINING files"
fi

if [ "$COMMITS_MADE" -eq 0 ]; then
    log "Nothing to commit."
    exit 0
fi

# Push dev branch via SSH
PUSH_OUTPUT=$(git push origin dev 2>&1) || {
    log "PUSH FAILED: $PUSH_OUTPUT"
    exit 1
}
log "Pushed $COMMITS_MADE commit(s) to dev: ${PUSH_OUTPUT:-ok}"

# Read PAT for GitHub API calls
TOKEN=*** "$PAT_FILE" | tr -d '[:space:]')

# Check if PR from dev→main already exists
EXISTING_PR=$(curl -s \
    -H "Authorization: token $TOKEN" \
    -H "Accept: application/vnd.github.v3+json" \
    "https://api.github.com/repos/${REPO}/pulls?head=naksh-atra:dev&base=main&state=open" \
    | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if isinstance(data, list) and len(data) > 0:
        print(str(data[0]['number']))
    else:
        print('')
except:
    print('')
" 2>/dev/null || echo "")

LATEST_MSG=$(git log -1 --pretty=%s dev)
LATEST_SHA=$(git rev-parse --short dev)

if [ -n "$EXISTING_PR" ]; then
    log "PR #$EXISTING_PR exists, updating..."
    
    # Add comment about new commits
    COMMIT_LIST=$(git log "origin/main..dev" --oneline 2>/dev/null | head -10 | sed 's/^/- `/; s/$/`/' || echo "- ${LATEST_SHA} - ${LATEST_MSG}")
    curl -s -X POST \
        -H "Authorization: token $TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${REPO}/issues/${EXISTING_PR}/comments" \
        -d "{\"body\":\"New commits pushed to dev:\\n${COMMIT_LIST}\"}" > /dev/null 2>&1 || true
    
    # Check if mergeable
    MERGEABLE=$(curl -s \
        -H "Authorization: token $TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${REPO}/pulls/${EXISTING_PR}" \
        | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    m = d.get('mergeable')
    print('true' if m is True else 'false')
except:
    print('unknown')
" 2>/dev/null || echo "unknown")
    
    if [ "$MERGEABLE" = "true" ]; then
        MERGE_RESULT=$(curl -s -w "\n%{http_code}" -X PUT \
            -H "Authorization: token $TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/${REPO}/pulls/${EXISTING_PR}/merge" \
            -d "{\"merge_method\":\"squash\",\"commit_title\":\"${LATEST_MSG}\"}")
        MERGE_CODE=$(echo "$MERGE_RESULT" | tail -1)
        if [ "$MERGE_CODE" = "200" ] || [ "$MERGE_CODE" = "201" ]; then
            log "Merged PR #$EXISTING_PR to main"
            # Sync main back to dev
            git fetch origin --quiet
            git merge origin/main --quiet -m "chore: sync main into dev" 2>/dev/null || true
            git push origin dev --quiet 2>&1 || true
        else
            log "Merge failed (HTTP $MERGE_CODE)"
        fi
    else
        log "PR #$EXISTING_PR not mergeable yet: $MERGEABLE"
    fi
else
    # Create new PR from dev to main
    COMMIT_LIST=$(git log "origin/main..dev" --oneline 2>/dev/null | head -20 | sed 's/^/- `/; s/$/`/' || echo "- \`${LATEST_SHA}\` - ${LATEST_MSG}")
    PR_BODY="Auto PR: dev → main\\n\\nCommits:\\n${COMMIT_LIST}"
    
    PR_RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Authorization: token $TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        "https://api.github.com/repos/${REPO}/pulls" \
        -d "{\"title\":\"${LATEST_MSG}\",\"head\":\"dev\",\"base\":\"main\",\"body\":\"${PR_BODY}\"}")
    
    HTTP_CODE=$(echo "$PR_RESPONSE" | tail -1)
    PR_JSON=$(echo "$PR_RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "201" ]; then
        PR_NUM=$(echo "$PR_JSON" | python3 -c "
import sys, json
try:
    print(str(json.load(sys.stdin).get('number', '?')))
except:
    print('?')
" 2>/dev/null || echo "?")
        log "Created PR #${PR_NUM} (dev→main)"
        
        # Try auto-merge after brief delay for status checks
        sleep 3
        MERGEABLE=$(curl -s \
            -H "Authorization: token $TOKEN" \
            -H "Accept: application/vnd.github.v3+json" \
            "https://api.github.com/repos/${REPO}/pulls/${PR_NUM}" \
            | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    m = d.get('mergeable')
    print('true' if m is True else 'false')
except:
    print('unknown')
" 2>/dev/null || echo "unknown")
        
        if [ "$MERGEABLE" = "true" ]; then
            MERGE_RESULT=$(curl -s -w "\n%{http_code}" -X PUT \
                -H "Authorization: token $TOKEN" \
                -H "Accept: application/vnd.github.v3+json" \
                "https://api.github.com/repos/${REPO}/pulls/${PR_NUM}/merge" \
                -d "{\"merge_method\":\"squash\",\"commit_title\":\"${LATEST_MSG}\"}")
            MERGE_CODE=$(echo "$MERGE_RESULT" | tail -1)
            if [ "$MERGE_CODE" = "200" ] || [ "$MERGE_CODE" = "201" ]; then
                log "Auto-merged PR #${PR_NUM}"
                git fetch origin --quiet
                git merge origin/main --quiet -m "chore: sync main into dev" 2>/dev/null || true
                git push origin dev --quiet 2>&1 || true
            fi
        fi
    else
        ERROR_MSG="unknown"
        ERROR_MSG=$(echo "$PR_JSON" | python3 -c "
import sys, json
try:
    print(json.load(sys.stdin).get('message', 'unknown'))
except:
    print('parse error')
" 2>/dev/null) || true
        log "PR creation failed (HTTP $HTTP_CODE): $ERROR_MSG"
    fi
fi

log "Done."
