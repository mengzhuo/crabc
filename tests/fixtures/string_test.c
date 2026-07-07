#include <stdio.h>
#include "string.h"

int main(void) {
    char buf[64];

    strcpy(buf, "hello");
    if (strcmp(buf, "hello") != 0) return 1;

    strncpy(buf, "abc", 5);
    if (strncmp(buf, "abc", 5) != 0) return 2;

    strcat(buf, "de");
    if (strcmp(buf, "abcde") != 0) return 3;

    strncat(buf, "fgh", 1);
    if (strcmp(buf, "abcdef") != 0) return 4;

    char abc[16] = "abc";
    if (strlen(abc) != 3) return 5;
    if (strnlen(abc, 10) != 3) return 6;
    if (strnlen(abc, 2) != 2) return 7;

    if (strchr("abc", 'b') != "abc" + 1) return 8;
    if (strrchr("abca", 'a') != "abca" + 3) return 9;

    if (strspn("123abc", "123") != 3) return 10;
    if (strcspn("123abc", "abc") != 3) return 11;
    if (strpbrk("123abc", "ba") != "123abc" + 3) return 12;
    if (strstr("hello world", "world") != "hello world" + 6) return 13;
    if (strstr("hello", "") != "hello") return 14;

    char mem[] = {1, 2, 3, 2, 1};
    if (memchr(mem, 2, sizeof(mem)) != mem + 1) return 15;
    if (memrchr(mem, 2, sizeof(mem)) != mem + 3) return 16;

    char tok[] = "a,b,c";
    char *r;
    r = strtok(tok, ",");
    if (r == NULL || strcmp(r, "a") != 0) return 17;
    r = strtok(NULL, ",");
    if (r == NULL || strcmp(r, "b") != 0) return 18;
    r = strtok(NULL, ",");
    if (r == NULL || strcmp(r, "c") != 0) return 19;
    if (strtok(NULL, ",") != NULL) return 20;

    puts("string ok");
    return 0;
}
