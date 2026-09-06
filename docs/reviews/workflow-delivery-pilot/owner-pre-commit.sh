#!/usr/bin/env bash
# Owner-installed OUTSIDE the development worktree after explicit promotion.
# Preparation never installs this hook or writes the required Git configuration.
set -euo pipefail
wdq_repo=$(git rev-parse --show-toplevel)
wdq_repo=$(realpath -- "$wdq_repo")
cd "$wdq_repo"
wdq_fail() { echo "WDQ-TRUST: $*" >&2; exit 2; }
wdq_authority=$(git config --local --get datum.workflowDeliveryAuthorityRef) || wdq_fail 'missing authority'
wdq_base=$(git config --local --get datum.workflowDeliveryBaseRef) || wdq_fail 'missing base'
wdq_environment=$(git config --local --get datum.workflowDeliveryEnvironmentPath) || wdq_fail 'missing environment'
wdq_runner=$(git config --local --get datum.workflowDeliveryRunnerPath) || wdq_fail 'missing external runner'
[[ $wdq_authority =~ ^([a-f0-9]{40}|[a-f0-9]{64})$ ]] || wdq_fail 'authority must be a full commit ID'
[[ $wdq_base =~ ^([a-f0-9]{40}|[a-f0-9]{64})$ ]] || wdq_fail 'base must be a full commit ID'
[[ -n $wdq_environment ]] || wdq_fail 'empty environment'
[[ $wdq_runner = /* && -f $wdq_runner ]] || wdq_fail 'external runner must be an existing absolute path'
wdq_runner=$(realpath -- "$wdq_runner")
case "$wdq_runner" in "$wdq_repo"/*) wdq_fail 'runner cannot be inside the candidate worktree';; esac
[[ ${wdq_runner##*/} = check_workflow_delivery.py ]] || wdq_fail 'unexpected runner entry point'
python3 scripts/check_file_lane_ownership.py --staged
python3 scripts/check_rustfmt.py --staged
exec python3 "$wdq_runner" --root "$wdq_repo" --enforce --staged \
  --authority-ref "$wdq_authority" --base-ref "$wdq_base" \
  --environment-path "$wdq_environment"
