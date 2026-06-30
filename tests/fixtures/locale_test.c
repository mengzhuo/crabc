#include <locale.h>
#include <langinfo.h>
#include <nl_types.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    char *loc = setlocale(LC_ALL, "");
    if (strcmp(loc, "C") != 0) return 1;

    loc = setlocale(LC_NUMERIC, "en_US.UTF-8");
    if (strcmp(loc, "C") != 0) return 2;

    struct lconv *lc = localeconv();
    if (strcmp(lc->decimal_point, ".") != 0) return 3;

    locale_t l = newlocale(LC_ALL_MASK, "C", (locale_t)0);
    if (l == (locale_t)0) return 4;
    freelocale(l);

    locale_t old = uselocale((locale_t)0);
    if (old != LC_GLOBAL_LOCALE) return 5;

    locale_t c = newlocale(LC_ALL_MASK, "C", (locale_t)0);
    if (c == (locale_t)0) return 6;

    locale_t prev = uselocale(c);
    if (prev != LC_GLOBAL_LOCALE) return 7;

    locale_t cur = uselocale((locale_t)0);
    if (cur != c) return 8;

    locale_t cur2 = uselocale((locale_t)0);
    if (cur2 != cur) return 9;

    locale_t dup = duplocale(c);
    if (dup == (locale_t)0) return 10;
    freelocale(dup);

    uselocale(old);
    freelocale(c);

    char *cs = nl_langinfo(CODESET);
    if (strcmp(cs, "UTF-8") != 0) return 11;

    char *radix = nl_langinfo(RADIXCHAR);
    if (strcmp(radix, ".") != 0) return 12;

    char *day = nl_langinfo(DAY_1);
    if (strcmp(day, "Sunday") != 0) return 13;

    char *abday = nl_langinfo(ABDAY_1);
    if (strcmp(abday, "Sun") != 0) return 14;

    char *mon = nl_langinfo(MON_1);
    if (strcmp(mon, "January") != 0) return 15;

    char *abmon = nl_langinfo(ABMON_1);
    if (strcmp(abmon, "Jan") != 0) return 16;

    char *am = nl_langinfo(AM_STR);
    if (strcmp(am, "AM") != 0) return 17;

    char *dt = nl_langinfo(D_T_FMT);
    if (dt[0] == '\0') return 18;

    char *yes = nl_langinfo(YESEXPR);
    if (yes[0] == '\0') return 19;

    nl_catd cat = catopen("nonexistent", 0);
    char *msg = catgets(cat, 1, 1, "default");
    if (strcmp(msg, "default") != 0) return 20;
    catclose(cat);

    printf("locale ok\n");
    return 0;
}
