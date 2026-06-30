#include "unistd.h"
#include "pthread_atfork.h"

static int prepare_seq[10], parent_seq[10], child_seq[10];
static int pi, pai, chi;

static void prep_a(void) { prepare_seq[pi++] = 'A'; }
static void prep_b(void) { prepare_seq[pi++] = 'B'; }
static void prep_c(void) { prepare_seq[pi++] = 'C'; }
static void par_a(void) { parent_seq[pai++] = 'A'; }
static void par_b(void) { parent_seq[pai++] = 'B'; }
static void par_c(void) { parent_seq[pai++] = 'C'; }
static void ch_a(void) { child_seq[chi++] = 'A'; }
static void ch_b(void) { child_seq[chi++] = 'B'; }
static void ch_c(void) { child_seq[chi++] = 'C'; }

static void dump(int *seq, int n) {
    for (int i = 0; i < n; i++) {
        char c = seq[i];
        write(1, &c, 1);
    }
    write(1, "\n", 1);
}

int main(void) {
    pthread_atfork(prep_a, par_a, ch_a);
    pthread_atfork(prep_b, par_b, ch_b);
    pthread_atfork(prep_c, par_c, ch_c);

    pid_t pid = fork();
    if (pid < 0) return 1;

    if (pid == 0) {
        dump(prepare_seq, pi);
        dump(child_seq, chi);
        _exit(0);
    }

    int status;
    waitpid(pid, &status, 0);
    dump(prepare_seq, pi);
    dump(parent_seq, pai);
    return 0;
}
