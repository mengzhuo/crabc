#include "stdio.h"

int main(void) {
    int c = getchar();
    if (c != 'h') return 1;

    char buf[8];
    if (fgets(buf, sizeof(buf), stdin) == NULL) return 2;
    if (buf[0] != 'i' || buf[1] != '\n' || buf[2] != 0) return 3;

    c = fgetc(stdin);
    if (c != 'x') return 4;
    if (ungetc(c, stdin) != c) return 5;
    if (fgetc(stdin) != c) return 6;

    char big[4];
    size_t n = fread(big, 1, sizeof(big), stdin);
    if (n != 3) return 7;
    if (big[0] != 'y' || big[1] != '\n' || big[2] != 'z') return 8;

    puts("stdio input ok");
    return 0;
}
