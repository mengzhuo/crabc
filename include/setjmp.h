#ifndef _SETJMP_H
#define _SETJMP_H

#ifdef __cplusplus
extern "C" {
#endif

typedef unsigned long jmp_buf[8];
typedef unsigned long sigjmp_buf[10];

int setjmp(jmp_buf);
void longjmp(jmp_buf, int);
int sigsetjmp(sigjmp_buf, int);
void siglongjmp(sigjmp_buf, int);

#ifdef __cplusplus
}
#endif

#endif
