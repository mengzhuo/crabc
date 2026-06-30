#include <stdio.h>
#include <errno.h>
#include <string.h>
#include "sys/statvfs.h"

int main(void) {
    struct statvfs buf;

    if (statvfs("/", &buf) != 0) {
        fprintf(stderr, "statvfs(\"/\") failed: %s\n", strerror(errno));
        return 1;
    }

    if (buf.f_bsize == 0) {
        fprintf(stderr, "f_bsize is zero\n");
        return 2;
    }

    if (buf.f_frsize == 0) {
        fprintf(stderr, "f_frsize is zero\n");
        return 3;
    }

    if (buf.f_blocks == 0) {
        fprintf(stderr, "f_blocks is zero\n");
        return 4;
    }

    if (buf.f_files == 0) {
        fprintf(stderr, "f_files is zero\n");
        return 5;
    }

    if (buf.f_blocks < buf.f_bfree) {
        fprintf(stderr, "f_bfree > f_blocks\n");
        return 6;
    }

    if (buf.f_blocks < buf.f_bavail) {
        fprintf(stderr, "f_bavail > f_blocks\n");
        return 7;
    }

    if (buf.f_namemax == 0 || buf.f_namemax > (1U << 28)) {
        fprintf(stderr, "f_namemax bogus: %lu\n", (unsigned long)buf.f_namemax);
        return 8;
    }

    puts("statvfs ok");
    return 0;
}
