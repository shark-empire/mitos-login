/* Spawns real children with known exit codes / termination signals, waits for
 * them for real, and prints the raw status plus what the libc W* macros say —
 * ground truth for src/sys/mod.rs's own status-decoding reimplementation. */
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>
#include <signal.h>

static void report(const char *label, int status) {
    printf("%s raw=%d exited=%d exitstatus=%d signaled=%d termsig=%d\n",
           label, status, WIFEXITED(status), WIFEXITED(status)?WEXITSTATUS(status):-1,
           WIFSIGNALED(status), WIFSIGNALED(status)?WTERMSIG(status):-1);
}

int main(void) {
    printf("WNOHANG=%d\n", WNOHANG);
    int cases[] = {0, 1, 42, 255};
    for (int i = 0; i < 4; i++) {
        pid_t p = fork();
        if (p == 0) { _exit(cases[i]); }
        int status; waitpid(p, &status, 0);
        char label[32]; snprintf(label, sizeof label, "exit(%d):", cases[i]);
        report(label, status);
    }
    int sigs[] = {SIGTERM, SIGKILL, SIGSEGV};
    for (int i = 0; i < 3; i++) {
        pid_t p = fork();
        if (p == 0) { raise(sigs[i]); _exit(99); }
        int status; waitpid(p, &status, 0);
        char label[32]; snprintf(label, sizeof label, "sig(%d):", sigs[i]);
        report(label, status);
    }
    return 0;
}
