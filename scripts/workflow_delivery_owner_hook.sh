#!/usr/bin/env bash
# Prepared owner-hook candidate. Installation requires the separate owner packet.
set -euo pipefail
wdq_fail() { echo "WDQ-TRUST: $*" >&2; exit 2; }
wdq_git() { env -i PATH="$PATH" LC_ALL=C GIT_GRAFT_FILE=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null git --no-replace-objects --no-optional-locks "$@"; }
wdq_repo=$(wdq_git rev-parse --show-toplevel) || wdq_fail 'repository unavailable'
wdq_repo=$(realpath -- "$wdq_repo")
cd "$wdq_repo"
wdq_gitdir=$(wdq_git rev-parse --absolute-git-dir)
[[ -z ${GIT_DIR-} || $(realpath -- "$GIT_DIR") = "$wdq_gitdir" ]] || wdq_fail 'Git directory differs from worktree'
[[ -z ${GIT_WORK_TREE-} || $(realpath -- "$GIT_WORK_TREE") = "$wdq_repo" ]] || wdq_fail 'Git worktree differs from invocation'
[[ -z ${GIT_NAMESPACE-} ]] || wdq_fail 'namespaced Git invocation is not the selected repository'
wdq_authority=$(wdq_git config --local --get datum.workflowDeliveryAuthorityRef) || wdq_fail 'missing authority'
wdq_base=$(wdq_git config --local --get datum.workflowDeliveryBaseRef) || wdq_fail 'missing base'
wdq_environment=$(wdq_git config --local --get datum.workflowDeliveryEnvironmentPath) || wdq_fail 'missing environment'
wdq_runner=$(wdq_git config --local --get datum.workflowDeliveryRunnerPath) || wdq_fail 'missing runner'
[[ $wdq_authority =~ ^([a-f0-9]{40}|[a-f0-9]{64})$ ]] || wdq_fail 'authority must be a full commit ID'
[[ $wdq_base =~ ^([a-f0-9]{40}|[a-f0-9]{64})$ ]] || wdq_fail 'base must be a full commit ID'
[[ -n $wdq_environment ]] || wdq_fail 'empty environment'
wdq_common=$(wdq_git rev-parse --path-format=absolute --git-common-dir)
wdq_common=$(realpath -- "$wdq_common")
[[ -z ${GIT_COMMON_DIR-} || $(realpath -- "$GIT_COMMON_DIR") = "$wdq_common" ]] || wdq_fail 'Git common directory differs from worktree'
wdq_store="$wdq_common/datum-wdq/trusted/$wdq_authority"
[[ $(realpath -- "$wdq_store") = "$wdq_store" ]] || wdq_fail 'redirected support store'
wdq_hook="$wdq_store/owner-hooks/pre-commit"
[[ ! -L ${BASH_SOURCE[0]} && $(realpath -- "${BASH_SOURCE[0]}") = "$wdq_hook" ]] || wdq_fail 'unexpected hook location'
[[ $wdq_runner = "$wdq_store/scripts/check_workflow_delivery.py" ]] || wdq_fail 'unexpected runner location'
wdq_verify() {
    local source=$1 target=$2 expected actual
    [[ -f $target && ! -L $target ]] || wdq_fail "missing or redirected $source"
    expected=$(wdq_git rev-parse --verify "$wdq_authority:$source") || wdq_fail "unpinned $source"
    actual=$(wdq_git hash-object --no-filters -- "$target") || wdq_fail "unreadable $source"
    [[ $actual = "$expected" ]] || wdq_fail "changed pinned $source"
}
wdq_verify scripts/workflow_delivery_owner_hook.sh "$wdq_hook"
wdq_bootstrap="$wdq_store/scripts/workflow_delivery_bootstrap.py"
wdq_verify scripts/workflow_delivery_bootstrap.py "$wdq_bootstrap"
wdq_arguments=(--root "$wdq_repo" --authority-ref "$wdq_authority" --base-ref "$wdq_base" --environment-path "$wdq_environment" --runner "$wdq_runner")
if [[ ${DATUM_WDQ_OBSERVE_PATH+x} ]]; then
    wdq_trace=$DATUM_WDQ_OBSERVE_PATH
    [[ $wdq_trace = /* && $(realpath -- "$wdq_trace") = "$wdq_trace" ]] || wdq_fail 'explicit canonical observation path required'
    [[ ! -e $wdq_trace && ! -L $wdq_trace ]] || wdq_fail 'observation output already exists'
    [[ $wdq_trace != "$wdq_repo/"* || $wdq_trace = "$wdq_common/datum-wdq/logs/"* ]] || wdq_fail 'observation cannot write worktree inputs'
    [[ $wdq_trace != "$wdq_common/"* || $wdq_trace = "$wdq_common/datum-wdq/logs/"* ]] || wdq_fail 'observation cannot write Git metadata outside WDQ logs'
    wdq_observer="$wdq_store/scripts/workflow_delivery_observe_python.py"
    wdq_verify scripts/workflow_delivery_observe_python.py "$wdq_observer"
    # No bundle directory enters sys.path until the authenticated bootstrap has
    # verified the complete bundle. Observation does not disable enforcement.
    exec python3 -I -S -B "$wdq_observer" --script "$wdq_bootstrap" --source-root "$wdq_store" \
      --output "$wdq_trace" --function main --function _validate_delivery --function load_environments \
      --no-script-path -- "${wdq_arguments[@]}"
fi
exec python3 -I -S -B "$wdq_bootstrap" "${wdq_arguments[@]}"
