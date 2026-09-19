#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <errno.h>
#include <unistd.h>
// Diagnostic for this Linux x86_64 Mesa process only; never product code.
static int (*real_ioctl)(int, unsigned long, ...);
__attribute__((constructor)) static void init(void) {
    *(void **)(&real_ioctl) = dlsym(RTLD_NEXT, "ioctl");
    if (!real_ioctl) _exit(125);
}
static uint64_t ns(clockid_t id) {
    struct timespec t;
    if (clock_gettime(id, &t)) _exit(126);
    return (uint64_t)t.tv_sec * 1000000000ull + t.tv_nsec;
}
int ioctl(int fd, unsigned long request, ...) {
    va_list args;
    va_start(args, request);
    void *arg = va_arg(args, void *);
    va_end(args);
    // Mesa DRM requests have the third pointer argument. Other callers are
    // forwarded unchanged; this diagnostic is not a portable ioctl wrapper.
    int incoming_errno = errno;
    uint64_t start = ns(CLOCK_THREAD_CPUTIME_ID);
    errno = incoming_errno;
    int result = real_ioctl(fd, request, arg);
    int saved_errno = errno;
    uint64_t elapsed = ns(CLOCK_THREAD_CPUTIME_ID) - start;
    if (elapsed >= 100000) {
        char line[192];
        int n = snprintf(line, sizeof(line), "datum-ioctl realtime_ns=%llu request=0x%lx cpu_ns=%llu result=%d errno=%d\n", (unsigned long long)ns(CLOCK_REALTIME), request, (unsigned long long)elapsed, result, saved_errno);
        if (n > 0 && n < (int)sizeof(line)) (void)write(2, line, (size_t)n);
    }
    errno = saved_errno;
    return result;
}
