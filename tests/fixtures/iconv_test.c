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

    cd = iconv_open("UTF-8", "UTF-16LE");
    if (cd == (iconv_t)-1) return 6;
    iconv_close(cd);

    cd = iconv_open("UTF-8", "ISO-8859-1");
    if (cd == (iconv_t)-1) return 7;
    iconv_close(cd);

    cd = iconv_open("UTF-8", "KOI8R");
    if (cd == (iconv_t)-1) return 8;
    iconv_close(cd);

    cd = iconv_open("UTF-8", "CP1252");
    if (cd == (iconv_t)-1) return 9;
    iconv_close(cd);

    cd = iconv_open("UTF-8", "GBK");
    if (cd == (iconv_t)-1) return 10;
    iconv_close(cd);

    cd = iconv_open("UTF-8", "Big5");
    if (cd == (iconv_t)-1) return 11;
    iconv_close(cd);

    cd = iconv_open("UTF-8", "SHIFTJIS");
    if (cd == (iconv_t)-1) return 12;
    iconv_close(cd);

    cd = iconv_open("UTF-8", "EUCJP");
    if (cd == (iconv_t)-1) return 13;
    iconv_close(cd);

    if (iconv_open("UTF-8", "NONSENSE") != (iconv_t)-1) return 14;

    printf("iconv ok\n");
    return 0;
}
