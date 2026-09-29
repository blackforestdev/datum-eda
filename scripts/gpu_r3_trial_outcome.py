"""Keep the first trial failure authoritative while collecting shutdown evidence."""


def record_failure(report, error):
    failures = report.setdefault('failures', [])
    detail = repr(error)
    if detail not in failures:
        failures.append(detail)
    report['status'] = 'invalid'
    report['error'] = failures[0]


def finish_result(report, budget_stop):
    report['budget_stop'] = budget_stop
    report['status'] = ('invalid' if report.get('failures') else
                        'valid_budget_failure' if budget_stop else 'valid_descriptive_run')
