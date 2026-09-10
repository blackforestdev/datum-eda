"""Replay transferred real fixtures from pinned bundle and per-case raw state."""
import hashlib,json,os,re,shutil,subprocess,sys,tarfile,uuid
from pathlib import Path,PurePosixPath
ROOT=Path('/home/bfadmin/Documents/datum-eda')
STORE=Path(__file__).parent
RUNTIME=ROOT/'.git/datum-wdq/proposals/sequencing-repair-20260910'
PIN='1d48f249dc7071fc3718b345a4ab16b366af43be'
BUNDLE=ROOT/'.git/datum-wdq/proposals/sequencing-roadmap-377323b2/source.bundle'
BUNDLE_SHA='8b9f667fe2d8f406edf2e790060b7329393c2acc939723c69f6954ee4f072f6b'
batch=sys.argv[1]
assert len(sys.argv)==2 and batch in ('real-roadmap','current-inventory')
producer=ROOT/'.git/datum-wdq/proposals/sequencing-paired-evidence-20260910/owner-reopened'/batch
own=STORE/(batch+'-exact')
J=lambda p:json.loads(p.read_bytes())
H=lambda b:hashlib.sha256(b).hexdigest()
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
bootstrap=RUNTIME/'scripts/workflow_delivery_source_only.py'
ns={'__name__':'_review_bootstrap'};exec(compile(bootstrap.read_bytes(),str(bootstrap),'exec'),ns);ns['install'](bootstrap.parent)
sys.path.insert(0,str(bootstrap.parent))
from workflow_delivery_capture_command import capture_command
from workflow_delivery_capture_state import protected_state,git
assert J(producer.parent/(batch+'-process.json'))['exit_code']==0
assert git(RUNTIME,'rev-parse','HEAD').decode().strip()==PIN
assert H(BUNDLE.read_bytes())==BUNDLE_SHA
rows=J(producer/'observations.json')['invocations']
assert len(rows)==(15 if batch=='real-roadmap' else 9)
fixture=producer/'fixture'
assert (producer/'fixture-retired-after-producer').is_dir()
assert not fixture.exists() and not fixture.is_symlink(), 'custody collision'
own.mkdir()
seed=own/'bundle-seed'
environment={'PATH':os.defpath,'HOME':str(own),'XDG_CONFIG_HOME':str(own),'LANG':'C',
             'GIT_CONFIG_NOSYSTEM':'1','GIT_CONFIG_GLOBAL':os.devnull,'GIT_GRAFT_FILE':os.devnull}
def git_seed(*args):
    result=subprocess.run(['git','--no-replace-objects','--no-optional-locks','-c','core.hooksPath=/dev/null',*args],
        env=environment,capture_output=True)
    assert result.returncode==0,(args,result.stderr)
    return result.stdout
git_seed('clone','--no-checkout',str(BUNDLE),str(seed))
first=J(producer/rows[0]['capture_directory']/'prestate-archive.json')
git_seed('-C',str(seed),'checkout','--detach',first['head'])
assert not (seed/'.git/objects/info/alternates').exists()
save(own/'bundle-seed.json',{'bundle':str(BUNDLE),'sha256':BUNDLE_SHA,'head':first['head'],
    'tree':git_seed('-C',str(seed),'rev-parse','HEAD^{tree}').decode().strip(),
    'scope':'Private immutable bundle seed. Only Git object files are hardlinked into fresh restored fixtures; tracked worktree files are copied, never hardlinked.'})
