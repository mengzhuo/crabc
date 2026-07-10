#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <locale.h>
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
    if (toascii(0x1ff) != 0x7f) return 18;
    if (!isascii(0x7f)) return 19;
    if (isascii(0x80)) return 20;

    locale_t loc = newlocale(LC_ALL_MASK, "C", (locale_t)0);
    if (loc == (locale_t)0) return 21;

    if (!isdigit_l('0', loc)) return 22;
    if (isdigit_l('a', loc)) return 23;
    if (!isalpha_l('A', loc)) return 24;
    if (tolower_l('A', loc) != 'a') return 25;
    if (toupper_l('z', loc) != 'Z') return 26;

    freelocale(loc);
    puts("ctype ok");
    return 0;
}
