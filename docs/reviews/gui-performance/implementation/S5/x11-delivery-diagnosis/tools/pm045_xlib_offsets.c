#include <X11/Xlibint.h>
#include <stddef.h>
#include <stdio.h>
int main(void){printf("{\"qlen\":%zu,\"head\":%zu,\"qevent_event\":%zu,\"qevent_next\":%zu,\"event_type\":%zu,\"event_window\":%zu,\"event_message_type\":%zu,\"event_data\":%zu}\n",offsetof(Display,qlen),offsetof(Display,head),offsetof(_XQEvent,event),offsetof(_XQEvent,next),offsetof(XEvent,xclient.type),offsetof(XEvent,xclient.window),offsetof(XEvent,xclient.message_type),offsetof(XEvent,xclient.data));}
