#ifndef _SYS_CACHECTL_H
#define _SYS_CACHECTL_H

#ifdef __cplusplus
extern "C" {
#endif

int riscv_flush_icache(void *, void *, unsigned long);
int __riscv_flush_icache(void *, void *, unsigned long);

#ifdef __cplusplus
}
#endif

#endif
