#include <stdio.h>
#include "ctype.h"

int main(void) {
    if (!isdigit('0')) return 1;
    if (isdigit('a')) return 2;
    if (!isalpha('A')) return 3;
    if (!isalnum('5')) return 4;
    if (isalnum(' ')) return 5;
    if (!isspace(' ')) return 6;
    if (!isspace('\n')) return 7;
    if (isspace('x')) return 8;
    if (!isxdigit('f')) return 9;
    if (isxdigit('g')) return 10;
    if (tolower('A') != 'a') return 11;
    if (toupper('z') != 'Z') return 12;
    if (!isprint(' ')) return 13;
    if (isprint(0x1f)) return 14;
    if (isgraph(' ')) return 15;
    if (!isgraph('~')) return 16;
    if (tolower(-1) != -1) return 17;
    puts("ctype ok");
    return 0;
}
