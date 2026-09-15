"""Normal WM_DELETE_WINDOW close using installed Xlib, not XDestroyWindow.

The ABI follows the installed X11/Xlib.h XClientMessageEvent/XEvent definitions.
This is proof tooling only; Datum's input/runtime dependencies are unchanged.
"""

import ctypes as c


class Data(c.Union):
    _fields_ = [("b", c.c_char * 20), ("s", c.c_short * 10), ("l", c.c_long * 5)]


class Client(c.Structure):
    _fields_ = [("type", c.c_int), ("serial", c.c_ulong), ("send_event", c.c_int),
                ("display", c.c_void_p), ("window", c.c_ulong),
                ("message_type", c.c_ulong), ("format", c.c_int), ("data", Data)]


class Event(c.Union):
    _fields_ = [("client", Client), ("pad", c.c_long * 24)]


def close_window(display_name, window):
    lib = c.CDLL("libX11.so.6")
    lib.XOpenDisplay.argtypes, lib.XOpenDisplay.restype = [c.c_char_p], c.c_void_p
    lib.XInternAtom.argtypes = [c.c_void_p, c.c_char_p, c.c_int]
    lib.XInternAtom.restype = c.c_ulong
    lib.XSendEvent.argtypes = [c.c_void_p, c.c_ulong, c.c_int, c.c_long, c.POINTER(Event)]
    lib.XCloseDisplay.argtypes = [c.c_void_p]
    lib.XFlush.argtypes = [c.c_void_p]
    display = lib.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError("private X display unavailable")
    try:
        event = Event()
        event.client.type = 33  # ClientMessage, X11/X.h
        event.client.send_event = 1
        event.client.display = display
        event.client.window = int(window)
        event.client.message_type = lib.XInternAtom(display, b"WM_PROTOCOLS", 0)
        event.client.format = 32
        event.client.data.l[0] = lib.XInternAtom(display, b"WM_DELETE_WINDOW", 0)
        event.client.data.l[1] = 0  # CurrentTime
        if not lib.XSendEvent(display, int(window), 0, 0, c.byref(event)):
            raise RuntimeError("WM_DELETE_WINDOW send failed")
        lib.XFlush(display)
    finally:
        lib.XCloseDisplay(display)
