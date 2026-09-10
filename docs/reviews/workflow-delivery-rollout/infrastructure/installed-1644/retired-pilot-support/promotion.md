# Owner-only activation handoff

Preparation succeeded; promotion has NOT been performed. This local candidate
reference retains an object, not authority. Your existing ACCEPT and two DEFER
responses remain valid. This transaction does not authorize Preferences GP-CM05.

Runnable version: after review, run `python3 /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/activate.py --activate` in your terminal.
It executes exactly the activation block below and saves combined output to a
private activation-*.log beside the script. Without --activate it prints help
and changes nothing. Do not paste the Markdown into a terminal.

Candidate: `4ccac933e9ca2b049ea68daf50de16fafe375fd8`
Comparison base: `76152fc7dab4ba0431da991f58d6a5ebc1838763`
Packet: `ab71468a0acd0e845f5e7cd5e2b21aae5121a85dfd5c2d72fa2f5f372c5745a0`
Review: `9a90b1703fa2041c8ccd0d4584c8b856c1d64e120318da8d754716a998ba7443`

Review first: inspect the six-file proposal and the external hook. The only
predecessor change removes this pilot from its remaining-unblocks lists. Other
work must stay paused during publication. Do not use an older preparation if
HEAD moved; regenerate against a fresh coordinated base instead of forcing it.

```bash
git -C /home/bfadmin/Documents/datum-eda diff 76152fc7dab4ba0431da991f58d6a5ebc1838763 4ccac933e9ca2b049ea68daf50de16fafe375fd8 --stat
git -C /home/bfadmin/Documents/datum-eda diff 76152fc7dab4ba0431da991f58d6a5ebc1838763 4ccac933e9ca2b049ea68daf50de16fafe375fd8
```

Only the project owner may select this candidate as trusted authority and run
the following block after review. The first validator invocation is read-only.
Publication/configuration happen only if it passes. Any failure stops the block;
do not bypass a refusal or infer that a partially completed block activated the
gate. Keep this external directory available: the hook depends on its pinned
runner. Do not modify that runner or move/delete it after installation.

```bash
(
set -euo pipefail
trap 'echo "WDQ activation stopped at line $LINENO (exit $?). No later commands ran; report this output." >&2' ERR
cd /home/bfadmin/Documents/datum-eda
echo 'WDQ: checking the pinned base and clean worktree; a mismatch means this block is obsolete.'
test "$(git rev-parse HEAD)" = 76152fc7dab4ba0431da991f58d6a5ebc1838763
test -z "$(git status --porcelain)"
echo 'WDQ: validating the exact candidate before publication.'
python3 /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/candidate/scripts/check_workflow_delivery.py --root /home/bfadmin/Documents/datum-eda --enforce --authority-ref 4ccac933e9ca2b049ea68daf50de16fafe375fd8 --base-ref 76152fc7dab4ba0431da991f58d6a5ebc1838763 --environment-path docs/reviews/workflow-delivery-pilot/proof-artifacts/environment.json --candidate-ref 4ccac933e9ca2b049ea68daf50de16fafe375fd8
test "$(git rev-parse HEAD)" = 76152fc7dab4ba0431da991f58d6a5ebc1838763
test -z "$(git status --porcelain)"
echo 'WDQ: publishing the verified candidate and installing owner-selected local trust.'
git merge --ff-only 4ccac933e9ca2b049ea68daf50de16fafe375fd8
git config --local datum.workflowDeliveryAuthorityRef 4ccac933e9ca2b049ea68daf50de16fafe375fd8
git config --local datum.workflowDeliveryBaseRef 76152fc7dab4ba0431da991f58d6a5ebc1838763
git config --local datum.workflowDeliveryEnvironmentPath docs/reviews/workflow-delivery-pilot/proof-artifacts/environment.json
git config --local datum.workflowDeliveryRunnerPath /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/candidate/scripts/check_workflow_delivery.py
git config --local core.hooksPath /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/owner-hooks
python3 /home/bfadmin/Documents/datum-wdq-owner-activation-20260907/candidate/scripts/check_workflow_delivery.py --root /home/bfadmin/Documents/datum-eda --enforce --authority-ref 4ccac933e9ca2b049ea68daf50de16fafe375fd8 --base-ref 76152fc7dab4ba0431da991f58d6a5ebc1838763 --environment-path docs/reviews/workflow-delivery-pilot/proof-artifacts/environment.json --staged
/home/bfadmin/Documents/datum-wdq-owner-activation-20260907/owner-hooks/pre-commit
python3 scripts/project_status.py check
echo 'WDQ: owner activation and blocking verification completed successfully.'
)
```

Return the candidate/base IDs and the complete verification output, including
any failure. G06 is not reported as operationally complete until publication,
configuration and blocking verification all succeed. No further enrollment,
product acceptance, or successor selection is implied. Earlier incomplete status
notes in the handoff are historical only after this transaction succeeds.
