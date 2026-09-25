#define _GNU_SOURCE
#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/syscall.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
    assert(argc == 2);
    int mode = atoi(argv[1]);
    if (mode == 1) {
        /* Negative: direct kernel calls bypass libc interposition entirely. */
        int fd = syscall(SYS_openat, AT_FDCWD, "/dev/dri/renderD128", O_RDWR | O_CLOEXEC, 0);
        assert(fd >= 0); assert(syscall(SYS_close, fd) == 0);
        puts("expected_direct_client=1"); return 0;
    }
    if (mode == 2) {
        int a = open("/dev/dri/renderD128", O_RDWR | O_CLOEXEC), pair[2];
        assert(a >= 0 && socketpair(AF_UNIX, SOCK_STREAM, 0, pair) == 0);
        char byte='x', control[CMSG_SPACE(sizeof(int))]={0};
        struct iovec iov={&byte,1};
        struct msghdr message={.msg_iov=&iov,.msg_iovlen=1,.msg_control=control,.msg_controllen=sizeof(control)};
        struct cmsghdr *c=CMSG_FIRSTHDR(&message);c->cmsg_level=SOL_SOCKET;c->cmsg_type=SCM_RIGHTS;c->cmsg_len=CMSG_LEN(sizeof(int));
        memcpy(CMSG_DATA(c),&a,sizeof(a));assert(sendmsg(pair[0],&message,0)==1);
        memset(control,0,sizeof(control));message.msg_controllen=sizeof(control);
        assert(recvmsg(pair[1],&message,MSG_CMSG_CLOEXEC)==1);
        c=CMSG_FIRSTHDR(&message);int received;memcpy(&received,CMSG_DATA(c),sizeof(received));
        assert(fcntl(received,F_GETFD)==FD_CLOEXEC);
        assert(close(received)==0);close(pair[0]);close(pair[1]);
        int range=fcntl(a,F_DUPFD_CLOEXEC,128);assert(range>=128);
        assert(syscall(SYS_close_range,range,range,0)==0);
        pid_t child=fork();assert(child>=0);
        if(!child){execl("/bin/true","true",NULL);_exit(123);}
        int status;assert(waitpid(child,&status,0)==child&&WIFEXITED(status)&&WEXITSTATUS(status)==0);
        assert(close(a)==0);puts("expected_clients=1 rights_range_fork_exec=pass");return 0;
    }
    int a = open("/dev/dri/renderD128", O_RDWR | O_CLOEXEC);
    int b = open64("/dev/dri/renderD128", O_RDWR | O_CLOEXEC);
    int c = openat(AT_FDCWD, "/dev/dri/renderD128", O_RDWR | O_CLOEXEC);
    int d = openat64(AT_FDCWD, "/dev/dri/renderD128", O_RDWR | O_CLOEXEC);
    assert(a >= 0 && b >= 0 && c >= 0 && d >= 0);
    int e = dup(a), f = fcntl(a, F_DUPFD_CLOEXEC, 64);
    assert(e >= 0 && f >= 0);
    assert(dup2(a, b) == b); /* Final counter for overwritten independent client. */
    assert(dup3(c, d, O_CLOEXEC) == d);
    assert(dup2(a, a) == a); /* No fictitious close/open on a no-op. */
    errno = 0; assert(dup3(a, a, 0) == -1 && errno == EINVAL);
    errno = 0; assert(open("/path/that/does/not/exist", O_RDONLY) == -1 && errno == ENOENT);
    assert(fcntl(f, F_GETFD) == FD_CLOEXEC);
    for (int i = 0; i < 100; ++i) {
        int transient = open("/dev/null", O_RDONLY); assert(transient >= 0);
        assert(close(transient) == 0);
    }
    int fds[] = {a, b, c, d, e, f};
    for (unsigned int i = 0; i < sizeof(fds)/sizeof(fds[0]); ++i) assert(close(fds[i]) == 0);
    puts("expected_clients=4 expected_births=8 expected_retirements=8");
    return 0;
}
