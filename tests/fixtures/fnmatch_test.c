#include <fnmatch.h>
#include <stdio.h>
#include <string.h>

struct test_case {
    const char *pat;
    const char *str;
    int flags;
    int want;
};

static int fail;

static void check(const char *name, const struct test_case *t, int got)
{
    if (got != t->want) {
        printf("FAIL %s: fnmatch(\"%s\", \"%s\", %d) got %d want %d\n",
               name, t->pat, t->str, t->flags, got, t->want);
        fail = 1;
    }
}

int main(void)
{
    const struct test_case tests[] = {
        { "*[[:alpha:]]/*[[:alnum:]]", "a/b", FNM_PATHNAME, 0 },
        { "*[[:alpha:]]/*[[:alnum:]]", "1/b", FNM_PATHNAME, FNM_NOMATCH },
        { "[[:digit:]]", "5", 0, 0 },
        { "[[:digit:]]", "a", 0, FNM_NOMATCH },
        { "[![:space:]]", "x", 0, 0 },
        { "[![:space:]]", " ", 0, FNM_NOMATCH },
        { "[[:lower:]]", "a", 0, 0 },
        { "[[:lower:]]", "A", 0, FNM_NOMATCH },
        { "[[:upper:]]", "A", 0, 0 },
        { "[[:upper:]]", "a", 0, FNM_NOMATCH },
        { "[[:xdigit:]]", "f", 0, 0 },
        { "[[:xdigit:]]", "g", 0, FNM_NOMATCH },
        { "[[:punct:]]", "!", 0, 0 },
        { "[[:punct:]]", "a", 0, FNM_NOMATCH },
        { "[[:alnum:][:space:]]", " ", 0, 0 },
        { "[[=a=]]", "a", 0, 0 },
        { "[[=a=]]", "A", 0, FNM_NOMATCH },
        { 0 },
    };

    for (const struct test_case *t = tests; t->pat; t++) {
        check("posix class", t, fnmatch(t->pat, t->str, t->flags));
    }

    if (!fail) printf("OK\n");
    return fail;
}