results=[]
for row in rows:
    old=producer/row['capture_directory'];inv=J(old/'invocation.json');before=J(old/'before.json');expected=J(old/'result.json')
    descriptor=J(old/'prestate-archive.json');archive=Path(descriptor['archive'])
    assert Path(inv['cwd'])==fixture and Path(descriptor['original_root'])==fixture
    assert archive.parent==producer and H(archive.read_bytes())==descriptor['archive_sha256']
    assert descriptor['source_bundle_sha256']==BUNDLE_SHA and Path(descriptor['source_bundle'])==BUNDLE
    assert descriptor['head']==first['head'] and descriptor['fixture_nonce']==inv['fixture_nonce']
    assert not fixture.exists() and not fixture.is_symlink(), 'custody collision'
    out=own/row['capture_directory'];out.mkdir()
    fixture.mkdir(mode=0o700)
    # Copy immutable tracked bytes; never borrow a producer worktree or cache.
    for p in seed.iterdir():
        if p.name=='.git':continue
        destination=fixture/p.name
        if p.is_symlink():destination.symlink_to(os.readlink(p))
        elif p.is_dir():shutil.copytree(p,destination,symlinks=True)
        else:shutil.copy2(p,destination)
    (fixture/'.git').mkdir()
    shutil.copytree(seed/'.git/objects',fixture/'.git/objects',copy_function=os.link,symlinks=True)
    with tarfile.open(archive) as t:
        members=t.getmembers();names={}
        for m in members:
            n=PurePosixPath(m.name)
            assert not n.is_absolute() and '..' not in n.parts
            assert m.isdir() or m.isfile() or m.issym() or m.islnk(),m.name
            if str(n)=='.':continue
            assert str(n) not in names and not (str(n)=='.git/objects' or str(n).startswith('.git/objects/'))
            names[str(n)]=m
        for name,m in names.items():
            assert all(not (str(p) in names and (names[str(p)].issym() or names[str(p)].islnk())) for p in PurePosixPath(name).parents)
            assert all(not (fixture/str(p)).is_symlink() for p in PurePosixPath(name).parents)
        for name,m in names.items():
            p=fixture/name
            if m.isdir():
                assert not p.is_symlink();p.mkdir(parents=True,exist_ok=True)
            elif m.isfile():
                p.parent.mkdir(parents=True,exist_ok=True)
                with p.open('xb') as f:shutil.copyfileobj(t.extractfile(m),f)
                p.chmod(m.mode);os.utime(p,(m.mtime,m.mtime))
        for name,m in names.items():
            p=fixture/name
            if m.issym():p.symlink_to(m.linkname)
            elif m.islnk():
                link=PurePosixPath(m.linkname);assert not link.is_absolute() and '..' not in link.parts
                target=fixture/str(link);assert target.is_file() and not target.is_symlink();os.link(target,p)
        for name,m in names.items():
            if m.isdir():(fixture/name).chmod(m.mode)
    for name,mode in descriptor['filesystem_modes'].items():
        n=PurePosixPath(name);assert not n.is_absolute() and '..' not in n.parts
        p=fixture/name
        assert all(not (fixture/str(a)).is_symlink() for a in n.parents)
        if not p.is_symlink():p.chmod(mode)
    assert (fixture/'.git/datum-wdq-capture-owner').read_text().strip()==inv['fixture_nonce']
    actual=protected_state(fixture,before['untracked_roots'])
    save(out/'restored-before.json',actual)
    assert actual==before,(row['capture_directory'],'restored pre-state mismatch')
    nonce=str(uuid.uuid4());(fixture/'.git/datum-wdq-capture-owner').write_text(nonce+'\n')
    capture=out/'capture';relocate=lambda s:s.replace(str(old),str(capture))
    env={k:relocate(v) for k,v in inv['environment'].items()};env.update(HOME=str(own),XDG_CONFIG_HOME=str(own))
    save(out/'input-identity.json',{'producer_capture':str(old),'archive':descriptor,
        'exact_before_state_equal':True,'fresh_owner_nonce':True,
        'custody':'Producer terminal, original retained fixture moved recoverably by producer, absent original pathname atomically restored by reviewer. No cache/clock/claim regeneration.'})
    result=capture_command(fixture,capture,[relocate(v) for v in inv['argv']],env,
        fixture_nonce=nonce,case_id=inv['case_id'],surface=inv['surface'],untracked_roots=before['untracked_roots'],
        timeout=inv['timeout_seconds'],interrupt_on_call=inv['interrupt_on_call'])
    assert result['kind']==expected['kind'] and result['returncode']==expected['returncode'],(row,result)
    assert not result['changed_files'] and not result['changed_state_fields']
    oldtext=(old/'stdout.bin').read_bytes()+(old/'stderr.bin').read_bytes()
    text=(capture/'stdout.bin').read_bytes()+(capture/'stderr.bin').read_bytes()
    assert set(re.findall(rb'WDQ-[A-Z]+',text))==set(re.findall(rb'WDQ-[A-Z]+',oldtext)),(row,text)
    record=dict(row,invocation_id=result['invocation_id'],directory=out.name,
        result_sha256=H((capture/'result.json').read_bytes()),exact_before_state_equal=True,states_unchanged=True)
    save(out/'independent-assessment.json',record);results.append(record)
    assert fixture==producer/'fixture' and not fixture.is_symlink()
    assert (fixture/'.git/datum-wdq-capture-owner').read_text().strip()==nonce
    shutil.rmtree(fixture)
    save(out/'cleanup.json',{'path':str(fixture),'restored_owned_fixture_removed':not fixture.exists(),
        'recoverable_from':descriptor,'raw_capture_preserved':True})
    print(json.dumps({'completed':len(results),'batch':batch,'invocation_id':result['invocation_id']}),flush=True)
save(own/'observations.json',{'candidate':PIN,'observations':results,'reviewer_session':'wdq-independent-review-20260906',
    'scope':'Fresh exact same-path bundle/metadata/payload restoration; complete captured before-state equality required. No source/clock/cache regeneration.',
    'independent_recheck_complete':False,'activation_performed':False})
