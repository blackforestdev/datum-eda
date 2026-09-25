#include <X11/Xlib.h>
#include <assert.h>
#include <stdio.h>
static Bool accept(Display*d,XEvent*e,XPointer p){(void)d;(void)e;(void)p;return True;}
int main(void){Display*d=XOpenDisplay(NULL);assert(d);Window w=XCreateSimpleWindow(d,DefaultRootWindow(d),0,0,20,20,0,0,0);XEvent e={0};e.xclient.type=ClientMessage;e.xclient.display=d;e.xclient.window=w;e.xclient.message_type=XInternAtom(d,"WM_PROTOCOLS",False);e.xclient.format=32;e.xclient.data.l[0]=XInternAtom(d,"WM_DELETE_WINDOW",False);assert(XSendEvent(d,w,False,0,&e));XSync(d,False);assert(XPending(d)>0);assert(XCheckIfEvent(d,&e,accept,NULL));assert(e.type==ClientMessage);assert(!XFilterEvent(&e,w));printf("window=%lu protocols=%lu delete=%ld\n",w,e.xclient.message_type,e.xclient.data.l[0]);XDestroyWindow(d,w);XCloseDisplay(d);return 0;}
