#define _GNU_SOURCE
#include <link.h>
#include <X11/Xlib.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <time.h>
#include <sys/syscall.h>
static _Atomic(uintptr_t) open_fn, pending_fn, filter_fn, check_fn;
static _Atomic unsigned long sequence;
static void record(const char *kind, Display *display, XEvent *event, int result) {
 const char *base=getenv("PM045_X11_AUDIT_PATH"); if(!base)return;
 unsigned long seq=atomic_fetch_add(&sequence,1)+1;
 if(seq>1000000)_exit(121);
 struct timespec ts;clock_gettime(CLOCK_MONOTONIC,&ts);
 char path[1024],line[1024];
 int size=snprintf(path,sizeof(path),"%s.%d.jsonl",base,getpid());
 if(size<0 || (size_t)size>=sizeof(path))_exit(122);
 int fd=open(path,O_WRONLY|O_APPEND|O_CREAT|O_CLOEXEC,0600);if(fd<0)_exit(123);
 XClientMessageEvent *c=event && event->type==ClientMessage?&event->xclient:NULL;
 int n=snprintf(line,sizeof(line),"{\"seq\":%lu,\"ns\":%llu,\"pid\":%d,\"tid\":%ld,\"kind\":\"%s\",\"display\":\"%p\",\"result\":%d,\"event_type\":%d,\"window\":%lu,\"message_type\":%lu,\"data0\":%ld,\"data1\":%ld,\"serial\":%lu,\"send_event\":%d}\n",seq,(unsigned long long)ts.tv_sec*1000000000ull+ts.tv_nsec,getpid(),syscall(SYS_gettid),kind,(void*)display,result,event?event->type:0,event?event->xany.window:0,c?c->message_type:0,c?c->data.l[0]:0,c?c->data.l[1]:0,event?event->xany.serial:0,event?event->xany.send_event:0);
 if(n<0 || (size_t)n>=sizeof(line) || write(fd,line,n)!=n)_exit(124);
 close(fd);
}
static Display *observe_open(const char *name) {
 Display *d=((Display *(*)(const char *))atomic_load(&open_fn))(name);record("open_display",d,NULL,d!=NULL);return d;
}
static int observe_pending(Display *d) {
 int n=((int(*)(Display*))atomic_load(&pending_fn))(d);record("pending",d,NULL,n);return n;
}
static Bool observe_filter(XEvent *e,Window w) {
 record("filter_before",e->xany.display,e,0);
 Bool r=((Bool(*)(XEvent*,Window))atomic_load(&filter_fn))(e,w);
 record("filter_after",e->xany.display,e,r);
 return r;
}
static Bool observe_check(Display *d,XEvent *e,Bool(*predicate)(Display*,XEvent*,XPointer),XPointer arg) {
 Bool r=((Bool(*)(Display*,XEvent*,Bool(*)(Display*,XEvent*,XPointer),XPointer))atomic_load(&check_fn))(d,e,predicate,arg);
 if(r)record("check_event",d,e,r);
 return r;
}
unsigned int la_version(unsigned int v) {return v<LAV_CURRENT?v:LAV_CURRENT;}
unsigned int la_objopen(struct link_map *m,Lmid_t l,uintptr_t *cookie) {(void)m;(void)l;(void)cookie;return LA_FLG_BINDTO|LA_FLG_BINDFROM;}
uintptr_t la_symbind64(Elf64_Sym *sym,unsigned int ndx,uintptr_t *ref,uintptr_t *def,unsigned int *flags,const char *name) {
 (void)ndx;(void)ref;(void)def;(void)flags;
 if(strcmp(name,"XOpenDisplay"))return sym->st_value;
 _Atomic(uintptr_t) *slot=NULL;uintptr_t replacement=0;
 if(!strcmp(name,"XOpenDisplay")){slot=&open_fn;replacement=(uintptr_t)observe_open;}
 else if(!strcmp(name,"XPending")){slot=&pending_fn;replacement=(uintptr_t)observe_pending;}
 else if(!strcmp(name,"XFilterEvent")){slot=&filter_fn;replacement=(uintptr_t)observe_filter;}
 else if(!strcmp(name,"XCheckIfEvent")){slot=&check_fn;replacement=(uintptr_t)observe_check;}
 if(slot){uintptr_t previous=atomic_exchange(slot,sym->st_value);if(previous && previous!=sym->st_value)_exit(125);return replacement;}
 return sym->st_value;
}
