#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <signal.h>
#include <unistd.h>
#include <sys/syscall.h>
static uint64_t ns(clockid_t c){struct timespec t;clock_gettime(c,&t);return(uint64_t)t.tv_sec*1000000000+t.tv_nsec;}
__attribute__((noinline)) static uint64_t work(uint64_t v){for(int i=0;i<100000;i++)v=v*6364136223846793005ULL+1442695040888963407ULL;return v;}
int main(int argc,char **argv){int mode=argc>1?atoi(argv[1]):0;uint64_t start=ns(CLOCK_THREAD_CPUTIME_ID),wall=ns(CLOCK_MONOTONIC),v=1;if(mode)syscall(SYS_tgkill,getpid(),syscall(SYS_gettid),SIGRTMIN+9);if(mode==2){sigset_t s;sigemptyset(&s);sigaddset(&s,SIGRTMIN+8);sigprocmask(SIG_BLOCK,&s,0);uint64_t t=ns(CLOCK_THREAD_CPUTIME_ID);while(ns(CLOCK_THREAD_CPUTIME_ID)-t<50000000)v=work(v);sigprocmask(SIG_UNBLOCK,&s,0);}for(int i=0;i<5000;i++)v=work(v);if(mode)syscall(SYS_tgkill,getpid(),syscall(SYS_gettid),SIGRTMIN+9);printf("{\"mode\":%d,\"cpu_ns\":%llu,\"wall_ns\":%llu,\"value\":%llu}\n",mode,(unsigned long long)(ns(CLOCK_THREAD_CPUTIME_ID)-start),(unsigned long long)(ns(CLOCK_MONOTONIC)-wall),(unsigned long long)v);return 0;}
