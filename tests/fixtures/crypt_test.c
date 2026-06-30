
#include <crypt.h>
#include <string.h>
#include <unistd.h>

static int failures = 0;
static char *p;

#define CHECK(expected, salt, key) \
    p = crypt(key, salt); \
    if (!p) p = "*"; \
    if (strcmp(p, expected) != 0) { \
        write(2, "FAIL\n", 5); \
        failures++; \
    }

int main() {
    /* MD5 */
    CHECK("$1$abcd0123$9Qcg8DyviekV3tDGMZynJ1", "$1$abcd0123$", "Xy01@#\x01\x02\x80\x7f\xff\r\n\x81\t !")
    CHECK("$1$$qRPK7m23GJusamGpoGLby/", "$1$$", "")
    CHECK("$1$salt$UsdFqFVB.FsuinRDK5eE..", "$1$salt$", "")

    /* SHA-256 */
    CHECK("$5$$3c2QQ0KjIU1OLtB29cl8Fplc2WN7X89bnoEjaR7tWu.", "$5$$", "")
    CHECK("$5$rounds=1234$abc0123456789$3VfDjPt05VHFn47C/ojFZ6KRPYrOjj1lLbH.dkF3bZ6", "$5$rounds=1234$abc0123456789$", "Xy01@#\x01\x02\x80\x7f\xff\r\n\x81\t !")
    CHECK("$5$saltstring$5B8vYYiY.CVt1RlTTf8KbXBH3hsxY/GNooZaBBGWEc5", "$5$saltstring", "Hello world!")

    /* SHA-512 */
    CHECK("$6$$/chiBau24cE26QQVW3IfIe68Xu5.JQ4E8Ie7lcRLwqxO5cxGuBhqF2HmTL.zWJ9zjChg3yJYFXeGBQ2y3Ba1d1", "$6$$", "")
    CHECK("$6$rounds=1234$abc0123456789$BCpt8zLrc/RcyuXmCDOE1ALqMXB2MH6n1g891HhFj8.w7LxGv.FTkqq6Vxc/km3Y0jE0j24jY5PIv/oOu6reg1", "$6$rounds=1234$abc0123456789$", "Xy01@#\x01\x02\x80\x7f\xff\r\n\x81\t !")
    CHECK("$6$saltstring$svn8UoSVapNtMuq1ukKS4tPQd8iKwSMHWjl/O817G3uBnIFNjnQJuesI68u4OTLiBFdcbYEdFCoEOfaS35inz1", "$6$saltstring", "Hello world!")

    /* Blowfish (bcrypt) */
    CHECK("$2a$04$012345678901234567890u8auMTJmy9uQv1pCMPSGmRjXec5nzCf6", "$2a$04$0123456789012345678901", "")
    CHECK("$2a$04$abcdefghijklmnopqrstuu8J3SjO9LQpndv9O3HW/e0PB1xKk.PJu", "$2a$04$abcdefghijklmnopqrstuv", "Aa@\xaa 0123456789")
    CHECK("$2y$04$abcdefghijklmnopqrstuubUAnPDiHn0JtKfNM4q6HN1ZsdaC1D8i", "$2y$04$abcdefghijklmnopqrstuv", "\xff\xff\xff\xa3\x33\x01\x40")

    /* Blowfish invalid salts return "*" */
    p = crypt("", "$2a$00$0123456789012345678901");
    if (!p || strcmp(p, "*") != 0) { write(2, "FAIL\n", 5); failures++; }
    p = crypt("", "$2a$08$01234567890123456789");
    if (!p || strcmp(p, "*") != 0) { write(2, "FAIL\n", 5); failures++; }

    if (failures == 0) write(1, "crypt ok\n", 9);
    return failures;
}
