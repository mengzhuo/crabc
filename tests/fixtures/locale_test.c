#include <locale.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    char *loc = setlocale(LC_ALL, "");
    if (strcmp(loc, "C") != 0) return 1;

    loc = setlocale(LC_NUMERIC, "en_US.UTF-8");
    if (strcmp(loc, "C") != 0) return 2;

    struct lconv *lc = localeconv();
    if (strcmp(lc->decimal_point, ".") != 0) return 3;

    printf("locale ok\n");
    return 0;
}
