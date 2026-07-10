#include <fenv.h>
#include <stdio.h>

#define CHECK(cond, code) \
    do { \
        if (!(cond)) { \
            fprintf(stderr, "fenv_test failed at line %d with code %d\n", __LINE__, code); \
            return code; \
        } \
    } while (0)

int main(void) {
    CHECK(fesetround(FE_TONEAREST) == 0, 1);
    CHECK(fegetround() == FE_TONEAREST, 2);

    CHECK(fesetround(FE_UPWARD) == 0, 3);
    CHECK(fegetround() == FE_UPWARD, 4);

    CHECK(fesetround(FE_DOWNWARD) == 0, 5);
    CHECK(fegetround() == FE_DOWNWARD, 6);

    CHECK(fesetround(FE_TOWARDZERO) == 0, 7);
    CHECK(fegetround() == FE_TOWARDZERO, 8);

    CHECK(fesetround(0x123) != 0, 9);

    feclearexcept(FE_ALL_EXCEPT);
    CHECK(fetestexcept(FE_ALL_EXCEPT) == 0, 10);

    feraiseexcept(FE_INVALID);
    CHECK((fetestexcept(FE_ALL_EXCEPT) & FE_INVALID) != 0, 11);

    fexcept_t flag;
    CHECK(fegetexceptflag(&flag, FE_ALL_EXCEPT) == 0, 12);

    feclearexcept(FE_ALL_EXCEPT);
    CHECK(fetestexcept(FE_ALL_EXCEPT) == 0, 13);

    CHECK(fesetexceptflag(&flag, FE_INVALID) == 0, 14);
    CHECK((fetestexcept(FE_ALL_EXCEPT) & FE_INVALID) != 0, 15);

    fenv_t env;
    CHECK(fegetenv(&env) == 0, 16);
    CHECK(fesetenv(FE_DFL_ENV) == 0, 17);
    CHECK(fegetround() == FE_TONEAREST, 18);
    CHECK(fesetenv(&env) == 0, 19);
    CHECK(fegetround() == FE_TOWARDZERO, 20);

    CHECK(fesetenv(FE_DFL_ENV) == 0, 21);

    puts("OK");
    return 0;
}
