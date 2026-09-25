#define _GNU_SOURCE
/* First-party Linux x86_64 descriptor observer for OWNED children only.
 * Kernel seccomp/ptrace events include direct and libc-hidden syscalls.
 * Diagnostic timing perturbation must be measured; not a product dependency. */
#include <sys/ptrace.h>
#include <linux/ptrace.h>
#include <linux/audit.h>
#include <linux/filter.h>
#include <linux/seccomp.h>
#include <sys/prctl.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#include <sys/types.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

static FILE *output;
static unsigned long sequence;
struct thread { pid_t tid; int pending; uint64_t number, args[6]; unsigned long sequence; };
static struct thread threads[1024];
static void fail(const char *why) { fprintf(stderr,"fd-trace fatal: %s: %s\n",why,strerror(errno)); exit(124); }
static uint64_t now(void) { struct timespec t; if(clock_gettime(CLOCK_MONOTONIC,&t))fail("clock");return(uint64_t)t.tv_sec*1000000000+t.tv_nsec; }
static struct thread *thread(pid_t tid) {
    for(size_t i=0;i<1024;i++)if(threads[i].tid==tid)return &threads[i];
    for(size_t i=0;i<1024;i++)if(!threads[i].tid){threads[i]=(struct thread){.tid=tid};return &threads[i];}
    fail("thread bound exceeded");return NULL;
}
static long read_file(const char *path,char *data,size_t capacity) {
    int fd=open(path,O_RDONLY|O_CLOEXEC);if(fd<0)return -1;
    ssize_t n=read(fd,data,capacity-1);int saved=errno;close(fd);errno=saved;
    if(n>=0)data[n]=0;
    return n;
}
static void identity(pid_t tid,pid_t *tgid,unsigned long long *start) {
    char path[64],bytes[4096];snprintf(path,sizeof(path),"/proc/%d/status",tid);
    if(read_file(path,bytes,sizeof(bytes))<0)fail("process status");
    char *key=strstr(bytes,"Tgid:");if(!key)fail("process TGID");*tgid=atoi(key+5);
    snprintf(path,sizeof(path),"/proc/%d/stat",*tgid);
    if(read_file(path,bytes,sizeof(bytes))<0)fail("process stat");
    char *p=strrchr(bytes,')');if(!p)fail("process stat parse");p+=2;
    for(int field=3;field<22;field++){p=strchr(p,' ');if(!p)fail("process start field");p++;}
    *start=strtoull(p,NULL,10);
}
static void flush(void) { if(fflush(output)||ferror(output))fail("ledger write"); }
static void descriptor(struct thread *t,const char *phase,int fd) {
    char path[80],raw[4096];snprintf(path,sizeof(path),"/proc/%d/fdinfo/%d",t->tid,fd);
    long n=read_file(path,raw,sizeof(raw));
    if(n<0){
        /* EBADF close is legitimate. Preserve every missing lookup for audit. */
        fprintf(output,"{\"kind\":\"lookup_missing\",\"tid\":%d,\"fd\":%d,\"sequence\":%lu,\"phase\":\"%s\",\"errno\":%d}\n",t->tid,fd,t->sequence,phase,errno);flush();return;
    }
    if(!strstr(raw,"drm-client-id:"))return;
    if(n==(long)sizeof(raw)-1)fail("fdinfo truncated");
    pid_t pid;unsigned long long start;identity(t->tid,&pid,&start);
    fprintf(output,"{\"kind\":\"drm\",\"phase\":\"%s\",\"pid\":%d,\"start\":%llu,\"tid\":%d,\"fd\":%d,\"sequence\":%lu,\"syscall\":%llu,\"monotonic_ns\":%llu,\"raw_hex\":\"",phase,pid,start,t->tid,fd,t->sequence,(unsigned long long)t->number,(unsigned long long)now());
    for(long i=0;i<n;i++)fprintf(output,"%02x",(unsigned char)raw[i]);
    fprintf(output,"\"}\n");flush();
}
static void descriptor_set(struct thread *t,const char *phase,unsigned first,unsigned last,int cloexec_only) {
    char path[80];snprintf(path,sizeof(path),"/proc/%d/fdinfo",t->tid);
    DIR *dir=opendir(path);if(!dir)fail("fd inventory");struct dirent *entry;unsigned count=0;
    while((entry=readdir(dir))) {
        char *end;unsigned long fd=strtoul(entry->d_name,&end,10);
        if(*end||fd<first||fd>last)continue;
        if(++count>65536)fail("fd inventory bound");
        if(cloexec_only){
            char raw[4096];snprintf(path,sizeof(path),"/proc/%d/fdinfo/%lu",t->tid,fd);
            if(read_file(path,raw,sizeof(raw))<0)fail("exec fd reading");
            char *flags=strstr(raw,"flags:");if(!flags)fail("fd flags");
            if(!(strtoul(flags+6,NULL,8)&O_CLOEXEC))continue;
        }
        descriptor(t,phase,(int)fd);
    }
    closedir(dir);
}
static void before(struct thread *t) {
    switch(t->number) {
    case SYS_close:descriptor(t,"retire_before",(int)t->args[0]);break;
    case SYS_dup2:case SYS_dup3:
        if(t->args[0]!=t->args[1])descriptor(t,"replace_before",(int)t->args[1]);
        break;
    case SYS_close_range:
        if(!(t->args[2]&4))descriptor_set(t,"range_before",t->args[0],t->args[1],0);
        break;
    case SYS_execve:case SYS_execveat:descriptor_set(t,"exec_before",0,~0u,1);break;
    case SYS_exit_group:descriptor_set(t,"exit_before",0,~0u,0);break;
    default:break;
    }
}
static void after(struct thread *t,int64_t result) {
    fprintf(output,"{\"kind\":\"result\",\"tid\":%d,\"sequence\":%lu,\"syscall\":%llu,\"result\":%lld,\"monotonic_ns\":%llu}\n",t->tid,t->sequence,(unsigned long long)t->number,(long long)result,(unsigned long long)now());flush();
    if(result<0)return;
    switch(t->number) {
    case SYS_open:case SYS_openat:case SYS_openat2:case SYS_creat:case SYS_dup:
        descriptor(t,"birth_after",(int)result);
        break;
    case SYS_dup2:case SYS_dup3:
        if(t->args[0]!=t->args[1])descriptor(t,"birth_after",(int)result);
        break;
    case SYS_fcntl:
        if(t->args[1]==F_DUPFD||t->args[1]==F_DUPFD_CLOEXEC)descriptor(t,"birth_after",(int)result);
        break;
    default:break;
    }
}
#define TRACE(n) BPF_JUMP(BPF_JMP|BPF_JEQ|BPF_K,(n),0,1),BPF_STMT(BPF_RET|BPF_K,SECCOMP_RET_TRACE)
static void filter(void) {
    struct sock_filter code[]={
        BPF_STMT(BPF_LD|BPF_W|BPF_ABS,offsetof(struct seccomp_data,arch)),
        BPF_JUMP(BPF_JMP|BPF_JEQ|BPF_K,AUDIT_ARCH_X86_64,1,0),
        BPF_STMT(BPF_RET|BPF_K,SECCOMP_RET_KILL_PROCESS),
        BPF_STMT(BPF_LD|BPF_W|BPF_ABS,offsetof(struct seccomp_data,nr)),
        TRACE(SYS_open),TRACE(SYS_openat),TRACE(SYS_openat2),TRACE(SYS_creat),
        TRACE(SYS_close),TRACE(SYS_close_range),TRACE(SYS_dup),TRACE(SYS_dup2),TRACE(SYS_dup3),
        TRACE(SYS_fcntl),TRACE(SYS_execve),TRACE(SYS_execveat),TRACE(SYS_exit_group),
        BPF_STMT(BPF_RET|BPF_K,SECCOMP_RET_ALLOW)};
    struct sock_fprog program={.len=sizeof(code)/sizeof(code[0]),.filter=code};
    if(prctl(PR_SET_NO_NEW_PRIVS,1,0,0,0))fail("no_new_privs");
    if(prctl(PR_SET_SECCOMP,SECCOMP_MODE_FILTER,&program))fail("seccomp filter");
}
static void resume(struct thread *t,int signal) {
    if(ptrace(t->pending?PTRACE_SYSCALL:PTRACE_CONT,t->tid,0,signal)<0&&errno!=ESRCH)fail("resume");
}
int main(int argc,char **argv) {
    if(argc<3){fprintf(stderr,"usage: fd-trace LEDGER PROGRAM [ARGS...]\n");return 2;}
    output=fopen(argv[1],"wx");if(!output)fail("create ledger");
    if(fcntl(fileno(output),F_SETFD,FD_CLOEXEC)<0)fail("ledger cloexec");
    pid_t root=fork();if(root<0)fail("fork");
    if(!root){
        fclose(output);
        const char *group=getenv("DATUM_TRACE_CGROUP");
        if(group){int fd=open(group,O_WRONLY|O_CLOEXEC);if(fd<0||dprintf(fd,"%ld",(long)getpid())<0)fail("enter cgroup");close(fd);}
        if(ptrace(PTRACE_TRACEME,0,0,0)<0)fail("traceme");
        raise(SIGSTOP);filter();execvp(argv[2],argv+2);fail("exec");
    }
    int status;if(waitpid(root,&status,0)!=root||!WIFSTOPPED(status))fail("initial stop");
    unsigned long options=PTRACE_O_TRACESYSGOOD|PTRACE_O_TRACESECCOMP|PTRACE_O_TRACECLONE|PTRACE_O_TRACEFORK|PTRACE_O_TRACEVFORK|PTRACE_O_TRACEEXEC|PTRACE_O_TRACEEXIT|PTRACE_O_EXITKILL;
    if(ptrace(PTRACE_SETOPTIONS,root,0,options)<0)fail("ptrace options");
    fprintf(output,"{\"kind\":\"root\",\"pid\":%d,\"observer_pid\":%d,\"monotonic_ns\":%llu}\n",root,getpid(),(unsigned long long)now());flush();resume(thread(root),0);
    int root_result=124;
    while(1){
        pid_t tid=waitpid(-1,&status,__WALL);if(tid<0){if(errno==EINTR)continue;if(errno==ECHILD)break;fail("waitpid");}
        struct thread *t=thread(tid);
        if(WIFEXITED(status)||WIFSIGNALED(status)){
            fprintf(output,"{\"kind\":\"exit\",\"tid\":%d,\"status\":%d,\"monotonic_ns\":%llu}\n",tid,status,(unsigned long long)now());flush();
            if(tid==root)root_result=WIFEXITED(status)?WEXITSTATUS(status):128+WTERMSIG(status);
            *t=(struct thread){0};continue;
        }
        if(!WIFSTOPPED(status))fail("unexpected wait state");
        unsigned event=(unsigned)status>>16;int signal=WSTOPSIG(status);
        if(event==PTRACE_EVENT_SECCOMP){
            struct ptrace_syscall_info info;
            if(ptrace(PTRACE_GET_SYSCALL_INFO,tid,sizeof(info),&info)<0||info.op!=PTRACE_SYSCALL_INFO_SECCOMP)fail("seccomp syscall info");
            t->pending=1;t->number=info.seccomp.nr;memcpy(t->args,info.seccomp.args,sizeof(t->args));t->sequence=++sequence;
            before(t);resume(t,0);continue;
        }
        if(event==PTRACE_EVENT_CLONE||event==PTRACE_EVENT_FORK||event==PTRACE_EVENT_VFORK){
            unsigned long child;if(ptrace(PTRACE_GETEVENTMSG,tid,0,&child)<0)fail("child event");
            fprintf(output,"{\"kind\":\"spawn\",\"parent_tid\":%d,\"child_tid\":%lu,\"event\":%u}\n",tid,child,event);flush();resume(t,0);continue;
        }
        if(event==PTRACE_EVENT_EXEC||event==PTRACE_EVENT_EXIT){
            fprintf(output,"{\"kind\":\"lifecycle\",\"tid\":%d,\"event\":%u,\"sequence\":%lu,\"monotonic_ns\":%llu}\n",tid,event,t->sequence,(unsigned long long)now());flush();resume(t,0);continue;
        }
        if(signal==(SIGTRAP|0x80)){
            struct ptrace_syscall_info info;if(ptrace(PTRACE_GET_SYSCALL_INFO,tid,sizeof(info),&info)<0)fail("syscall info");
            if(info.op==PTRACE_SYSCALL_INFO_EXIT&&t->pending){after(t,info.exit.rval);t->pending=0;}
            resume(t,0);continue;
        }
        resume(t,signal==SIGSTOP?0:signal);
    }
    fprintf(output,"{\"kind\":\"observer_end\",\"root_exit\":%d,\"sequences\":%lu}\n",root_result,sequence);flush();fclose(output);return root_result;
}
