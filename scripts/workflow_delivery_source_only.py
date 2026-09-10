"""Read repository Python source without consuming adjacent bytecode caches.

Entrypoints must execute these source bytes before their first repository import;
an ordinary import of this bootstrap could itself consume an unverified cache.
This is cache isolation, not source authorization or an ignored-file exemption.
"""

from importlib.machinery import PathFinder, SourceFileLoader, SourcelessFileLoader
import os
import sys


class SourceOnlyLoader(SourceFileLoader):
    def get_code(self, fullname):
        # Do not call the parent get_code: it reads .pyc even with Python -B.
        return self.source_to_code(self.get_data(self.path), self.path)


class RepositorySourceFinder:
    def __init__(self, root):
        self.root = os.path.abspath(os.fspath(root))

    def find_spec(self, fullname, path=None, target=None):
        spec = PathFinder.find_spec(fullname, path, target)
        if spec is None or not isinstance(spec.origin, str):
            return None
        origin = os.path.abspath(spec.origin)
        if os.path.commonpath((self.root, origin)) != self.root:
            return None
        if isinstance(spec.loader, SourcelessFileLoader):
            raise ImportError("WDQ source-only runtime refuses sourceless repository bytecode: " + origin)
        if isinstance(spec.loader, SourceFileLoader):
            spec.loader = SourceOnlyLoader(fullname, spec.origin)
            return spec
        return None


def install(root):
    """Protect future imports below one explicit directory; never alter files."""
    finder = RepositorySourceFinder(root)
    sys.meta_path.insert(0, finder)
    return finder
