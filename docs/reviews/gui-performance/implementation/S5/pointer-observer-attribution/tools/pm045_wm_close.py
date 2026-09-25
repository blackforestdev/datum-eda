"""Request native X11 client closure using the existing system Xlib ABI."""
import ctypes as c

class Data(c.Union):
    _fields_=[('b',c.c_char*20),('s',c.c_short*10),('l',c.c_long*5)]
class Client(c.Structure):
    _fields_=[('type',c.c_int),('serial',c.c_ulong),('send_event',c.c_int),('display',c.c_void_p),('window',c.c_ulong),('message_type',c.c_ulong),('format',c.c_int),('data',Data)]
class Event(c.Union):
    _fields_=[('client',Client),('pad',c.c_long*24)]

def close_window(window):
    x=c.CDLL('libX11.so.6')
    x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=c.c_void_p
    x.XInternAtom.argtypes=[c.c_void_p,c.c_char_p,c.c_int];x.XInternAtom.restype=c.c_ulong
    x.XSendEvent.argtypes=[c.c_void_p,c.c_ulong,c.c_int,c.c_long,c.POINTER(Event)];x.XSendEvent.restype=c.c_int
    x.XFlush.argtypes=[c.c_void_p];x.XCloseDisplay.argtypes=[c.c_void_p]
    display=x.XOpenDisplay(None);assert display,'X display unavailable'
    try:
        event=Event();event.client.type=33;event.client.display=display;event.client.window=window
        event.client.message_type=x.XInternAtom(display,b'WM_PROTOCOLS',0);event.client.format=32
        event.client.data.l[0]=x.XInternAtom(display,b'WM_DELETE_WINDOW',0)
        assert x.XSendEvent(display,window,0,0,c.byref(event))!=0
        x.XFlush(display)
    finally:x.XCloseDisplay(display)
