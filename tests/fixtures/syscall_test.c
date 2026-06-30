#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include <sys/syscall.h>

static int t_status;

static void t_error(const char *fmt, ...) {
    t_status = 1;
}

#define T(f) (!(f) && (t_error(#f " failed: %s\n", strerror(errno)), 0))

int main(void)
{
	char buf[1] = {1};
	int fd;
	int r;

	T((fd = open("/dev/zero", O_RDONLY)) >= 0);
	T((r = syscall(SYS_read, fd, buf, 1)) == 1);
	if (buf[0] != 0)
		t_error("read %d instead of 0\n", buf[0]);

	close(fd);

	if (t_status)
		return 1;
	puts("OK");
	return 0;
}
