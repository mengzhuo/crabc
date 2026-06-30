#include <fenv.h>
#include <stdio.h>

int main(void) {
    if (fesetround(FE_TONEAREST) != 0) return 1;
    if (fegetround() != FE_TONEAREST) return 2;

    if (fesetround(FE_UPWARD) != 0) return 3;
    if (fegetround() != FE_UPWARD) return 4;

    if (fesetround(FE_DOWNWARD) != 0) return 5;
    if (fegetround() != FE_DOWNWARD) return 6;

    if (fesetround(FE_TOWARDZERO) != 0) return 7;
    if (fegetround() != FE_TOWARDZERO) return 8;

    if (fesetround(0x123) == 0) return 9;

    feclearexcept(FE_ALL_EXCEPT);
    if (fetestexcept(FE_ALL_EXCEPT) != 0) return 10;

    feraiseexcept(FE_INVALID);
    if ((fetestexcept(FE_ALL_EXCEPT) & FE_INVALID) == 0) return 11;

    fexcept_t flag;
    if (fegetexceptflag(&flag, FE_ALL_EXCEPT) != 0) return 12;

    feclearexcept(FE_ALL_EXCEPT);
    if (fetestexcept(FE_ALL_EXCEPT) != 0) return 13;

    if (fesetexceptflag(&flag, FE_INVALID) != 0) return 14;
    if ((fetestexcept(FE_ALL_EXCEPT) & FE_INVALID) == 0) return 15;

    fenv_t env;
    if (fegetenv(&env) != 0) return 16;
    if (fesetenv(FE_DFL_ENV) != 0) return 17;
    if (fegetround() != FE_TONEAREST) return 18;
    if (fesetenv(&env) != 0) return 19;
    if (fegetround() != FE_TOWARDZERO) return 20;

    if (fesetenv(FE_DFL_ENV) != 0) return 21;

    puts("OK");
    return 0;
}
