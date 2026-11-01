"""Read-only native log readiness; application receipts are not displayed pixels."""
import json
import re


def complete_records(text):
    """Keep byte positions while refusing an unfinished trailing log record."""
    position = 0
    for number, line in enumerate(text.splitlines(keepends=True), 1):
        end = position + len(line.encode('utf-8'))
        if not line.endswith('\n'):
            break
        yield {'line': number, 'start_byte': position, 'end_byte': end}, line
        position = end


def structured_records(records, marker):
    return [(position, json.loads(line.split(marker, 1)[1]))
            for position, line in records if marker in line]


def selected_presentation(records, host, requirement, ledger_position):
    """Correlate one released subject's immutable frame to its native return."""
    events = [position for position, line in records
              if position['start_byte'] >= requirement['after_byte']
              and f"window event {host} {requirement['event']}" in line]
    if len(events) != 1:
        return None
    event = events[0]
    contents = [(position, content) for position, content in
                structured_records(records, 'native submitted content ')
                if position['start_byte'] >= event['end_byte']
                and content['native_identity']['window'] == host]
    if not contents:
        return None
    content_position, content = contents[-1]
    selection = content['current_selection']
    if any(selection.get(key) != value
           for key, value in requirement['selection_fields'].items()):
        return None
    for region, expected in requirement['text_by_region'].items():
        actual = {row['text'] for row in content['cpu_host_text']
                  if row['region'] == region}
        if not set(expected).issubset(actual):
            return None
    identity = content['native_identity']
    lifecycles = [(position, value) for position, value in
                  structured_records(records, 'native_surface_lifecycle ')
                  if position['start_byte'] >= content_position['end_byte']
                  and value['window'] == host]
    matches = [(position, value) for position, value in lifecycles
               if value['reason'] == 'present'
               and value['host'] == identity['host']
               and value['queue_epoch'] == identity['queue_epoch']
               and value['configuration_generation'] == identity['configuration']
               and value['acquired'] == identity['acquisition']
               and value['last_frame_submission_receipt'] == identity['submission_receipt']
               and value['last_present_receipt'] == identity['submission_receipt']
               and value['active'] == 0]
    if len(matches) != 1 or ledger_position['start_byte'] < matches[0][0]['end_byte']:
        return None
    latest = lifecycles[-1][1]
    if (latest['active'] != 0 or latest['host'] != identity['host']
            or latest['queue_epoch'] != identity['queue_epoch']
            or latest['configuration_generation'] != identity['configuration']
            or latest['acquired'] != identity['acquisition']):
        return None
    return {'event': event, 'submitted_content': content_position,
            'native_present': matches[0][0], 'settled_ledger': ledger_position,
            'native_identity': identity, 'content_revision': content['content_revision'],
            'limits': 'Ordered application content/present/ledger receipts only; '
                      'not GPU pixels, compositor sampling or visible-output acceptance.'}


class PresentationReadiness:
    """Own host/focus/settled-frame admission and optional content correlation."""

    def __init__(self, after_damage=None, *, require_stable=True, frame_requirement=None,
                 target_window=None):
        self.after_damage = after_damage
        self.require_stable = require_stable
        self.frame_requirement = frame_requirement
        self.target_window = target_window
        self.settled = None

    def __call__(self, text, *, owned_dialog=False):
        if self.frame_requirement is not None and not text.endswith('\n'):
            return self.not_ready()
        try:
            return self.current_frame(list(complete_records(text)), owned_dialog)
        except (KeyError, TypeError, ValueError, IndexError):
            self.settled = None
            return False

    def current_frame(self, records, owned_dialog):
        text = ''.join(line for _, line in records)
        identities = re.findall(r'surface identity window=(WindowId\([0-9]+\))', text)
        if not identities:
            return self.not_ready()
        host = self.target_window or (identities[-1] if owned_dialog else identities[0])
        if host not in identities:
            return self.not_ready()
        focus = re.findall(r'window event (WindowId\([0-9]+\)) focused (true|false)', text)
        if not focus or focus[-1] != (host, 'true'):
            return self.not_ready()
        ledgers = [(position, value) for position, value in
                   structured_records(records, 'native redraw ledger ')
                   if value['window'] == host]
        if not ledgers:
            return self.not_ready()
        position, latest = ledgers[-1]
        if (latest['damage'] != latest['presented_damage'] or latest['pending']
                or latest['rendering'] is not None
                or (self.after_damage is not None
                    and latest['presented_damage'] <= self.after_damage)):
            return self.not_ready()
        identity = (host, latest['presented_damage'])
        if self.frame_requirement is not None:
            binding = selected_presentation(records, host, self.frame_requirement, position)
            if binding is None:
                return self.not_ready()
            latest = dict(latest, content_presentation=binding)
            identity += tuple(sorted(binding['native_identity'].items()))
        ready = not self.require_stable or self.settled == identity
        self.settled = identity
        return latest if ready else False

    def not_ready(self):
        self.settled = None
        return False


def matched_presentation(text, expectation):
    """Bind the sole newly opened dialog to a post-action native present.

    These are application telemetry observations, never proof of GPU correctness.
    The caller must compare this binding around the compositor pixel reads.
    """
    if not text.endswith('\n'):
        return None
    target = expectation['window']
    receipt = PresentationReadiness(require_stable=False, target_window=target)(text)
    if not receipt:
        return None
    try:
        records = list(complete_records(text))
        opened = [match.group(1) for position, line in records
                  if position['start_byte'] >= expectation['opened_after_byte']
                  for match in [re.search(r'surface identity window=(WindowId\([0-9]+\))', line)]
                  if match]
        if opened != [target]:
            return None
        lifecycles = [(position, value) for position, value in
                      structured_records(records, 'native_surface_lifecycle ')
                      if value['window'] == target]
        presents = [(position, value) for position, value in lifecycles
                    if value['reason'] == 'present'
                    and position['start_byte'] >= expectation['after_byte']]
        if not presents:
            return None
        position, presented = presents[-1]
        latest = lifecycles[-1][1]
        keys = ('window', 'host', 'queue_epoch', 'configuration_generation',
                'configured_extent', 'acquired', 'last_present_receipt')
        identity = {key: presented[key] for key in keys}
        if (presented['active'] != 0 or latest['active'] != 0
                or presented['last_present_receipt'] != presented['last_frame_submission_receipt']
                or presented['last_present_receipt'] <= 0
                or identity['configured_extent'] != expectation['extent']
                or any(latest[key] != value for key, value in identity.items())):
            return None
        ledgers = [(where, value) for where, value in
                   structured_records(records, 'native redraw ledger ')
                   if value['window'] == target]
        ledger_position = ledgers[-1][0]
        if ledger_position['start_byte'] < position['end_byte']:
            return None
        return {'native_identity': identity, 'native_present': position,
                'settled_ledger': ledger_position, 'ledger': receipt}
    except (KeyError, TypeError, ValueError, IndexError):
        return None
