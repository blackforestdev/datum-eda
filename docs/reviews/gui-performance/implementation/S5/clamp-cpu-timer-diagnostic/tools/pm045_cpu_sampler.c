#define _GNU_SOURCE
/* Datum-owned diagnostic only: bounded main-thread CPU-timer PC sampling.
   No stack unwinding, allocation or stdio in signal handlers. */
#include <signal.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <fcntl.h>
#include <time.h>
#include <ucontext.h>
#include <string.h>
#include <errno.h>
#define CAP 65536
struct sample { uint64_t pc, cpu_ns; int64_t overrun; };
struct header { uint64_t magic, version, sequence, count, dropped, begin_cpu, end_cpu, begin_mono, end_mono, period, tid; };
static struct sample records[CAP];
static volatile sig_atomic_t active, count, dropped, sequence;
static uint64_t begin_cpu,begin_mono,period=1000000;
static timer_t timer_id;
static int fd=-1, sample_signal, control_signal;
static uint64_t ns(clockid_t clock) {struct timespec t;if(clock_gettime(clock,&t))_exit(121);return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;}
static void allwrite(const void *p,size_t n) {while(n){ssize_t w=write(fd,p,n);if(w<0&&errno==EINTR)continue;if(w<=0)_exit(122);p=(const char*)p+w;n-=w;}}
static void sample(int sig,siginfo_t *info,void *context) {
 (void)sig;int saved=errno;
 if(active){int i=count;if(i<CAP){ucontext_t *u=context;records[i]=(struct sample){(uint64_t)u->uc_mcontext.gregs[REG_RIP],ns(CLOCK_THREAD_CPUTIME_ID),info->si_overrun};count=i+1;}else dropped++;}
 errno=saved;
}
static void control(int sig) {
 (void)sig;int saved=errno;
 if(!active){count=0;dropped=0;sequence++;begin_cpu=ns(CLOCK_THREAD_CPUTIME_ID);begin_mono=ns(CLOCK_MONOTONIC);active=1;struct itimerspec t={{period/1000000000,period%1000000000},{period/1000000000,period%1000000000}};if(timer_settime(timer_id,0,&t,0))_exit(123);}
 else {active=0;struct itimerspec zero={0};if(timer_settime(timer_id,0,&zero,0))_exit(123);struct header h={0x504d303435435055,1,(uint64_t)sequence,(uint64_t)count,(uint64_t)dropped,begin_cpu,ns(CLOCK_THREAD_CPUTIME_ID),begin_mono,ns(CLOCK_MONOTONIC),period,(uint64_t)syscall(SYS_gettid)};allwrite(&h,sizeof(h));allwrite(records,(size_t)count*sizeof(records[0]));}
 errno=saved;
}
__attribute__((constructor)) static void init(void) {
 const char *path=getenv("PM045_CPU_SAMPLE_PATH");const char *target=getenv("PM045_CPU_SAMPLE_EXE");if(!path||!target)return;
 char exe[4096];ssize_t n=readlink("/proc/self/exe",exe,sizeof(exe)-1);if(n<0)_exit(124);exe[n]=0;if(strcmp(exe,target))return;
 const char *p=getenv("PM045_CPU_SAMPLE_PERIOD_NS");if(p)period=strtoull(p,0,10);if(period<100000||period>100000000)_exit(124);
 sample_signal=SIGRTMIN+8;control_signal=SIGRTMIN+9;if(control_signal>SIGRTMAX)_exit(124);
 sigset_t mask;if(sigprocmask(SIG_SETMASK,0,&mask))_exit(124);
 int signals[2]={sample_signal,control_signal};for(int i=0;i<2;i++){struct sigaction old;if(sigaction(signals[i],0,&old)||old.sa_handler!=SIG_DFL||sigismember(&mask,signals[i]))_exit(124);}
 fd=open(path,O_WRONLY|O_CREAT|O_EXCL|O_CLOEXEC,0600);if(fd<0)_exit(124);
 struct sigaction a={0};a.sa_sigaction=sample;a.sa_flags=SA_SIGINFO|SA_RESTART;sigemptyset(&a.sa_mask);sigaddset(&a.sa_mask,control_signal);if(sigaction(sample_signal,&a,0))_exit(124);
 struct sigaction b={0};b.sa_handler=control;b.sa_flags=SA_RESTART;sigemptyset(&b.sa_mask);sigaddset(&b.sa_mask,sample_signal);if(sigaction(control_signal,&b,0))_exit(124);
 struct sigevent e={0};e.sigev_notify=SIGEV_THREAD_ID;e.sigev_signo=sample_signal;e._sigev_un._tid=syscall(SYS_gettid);if(timer_create(CLOCK_THREAD_CPUTIME_ID,&e,&timer_id))_exit(124);
}
