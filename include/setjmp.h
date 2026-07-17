#ifndef _SETJMP_H
#define _SETJMP_H

#ifdef __cplusplus
extern "C" {
#endif

#ifdef __aarch64__
typedef unsigned long jmp_buf[22];
typedef unsigned long sigjmp_buf[40];
#elif defined(__riscv)
typedef unsigned long jmp_buf[26];
typedef unsigned long sigjmp_buf[44];
#else
typedef unsigned long jmp_buf[8];
typedef unsigned long sigjmp_buf[10];
#endif

int setjmp(jmp_buf);
void longjmp(jmp_buf, int);
int sigsetjmp(sigjmp_buf, int);
void siglongjmp(sigjmp_buf, int);

#ifdef __cplusplus
}
#endif

#endif
