#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

// Creates orphan process to be reaped.
int main() {
    pid_t pid    = getpid();
    pid_t group  = getpgid(0);
    pid_t parent = getppid();
    pid_t child;

    if ((child = fork()) < 0) {
        perror("could not create a child process");
        exit(1);
    }

    if (child > 0) {
        fprintf(
            stdout,
            "Parent\tpid=%d\tgroup=%d\tparent=%d\n",
            pid,
            group,
            parent
        );
        fflush(stdout);
        exit(0);
    } else {
        // Wait to be reparented.
        while (getppid() != parent) {
        }
        fprintf(
            stdout,
            "Child\tpid=%d\tgroup=%d\tparent=%d\n",
            getpid(),
            getpgid(0),
            getppid()
        );
        fflush(stdout);
        exit(0);
    }
}
