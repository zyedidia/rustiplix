#include <stdint.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>
#include <stdio.h>
#include <sys/wait.h>

#include "syslib.h"

void delay_cycles() {
    usleep(500 * 1000);
    /* for (int i = 0; i < 1000000000 / 2; i++) { */
    /*     asm volatile ("nop"); */
    /* } */
}

int main() {
    fork();
    int pid = getpid();
    for (int i = 0; i < 5; i++) {
        printf("%d: loop %d\n", pid, i);
        delay_cycles();
    }
}
