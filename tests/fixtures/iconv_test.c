#include <iconv.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    iconv_t cd = iconv_open("UTF-8", "UTF-8");
    if (cd == (iconv_t)-1) return 1;

    char in[] = "hello";
    char out[16];
    char *pin = in;
    char *pout = out;
    size_t inleft = strlen(in);
    size_t outleft = sizeof(out);

    size_t r = iconv(cd, &pin, &inleft, &pout, &outleft);
    if (r != 0) return 2;
    if (inleft != 0) return 3;
    *pout = '\0';
    if (strcmp(out, "hello") != 0) return 4;
    if (iconv_close(cd) != 0) return 5;

    if (iconv_open("UTF-8", "UTF-16") != (iconv_t)-1) return 6;

    printf("iconv ok\n");
    return 0;
}
