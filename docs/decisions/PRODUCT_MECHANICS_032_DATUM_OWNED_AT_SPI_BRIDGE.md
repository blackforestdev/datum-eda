# Product Mechanics 032: Datum-Owned AT-SPI Accessibility Bridge

Status: ratified doctrine

## Decision

Datum will publish its existing immutable terminal accessibility semantics to
Linux assistive technology through a Datum-authored, standard-library-only
AT-SPI bridge over the documented D-Bus protocol.

The bridge may use Rust's standard library, existing Datum workspace substrate,
Unix-domain sockets, the desktop session bus, and the operating-system
accessibility bus. It may not add AccessKit, `libatspi`, a D-Bus crate,
generated third-party bindings, copied implementation source, a build download,
or a runtime helper subprocess. `TerminalCore` remains the sole terminal
semantic authority; the bridge only marshals its bounded immutable projection
to operating-system accessibility interfaces.

This decision supersedes the rejected AccessKit proposal. Authoritative package
metadata established that `accesskit_winit` 0.33.2 is Apache-2.0-only, and the
owner determined that Apache-only obligations are incompatible with Datum's
intended commercial licensing structure under Product Mechanics 029 DA-008.

## Required Surface

- Connect and authenticate to the standard Linux D-Bus accessibility bus using
  the documented local Unix transport and bounded protocol state.
- Register one Datum application root and terminal accessible object with the
  AT-SPI registry.
- Implement the bounded AT-SPI Application, Accessible, Component, Text, and
  Hypertext behavior required for terminal name, text, caret, selection,
  focus, geometry, and hyperlinks.
- Publish bounded focus, caret, selection, text-change, title/name, and
  lifecycle events derived from immutable TerminalCore accessibility updates.
- Fail closed and keep Datum operational when accessibility is disabled, the
  bus is absent, authentication fails, a peer sends malformed data, or the
  bridge disconnects.
- Keep all D-Bus parsing, marshalling, object dispatch, AT-SPI mapping, and
  lifecycle ownership in cohesive Datum-owned modules subject to decision 022.

## Proof Boundary

DTC-P24B must include deterministic D-Bus authentication, marshalling,
alignment, signature, reply-correlation, malformed-message, size-limit,
disconnect, AT-SPI object, text-offset, caret, selection, focus, hyperlink, and
event tests. Production evidence must show Datum registering on a real Linux
accessibility bus and a real screen reader observing terminal content and state.
No dependency, license exception, terminal semantic fork, or external helper is
authorized by that proof.

## Primary Standards

- D-Bus Specification: <https://dbus.freedesktop.org/doc/dbus-specification.html>
- AT-SPI2 interface definitions:
  <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/xml-interfaces.html>
- AT-SPI Accessible interface:
  <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Accessible.html>
- AT-SPI Text interface:
  <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Text.html>

## Consequences

Datum accepts the engineering and maintenance cost of owning the small Linux
accessibility protocol boundary. The product keeps commercial licensing and
supply-chain control while retaining the full screen-reader requirement. The
decision does not reduce accessibility acceptance or authorize a private
protocol in place of AT-SPI.
