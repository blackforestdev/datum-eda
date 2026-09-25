#define _GNU_SOURCE
/* First-party Linux x86_64 diagnostic only. Not linked into Datum.
 * Capture libc descriptor boundaries. Direct/hidden syscalls are NOT covered;
 * callers must audit actual coverage before using a total for qualification. */
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/sysmacros.h>
#include <time.h>
#include <unistd.h>

static int log_fd = -1;
static int (*next_open)(const char *, int, ...);
static int (*next_open64)(const char *, int, ...);
static int (*next_openat)(int, const char *, int, ...);
static int (*next_openat64)(int, const char *, int, ...);
static int (*next_close)(int);
static int (*next_dup)(int);
static int (*next_dup2)(int, int);
static int (*next_dup3)(int, int, int);
static int (*next_fcntl)(int, int, ...);

static void fail(const char *message) {
    (void)syscall(SYS_write, STDERR_FILENO, message, strlen(message));
    (void)syscall(SYS_exit_group, 124);
    __builtin_unreachable();
}
static uint64_t mono(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) fail("fd-lifecycle clock failed\n");
    return (uint64_t)t.tv_sec * 1000000000 + (uint64_t)t.tv_nsec;
}
static long read_proc(const char *path, char *bytes, size_t size) {
    int fd = (int)syscall(SYS_openat, AT_FDCWD, path, O_RDONLY | O_CLOEXEC, 0);
    if (fd < 0) return -1;
    long n = syscall(SYS_read, fd, bytes, size);
    int saved = errno;
    (void)syscall(SYS_close, fd);
    errno = saved;
    return n;
}
static unsigned long long process_start(void) {
    char bytes[4096];
    long n = read_proc("/proc/self/stat", bytes, sizeof(bytes) - 1);
    if (n <= 0) fail("fd-lifecycle process identity unavailable\n");
    bytes[n] = 0;
    char *p = strrchr(bytes, ')');
    if (!p) fail("fd-lifecycle invalid process identity\n");
    p += 2;
    for (int field = 3; field < 22; ++field) {
        p = strchr(p, ' ');
        if (!p) fail("fd-lifecycle missing process start\n");
        ++p;
    }
    return strtoull(p, NULL, 10);
}
struct observation {
    int present;
    unsigned int major, minor;
    uint64_t at;
    long length;
    char raw[4096];
};
static struct observation observe(int fd) {
    int saved = errno;
    struct observation o = {0};
    if (log_fd >= 0) {
        struct stat st;
        if (!fstat(fd, &st) && S_ISCHR(st.st_mode) && major(st.st_rdev) == 226) {
            char path[64];
            int length = snprintf(path, sizeof(path), "/proc/self/fdinfo/%d", fd);
            if (length <= 0 || (size_t)length >= sizeof(path)) fail("fd-lifecycle path overflow\n");
            o.present = 1; o.major = major(st.st_rdev); o.minor = minor(st.st_rdev);
            o.at = mono(); o.length = read_proc(path, o.raw, sizeof(o.raw));
            if (o.length <= 0 || (size_t)o.length >= sizeof(o.raw))
                fail("fd-lifecycle DRM final reading missing or truncated\n");
        }
    }
    errno = saved;
    return o;
}
static void receipt(const char *event, int fd, int result, int operation_errno,
                    const struct observation *o) {
    if (!o->present) return;
    int saved = errno;
    char line[9216];
    int n = snprintf(line, sizeof(line),
        "fd_lifecycle version=1 event=%s pid=%ld start=%llu tid=%ld fd=%d result=%d errno=%d major=%u minor=%u observed_ns=%llu receipt_ns=%llu raw=",
        event, (long)getpid(), process_start(), syscall(SYS_gettid), fd, result,
        operation_errno, o->major, o->minor, (unsigned long long)o->at,
        (unsigned long long)mono());
    if (n < 0 || (size_t)n + (size_t)o->length * 2 + 1 > sizeof(line))
        fail("fd-lifecycle record overflow\n");
    static const char hex[] = "0123456789abcdef";
    for (long i = 0; i < o->length; ++i) {
        unsigned char byte = (unsigned char)o->raw[i];
        line[n++] = hex[byte >> 4]; line[n++] = hex[byte & 15];
    }
    line[n++] = '\n';
    if (syscall(SYS_write, log_fd, line, n) != n) fail("fd-lifecycle incomplete write\n");
    errno = saved;
}
static void birth(const char *event, int result) {
    if (result >= 0) {
        struct observation o = observe(result);
        receipt(event, result, result, 0, &o);
    }
}
__attribute__((constructor)) static void initialize(void) {
#define RESOLVE(name) do { *(void **)(&next_##name) = dlsym(RTLD_NEXT, #name); if (!next_##name) fail("fd-lifecycle symbol missing: " #name "\n"); } while (0)
    RESOLVE(open); RESOLVE(open64); RESOLVE(openat); RESOLVE(openat64);
    RESOLVE(close); RESOLVE(dup); RESOLVE(dup2); RESOLVE(dup3); RESOLVE(fcntl);
#undef RESOLVE
    const char *path = getenv("DATUM_FD_LIFECYCLE_LOG");
    if (path && *path) {
        log_fd = (int)syscall(SYS_openat, AT_FDCWD, path,
                             O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC, 0600);
        if (log_fd < 0) fail("fd-lifecycle log unavailable\n");
    }
}
static int has_mode(int flags) {
    return (flags & O_CREAT) || (flags & O_TMPFILE) == O_TMPFILE;
}
#define OPEN_WRAPPER(name) \
int name(const char *path, int flags, ...) { \
    mode_t mode = 0; \
    if (has_mode(flags)) { va_list args; va_start(args, flags); mode = va_arg(args, mode_t); va_end(args); } \
    int result = next_##name ? next_##name(path, flags, mode) : (int)syscall(SYS_openat, AT_FDCWD, path, flags, mode); \
    int saved = errno; birth(#name, result); errno = saved; return result; \
}
OPEN_WRAPPER(open)
OPEN_WRAPPER(open64)
#define OPENAT_WRAPPER(name) \
int name(int dir, const char *path, int flags, ...) { \
    mode_t mode = 0; \
    if (has_mode(flags)) { va_list args; va_start(args, flags); mode = va_arg(args, mode_t); va_end(args); } \
    int result = next_##name ? next_##name(dir, path, flags, mode) : (int)syscall(SYS_openat, dir, path, flags, mode); \
    int saved = errno; birth(#name, result); errno = saved; return result; \
}
OPENAT_WRAPPER(openat)
OPENAT_WRAPPER(openat64)
int close(int fd) {
    struct observation o = observe(fd);
    int result = next_close ? next_close(fd) : (int)syscall(SYS_close, fd);
    int saved = errno; receipt("close", fd, result, result < 0 ? saved : 0, &o);
    errno = saved; return result;
}
int dup(int fd) {
    int result = next_dup ? next_dup(fd) : (int)syscall(SYS_dup, fd);
    int saved = errno; birth("dup", result); errno = saved; return result;
}
int dup2(int fd, int target) {
    struct observation old = fd == target ? (struct observation){0} : observe(target);
    int result = next_dup2 ? next_dup2(fd, target) : (int)syscall(SYS_dup2, fd, target);
    int saved = errno;
    if (result >= 0 && fd != target) { receipt("replace", target, result, 0, &old); birth("dup2", result); }
    errno = saved; return result;
}
int dup3(int fd, int target, int flags) {
    struct observation old = fd == target ? (struct observation){0} : observe(target);
    int result = next_dup3 ? next_dup3(fd, target, flags) : (int)syscall(SYS_dup3, fd, target, flags);
    int saved = errno;
    if (result >= 0) { receipt("replace", target, result, 0, &old); birth("dup3", result); }
    errno = saved; return result;
}
int fcntl(int fd, int command, ...) {
    int result;
    va_list args; va_start(args, command);
    switch (command) {
    case F_GETFD: case F_GETFL: case F_GETOWN: case F_GETSIG:
    case F_GETLEASE: case F_GETPIPE_SZ: case F_GET_SEALS:
        result = next_fcntl ? next_fcntl(fd, command) : (int)syscall(SYS_fcntl, fd, command, 0); break;
    case F_DUPFD: case F_DUPFD_CLOEXEC: case F_SETFD: case F_SETFL:
    case F_SETOWN: case F_SETSIG: case F_SETLEASE: case F_NOTIFY:
    case F_SETPIPE_SZ: case F_ADD_SEALS: {
        int arg = va_arg(args, int);
        result = next_fcntl ? next_fcntl(fd, command, arg) : (int)syscall(SYS_fcntl, fd, command, arg); break;
    }
    case F_GETLK: case F_SETLK: case F_SETLKW: case F_OFD_GETLK:
    case F_OFD_SETLK: case F_OFD_SETLKW: case F_GETOWN_EX: case F_SETOWN_EX: {
        void *arg = va_arg(args, void *);
        result = next_fcntl ? next_fcntl(fd, command, arg) : (int)syscall(SYS_fcntl, fd, command, arg); break;
    }
    default: fail("fd-lifecycle unsupported fcntl command; trial invalid\n");
    }
    va_end(args);
    int saved = errno;
    if (command == F_DUPFD || command == F_DUPFD_CLOEXEC) birth("fcntl_dup", result);
    errno = saved; return result;
}
