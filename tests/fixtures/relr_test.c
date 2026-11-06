#include <stdio.h>
#include <string.h>

extern int relr_value(void);

static const char *msg = "relr ok";

int main(void) {
    if (strcmp(msg, "relr ok") != 0) return 1;
    int v = relr_value();
    printf("%s %d\n", msg, v);
    return v != 42;
}
