#include <string.h>
#include "arpa/nameser.h"

int main(void) {
    unsigned char pkt_simple[] = {
        3, 'w', 'w', 'w',
        7, 'e', 'x', 'a', 'm', 'p', 'l', 'e',
        3, 'c', 'o', 'm',
        0
    };
    char name[256];
    int r;

    r = dn_expand(pkt_simple, pkt_simple + sizeof(pkt_simple),
                  pkt_simple, name, sizeof(name));
    if (r != 17) return 1;
    if (strcmp(name, "www.example.com") != 0) return 2;

    unsigned char pkt_compress[] = {
        3, 'w', 'w', 'w',
        7, 'e', 'x', 'a', 'm', 'p', 'l', 'e',
        3, 'c', 'o', 'm',
        0,
        4, 'm', 'a', 'i', 'l',
        0xc0, 4
    };
    r = dn_expand(pkt_compress, pkt_compress + sizeof(pkt_compress),
                  pkt_compress + 17, name, sizeof(name));
    if (r != 7) return 3;
    if (strcmp(name, "mail.example.com") != 0) return 4;

    unsigned char pkt_empty[] = { 0 };
    r = dn_expand(pkt_empty, pkt_empty + 1, pkt_empty, name, 256);
    if (r != 1) return 5;
    if (name[0] != 0) return 6;

    return 0;
}
