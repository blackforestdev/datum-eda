"""Verify real I04 captures independently of their prerequisite typed packet."""
from pathlib import Path
import sys

ROOT=Path('/home/bfadmin/Documents/datum-eda')
template=ROOT/'.git/datum-wdq/proposals/independent-sequencing-20260910/verify-workspace-exact.py'
source=template.read_text()
changes={
    'sequencing-paired-evidence-20260910':'terminal-integration-1644',
    'independent-sequencing-20260910':'independent-terminal-rollout-20260910',
    'sequencing-repair-20260910':'terminal-owner-20260910',
    '1d48f249dc7071fc3718b345a4ab16b366af43be':'9924678c39fe5d176e58ebfd8e25ab6feb213f35',
    "'renewal-result.json'":"'observations.json'",
}
for old,new in changes.items():
    assert old in source
    source=source.replace(old,new)
source=source.replace("'candidate':pin,'stats':stats", "'candidate':pin,'integration_candidate':'1644e2129f4ff83cec1cdf0e6e13534c3b258e8e','stats':stats")
source=source.replace("Not full matrix, renewed typed packet, RECHECK completion, approval or activation.", "Separate exact I04 integration only; prerequisite typed packet unchanged. Not installed verification, product acceptance or activation.")
assert sys.argv[1] in ('real-roadmap','current-inventory') and len(sys.argv)==2
exec(compile(source,str(template)+'[terminal-I04-integration]','exec'))
